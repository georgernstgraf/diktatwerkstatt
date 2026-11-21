//! Die SQLite-Datei: zwei Tabellen, `diktat` und `prompt`.
//!
//! Das Schema steht in `docs/ai/DOMAIN.md`; dieses Modul setzt es durch und
//! bietet die Vorgänge, mit denen der Zustand wechselt:
//!
//! - [`Store::begin_dictation`] — ein Diktat entsteht, das Arbeitsverzeichnis
//!   auch, das älteste fällt aus dem FIFO (Invarianten 1 und 9).
//! - [`Store::commit_raw_text`] → [`Store::remove_working_dir`] — der
//!   Abschluss, in dieser Reihenfolge (Invariante 3). [`Store::finish_transcription`]
//!   ist der eine Weg, der beide ruft.
//! - [`Store::append_raw`] und [`Store::correct_raw`] — jede Änderung am rohen
//!   Transkript verwirft das polierte (Invariante 6); Append nur, wenn die
//!   Transkription abgeschlossen ist (Invariante 5).
//! - [`Store::set_polished`] — genau ein Slot, vollständig überschrieben
//!   (Invariante 7).
//!
//! Es gibt **kein** zweites Zustandsfeld: `audio_dir` ist der einzige Marker
//! (Invariante 1), `raw_text` die einzige Speicherung des rohen Transkripts
//! (Invariante 4).

use std::path::{Path, PathBuf};

use chrono::{Local, SecondsFormat};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{Error, Result};
use crate::fragments::WorkingDir;

/// Kennung eines Diktats.
pub type DiktatId = i64;

/// Kennung eines Prompts.
pub type PromptId = i64;

/// Die Schemaversion, die dieser Kern schreibt und versteht.
const SCHEMA_VERSION: i64 = 1;

/// Schema v1 — wörtlich `docs/ai/DOMAIN.md`.
///
/// `STRICT` hält die Spaltentypen ehrlich; `audio_dir` ist die einzige
/// `NULL`-bare Zustandsangabe, und `polished_text` ist ein Slot, keine Historie.
const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS diktat (
    id            INTEGER PRIMARY KEY,
    created_at    TEXT    NOT NULL,
    audio_dir     TEXT,
    raw_text      TEXT    NOT NULL DEFAULT '',
    polished_text TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS "prompt" (
    id          INTEGER PRIMARY KEY,
    alias       TEXT    NOT NULL UNIQUE,
    is_system   INTEGER NOT NULL CHECK (is_system IN (0, 1)),
    prompt_text TEXT    NOT NULL
) STRICT;
"#;

const SELECT_DIKTAT: &str = "SELECT id, created_at, audio_dir, raw_text, polished_text FROM diktat";

/// Welcher Reiter der Workbench gemeint ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// „rohes Transkript".
    Raw,
    /// „poliertes Transkript".
    Polished,
}

/// Ein Diktat, wie es in der Datenbank steht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diktat {
    /// Kennung, zugleich die Sortier- und FIFO-Reihenfolge.
    pub id: DiktatId,
    /// ISO-8601 in Ortszeit, z. B. `2026-11-21T09:31:00+01:00`.
    pub created_at: String,
    /// Das Arbeitsverzeichnis — `None` heißt: Transkription abgeschlossen.
    pub audio_dir: Option<PathBuf>,
    /// Das rohe Transkript; nie umformatiert.
    pub raw_text: String,
    /// Das polierte Transkript, höchstens eines.
    pub polished_text: Option<String>,
}

impl Diktat {
    /// Ist die Transkription noch nicht abgeschlossen?
    ///
    /// Das ist Invariante 1: gesetztes `audio_dir` ⇔ offen. Es gibt kein
    /// zweites Feld, das man dazu befragen könnte.
    pub fn is_open(&self) -> bool {
        self.audio_dir.is_some()
    }

    /// Der Text, den die Workbench in diesem Reiter zeigt — und genau der Text,
    /// den die Lieferung in die Zwischenablage schreibt (Invariante 8).
    ///
    /// Der polierte Reiter ist leer, solange es kein poliertes Transkript gibt;
    /// er zeigt **nie** den Rohtext als Ersatz. Das ist der einzige Leseweg zum
    /// sichtbaren Text: einen zweiten gibt es nicht, damit Fenster und
    /// Zwischenablage nicht auseinanderlaufen können.
    pub fn view_text(&self, view: View) -> &str {
        match view {
            View::Raw => &self.raw_text,
            View::Polished => self.polished_text.as_deref().unwrap_or(""),
        }
    }
}

/// Ein Prompt der Bibliothek.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// Kennung.
    pub id: PromptId,
    /// Anzeigename, eindeutig, z. B. „DE-sachlich".
    pub alias: String,
    /// Rolle und Ausgabeformat (`System`) oder die Aufgabe (`User`).
    pub kind: PromptKind,
    /// Die Anweisung; im User-Prompt ist `{{text}}` das rohe Transkript.
    pub prompt_text: String,
}

/// Art eines Prompts — die Spalte `is_system`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    /// System-Prompt: Rolle, Ausgabeformat.
    System,
    /// User-Prompt: die eigentliche Aufgabe.
    User,
}

impl PromptKind {
    fn as_int(self) -> i64 {
        match self {
            PromptKind::System => 1,
            PromptKind::User => 0,
        }
    }

    fn from_int(value: i64) -> Self {
        if value == 1 {
            PromptKind::System
        } else {
            PromptKind::User
        }
    }
}

/// Was ein offenes Diktat beim Start braucht (Wiederaufnahme).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resume {
    /// Der Datenbankstand.
    pub diktat: Diktat,
    /// Fragmentnummern, deren `.wav` noch kein `.txt` hat — nur sie werden
    /// transkribiert.
    pub pending_fragments: Vec<u32>,
    /// Der verkettete Rohtext, sobald nichts mehr offen ist; sonst `None`.
    pub assembled_text: Option<String>,
}

/// Die Datenbank der Diktatwerkstatt.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Öffnet (oder erzeugt) die Datenbank unter `path` und legt das Schema an.
    ///
    /// Fehlende Elternverzeichnisse werden angelegt — das Datenverzeichnis
    /// entsteht beim ersten Start.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        Self::from_connection(Connection::open(path)?)
    }

    /// Öffnet eine Datenbank im Speicher — für Tests und Prüfungen.
    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        // Ein Diktat ist Mandantendaten: der Commit muss die Platte erreichen,
        // bevor das Audio verschwindet (Invariante 3).
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let found: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        if found > SCHEMA_VERSION {
            return Err(Error::SchemaTooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        if found < 1 {
            self.conn.execute_batch(SCHEMA_V1)?;
            self.conn
                .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(())
    }

    /// Legt ein Diktat an, legt sein Arbeitsverzeichnis an und räumt den FIFO auf
    /// (Invarianten 1 und 9).
    ///
    /// `audio_root` ist das `audio/`-Verzeichnis des Datenverzeichnisses; das
    /// Diktat bekommt `<audio_root>/<id>/`, wie in `docs/ai/DOMAIN.md`
    /// festgelegt. `fifo_limit` ist das N aus `config.json`; Werte unter eins
    /// werden als eins behandelt, weil ein Diktat Platz haben muss.
    ///
    /// Ein Verzeichnis ohne Diktat wäre Müll, ein Diktat mit gesetztem
    /// `audio_dir` ohne Verzeichnis wäre eine Lüge über seinen Zustand: deshalb
    /// entsteht das Verzeichnis vor dem Commit, und die Verzeichnisse der
    /// verdrängten Diktate verschwinden erst nach dem Commit.
    pub fn begin_dictation(&mut self, audio_root: &Path, fifo_limit: usize) -> Result<Diktat> {
        let limit = i64::try_from(fifo_limit.max(1)).unwrap_or(i64::MAX);
        let created_at = now_local_iso8601();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO diktat (created_at, audio_dir, raw_text) VALUES (?1, '', '')",
            params![created_at],
        )?;
        let id = tx.last_insert_rowid();
        let directory = audio_root.join(id.to_string());
        WorkingDir::new(&directory).create()?;
        tx.execute(
            "UPDATE diktat SET audio_dir = ?1 WHERE id = ?2",
            params![path_for_db(&directory), id],
        )?;

        let evicted: Vec<Option<String>> = {
            let mut statement = tx.prepare(
                "SELECT audio_dir FROM diktat \
                 WHERE id NOT IN (SELECT id FROM diktat ORDER BY id DESC LIMIT ?1)",
            )?;
            let rows = statement.query_map([limit], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        if !evicted.is_empty() {
            tx.execute(
                "DELETE FROM diktat \
                 WHERE id NOT IN (SELECT id FROM diktat ORDER BY id DESC LIMIT ?1)",
                [limit],
            )?;
        }
        tx.commit()?;

        for directory in evicted.into_iter().flatten() {
            let _ = WorkingDir::new(directory).remove();
        }

        self.get(id)
    }

    /// Liest ein Diktat.
    pub fn get(&self, id: DiktatId) -> Result<Diktat> {
        self.conn
            .query_row(
                &format!("{SELECT_DIKTAT} WHERE id = ?1"),
                [id],
                diktat_from_row,
            )
            .optional()?
            .ok_or(Error::DiktatNotFound { id })
    }

    /// Die letzten `limit` Diktate, neueste zuerst — der FIFO, wie ihn die
    /// Workbench zeigt.
    pub fn list(&self, limit: usize) -> Result<Vec<Diktat>> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut statement = self
            .conn
            .prepare(&format!("{SELECT_DIKTAT} ORDER BY id DESC LIMIT ?1"))?;
        let rows = statement.query_map([limit], diktat_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Alle Diktate mit gesetztem `audio_dir` — die Wiederaufnahme beim Start.
    pub fn open_dictations(&self) -> Result<Vec<Diktat>> {
        let mut statement = self.conn.prepare(&format!(
            "{SELECT_DIKTAT} WHERE audio_dir IS NOT NULL ORDER BY id ASC"
        ))?;
        let rows = statement.query_map([], diktat_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Was ein offenes Diktat noch braucht: die offenen Fragmente und, sobald
    /// nichts mehr offen ist, den fertigen Rohtext.
    ///
    /// Fehlt das Verzeichnis schon (Abbruch zwischen dem Entfernen und dem
    /// Setzen von `audio_dir = NULL`), gilt der bereits committete `raw_text`
    /// als der fertige Text — Invariante 3 hat ihn vorher geschrieben.
    pub fn resume(&self, id: DiktatId) -> Result<Resume> {
        let diktat = self.get(id)?;
        let Some(directory) = diktat.audio_dir.as_ref() else {
            return Err(Error::TranscriptionClosed { id });
        };

        let working_dir = WorkingDir::new(directory);
        if !working_dir.exists() {
            return Ok(Resume {
                pending_fragments: Vec::new(),
                assembled_text: Some(diktat.raw_text.clone()),
                diktat,
            });
        }

        let pending_fragments = working_dir.pending()?;
        let assembled_text = if pending_fragments.is_empty() {
            Some(working_dir.assemble()?)
        } else {
            None
        };
        Ok(Resume {
            diktat,
            pending_fragments,
            assembled_text,
        })
    }

    /// Schreibt das rohe Transkript — **Schritt 1** des Abschlusses.
    ///
    /// Invariante 3: der Text steht in der Datenbank, bevor das Audio
    /// verschwindet. Invariante 6: jede Änderung am rohen Transkript verwirft
    /// das polierte, hier ohne Ausnahme.
    ///
    /// Nur für ein offenes Diktat; ein abgeschlossenes ändert man über
    /// [`correct_raw`].
    ///
    /// [`correct_raw`]: Store::correct_raw
    pub fn commit_raw_text(&mut self, id: DiktatId, raw_text: &str) -> Result<Diktat> {
        let diktat = self.get(id)?;
        if diktat.audio_dir.is_none() {
            return Err(Error::TranscriptionClosed { id });
        }
        self.conn.execute(
            "UPDATE diktat SET raw_text = ?1, polished_text = NULL WHERE id = ?2",
            params![raw_text, id],
        )?;
        self.get(id)
    }

    /// Entfernt das Arbeitsverzeichnis und setzt `audio_dir` auf `NULL` —
    /// **Schritt 2** des Abschlusses.
    ///
    /// Invariante 3: der Text ist zu diesem Zeitpunkt schon committet. Ist das
    /// Diktat bereits geschlossen, ist der Aufruf folgenlos.
    pub fn remove_working_dir(&mut self, id: DiktatId) -> Result<Diktat> {
        let diktat = self.get(id)?;
        let Some(directory) = diktat.audio_dir.clone() else {
            return Ok(diktat);
        };
        WorkingDir::new(&directory).remove()?;
        self.conn
            .execute("UPDATE diktat SET audio_dir = NULL WHERE id = ?1", [id])?;
        self.get(id)
    }

    /// Der Abschluss: erst der Text, dann das Audio — der einzige Weg, den die
    /// App geht (Invariante 3).
    ///
    /// Gibt den neuen Fensterstand zurück; sein [`Diktat::view_text`] ist der
    /// Text, der in die Zwischenablage gehört (Invariante 8).
    pub fn finish_transcription(&mut self, id: DiktatId, raw_text: &str) -> Result<Diktat> {
        self.commit_raw_text(id, raw_text)?;
        self.remove_working_dir(id)
    }

    /// Hängt Rohtext an — nur bei abgeschlossener Transkription (Invariante 5:
    /// typisierter Fehler, kein Panic).
    ///
    /// `more` wird unverändert angehängt, ohne Trenner (Invariante 10): was
    /// zwischen zwei Aufnahmen stehen soll, entscheidet der Aufrufer. Das
    /// polierte Transkript ist danach verworfen (Invariante 6), ohne
    /// Neupolitur.
    pub fn append_raw(&mut self, id: DiktatId, more: &str) -> Result<Diktat> {
        let diktat = self.get(id)?;
        if let Some(directory) = &diktat.audio_dir {
            return Err(Error::TranscriptionOpen {
                id,
                audio_dir: directory.display().to_string(),
            });
        }
        let joined = format!("{}{more}", diktat.raw_text);
        self.conn.execute(
            "UPDATE diktat SET raw_text = ?1, polished_text = NULL WHERE id = ?2",
            params![joined, id],
        )?;
        self.get(id)
    }

    /// Korrigiert das rohe Transkript — der Edit im rohen Reiter.
    ///
    /// Der Text wird vollständig ersetzt und nie umformatiert; das polierte
    /// Transkript ist danach verworfen (Invariante 6), ohne Neupolitur.
    pub fn correct_raw(&mut self, id: DiktatId, raw_text: &str) -> Result<Diktat> {
        self.conn.execute(
            "UPDATE diktat SET raw_text = ?1, polished_text = NULL WHERE id = ?2",
            params![raw_text, id],
        )?;
        self.get(id)
    }

    /// Setzt das polierte Transkript — genau ein Slot, vollständig überschrieben
    /// (Invariante 7). Es gibt keine Historie und keine Herkunftsangabe.
    pub fn set_polished(&mut self, id: DiktatId, polished_text: &str) -> Result<Diktat> {
        self.conn.execute(
            "UPDATE diktat SET polished_text = ?1 WHERE id = ?2",
            params![polished_text, id],
        )?;
        self.get(id)
    }

    /// Nimmt einen Prompt in die Bibliothek auf.
    pub fn add_prompt(
        &mut self,
        alias: &str,
        kind: PromptKind,
        prompt_text: &str,
    ) -> Result<Prompt> {
        let inserted = self.conn.execute(
            "INSERT INTO \"prompt\" (alias, is_system, prompt_text) VALUES (?1, ?2, ?3)",
            params![alias, kind.as_int(), prompt_text],
        );
        match inserted {
            Ok(_) => self.prompt(self.conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(Error::AliasTaken {
                    alias: alias.to_string(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Liest einen Prompt.
    pub fn prompt(&self, id: PromptId) -> Result<Prompt> {
        self.conn
            .query_row(
                "SELECT id, alias, is_system, prompt_text FROM \"prompt\" WHERE id = ?1",
                [id],
                prompt_from_row,
            )
            .optional()?
            .ok_or(Error::PromptNotFound { id })
    }

    /// Die ganze Bibliothek, nach Alias sortiert.
    pub fn prompts(&self) -> Result<Vec<Prompt>> {
        let mut statement = self
            .conn
            .prepare("SELECT id, alias, is_system, prompt_text FROM \"prompt\" ORDER BY alias")?;
        let rows = statement.query_map([], prompt_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Ändert Alias, Art und Text eines Prompts.
    pub fn update_prompt(
        &mut self,
        id: PromptId,
        alias: &str,
        kind: PromptKind,
        prompt_text: &str,
    ) -> Result<Prompt> {
        let changed = self.conn.execute(
            "UPDATE \"prompt\" SET alias = ?1, is_system = ?2, prompt_text = ?3 WHERE id = ?4",
            params![alias, kind.as_int(), prompt_text, id],
        );
        match changed {
            Ok(0) => Err(Error::PromptNotFound { id }),
            Ok(_) => self.prompt(id),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(Error::AliasTaken {
                    alias: alias.to_string(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Löscht einen Prompt.
    pub fn delete_prompt(&mut self, id: PromptId) -> Result<()> {
        let removed = self
            .conn
            .execute("DELETE FROM \"prompt\" WHERE id = ?1", [id])?;
        if removed == 0 {
            return Err(Error::PromptNotFound { id });
        }
        Ok(())
    }
}

fn diktat_from_row(row: &Row<'_>) -> rusqlite::Result<Diktat> {
    let audio_dir: Option<String> = row.get("audio_dir")?;
    Ok(Diktat {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        audio_dir: audio_dir.map(PathBuf::from),
        raw_text: row.get("raw_text")?,
        polished_text: row.get("polished_text")?,
    })
}

fn prompt_from_row(row: &Row<'_>) -> rusqlite::Result<Prompt> {
    let is_system: i64 = row.get("is_system")?;
    Ok(Prompt {
        id: row.get("id")?,
        alias: row.get("alias")?,
        kind: PromptKind::from_int(is_system),
        prompt_text: row.get("prompt_text")?,
    })
}

/// `2026-11-21T09:31:00+01:00` — ISO-8601 in Ortszeit, sekundengenau.
fn now_local_iso8601() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

/// Der Pfad, wie er in der Datenbank steht.
fn path_for_db(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
