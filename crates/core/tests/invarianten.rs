//! Ein Test je Invariante aus `docs/ai/DOMAIN.md`, „Die zehn Invarianten".
//!
//! Jeder dieser Tests bricht, wenn man die Invariante aus dem Code entfernt —
//! das ist das Abnahmekriterium von #4. Dazu die Fälle, die im Issue daneben
//! stehen: die halb geschriebene `.txt`, die leere `.txt` als gültiges
//! Ergebnis, das Arbeitsverzeichnis im FIFO und die Obergrenze von 999
//! Fragmenten.

use std::fs;
use std::path::{Path, PathBuf};

use diktatwerkstatt_core::{Diktat, Error, MAX_FRAGMENT, PromptKind, Store, View, WorkingDir};
use rusqlite::Connection;
use tempfile::TempDir;

fn database(dir: &TempDir) -> PathBuf {
    dir.path().join("diktatwerkstatt.db")
}

fn store(dir: &TempDir) -> Store {
    Store::open(database(dir)).expect("Datenbank öffnen")
}

fn audio_root(dir: &TempDir) -> PathBuf {
    dir.path().join("audio")
}

fn working_dir(diktat: &Diktat) -> WorkingDir {
    WorkingDir::new(diktat.audio_dir.clone().expect("Arbeitsverzeichnis"))
}

fn columns(db: &Path, table: &str) -> Vec<String> {
    let conn = Connection::open(db).expect("zweite Verbindung");
    let mut statement = conn
        .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
        .expect("Spalten abfragen");
    let rows = statement
        .query_map([table], |row| row.get::<_, String>(0))
        .expect("Spalten lesen");
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .expect("Spalten sammeln")
}

fn tables(db: &Path) -> Vec<String> {
    let conn = Connection::open(db).expect("zweite Verbindung");
    let mut statement = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .expect("Tabellen abfragen");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("Tabellen lesen");
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .expect("Tabellen sammeln")
}

// ---------------------------------------------------------------------------
// Invariante 1
// ---------------------------------------------------------------------------

#[test]
fn audio_dir_is_the_only_state_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);

    let open = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    assert!(open.is_open(), "ein neues Diktat ist offen");
    assert_eq!(open.raw_text, "", "roh beginnt leer");
    assert_eq!(
        open.audio_dir,
        Some(audio_root(&tmp).join(open.id.to_string())),
        "audio/<id>/ ist der Pfad aus DOMAIN.md"
    );
    assert!(open.audio_dir.as_ref().unwrap().is_dir());

    let finished = store.finish_transcription(open.id, "Fertig.").unwrap();
    assert!(!finished.is_open());
    assert_eq!(finished.audio_dir, None);

    // Kein zweites Zustandsfeld: die Tabelle kennt genau diese fünf Spalten.
    assert_eq!(
        columns(&database(&tmp), "diktat"),
        ["id", "created_at", "audio_dir", "raw_text", "polished_text"],
        "es gibt kein status, kein closed_at, kein pinned"
    );
}

// ---------------------------------------------------------------------------
// Invariante 2
// ---------------------------------------------------------------------------

#[test]
fn text_before_audio_on_fragment_level() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = WorkingDir::new(tmp.path().join("7"));
    dir.create().unwrap();
    dir.stage_audio(1, b"RIFF-erfunden").unwrap();
    assert_eq!(dir.pending().unwrap(), vec![1], "nur .wav heißt offen");

    // Der Abbruch: die .txt ist umbenannt, die .wav liegt noch.
    dir.stage_transcript(1, "Hallo Naht").unwrap();
    assert!(
        dir.audio_path(1).unwrap().exists(),
        "das Audio ist noch da — genau der Zustand nach einem Absturz"
    );
    assert!(
        dir.pending().unwrap().is_empty(),
        "die .txt gewinnt: das Fragment gilt als erledigt"
    );
    assert_eq!(dir.done().unwrap(), vec![1]);
    assert_eq!(dir.assemble().unwrap(), "Hallo Naht");

    // Und der reguläre Weg räumt das Audio hinterher ab.
    dir.transcribe(2, "Zweiter Satz").unwrap();
    assert!(!dir.audio_path(2).unwrap().exists());
    assert_eq!(dir.done().unwrap(), vec![1, 2]);
}

#[test]
fn half_written_transcript_does_not_count() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = WorkingDir::new(tmp.path().join("8"));
    dir.create().unwrap();
    dir.stage_audio(1, b"RIFF-erfunden").unwrap();

    // Abbruch mitten im Schreiben: nur die Zwischendatei liegt da.
    fs::write(dir.root().join("001.txt.tmp"), "halb geschrieben").unwrap();

    assert_eq!(
        dir.pending().unwrap(),
        vec![1],
        "eine .tmp-Datei ist kein Transkript"
    );
    assert!(dir.done().unwrap().is_empty());
    assert!(
        matches!(dir.assemble(), Err(Error::FragmentMissing { number: 1 })),
        "eine lückenhafte Verkettung wird verweigert"
    );
}

#[test]
fn empty_transcript_counts_as_done() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = WorkingDir::new(tmp.path().join("9"));
    dir.create().unwrap();
    dir.stage_audio(1, b"RIFF-Stille").unwrap();
    dir.transcribe(1, "").unwrap();

    assert_eq!(
        dir.done().unwrap(),
        vec![1],
        "leer ist gültig: das war Stille"
    );
    assert!(dir.pending().unwrap().is_empty());
    assert_eq!(
        dir.assemble().unwrap(),
        "",
        "nichts zu transkribieren, nichts neu"
    );
}

#[test]
fn fragment_number_above_the_limit_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = WorkingDir::new(tmp.path().join("10"));
    dir.create().unwrap();

    dir.stage_audio(MAX_FRAGMENT, b"letztes").unwrap();
    assert!(matches!(
        dir.stage_audio(MAX_FRAGMENT + 1, b"zuviel"),
        Err(Error::FragmentNumber { number }) if number == MAX_FRAGMENT + 1
    ));
    assert!(matches!(
        dir.stage_transcript(0, "null"),
        Err(Error::FragmentNumber { number: 0 })
    ));
}

// ---------------------------------------------------------------------------
// Invariante 3
// ---------------------------------------------------------------------------

#[test]
fn text_before_audio_on_dictation_level() {
    let tmp = tempfile::tempdir().unwrap();
    let db = database(&tmp);

    let id = {
        let mut store = Store::open(&db).unwrap();
        let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
        let dir = working_dir(&diktat);
        dir.stage_audio(1, b"a").unwrap();
        dir.stage_audio(2, b"b").unwrap();
        dir.transcribe(1, "Guten ").unwrap();
        dir.transcribe(2, "Tag.").unwrap();

        // Schritt 1 des Abschlusses — und dann stürzt die App ab.
        store.commit_raw_text(diktat.id, "Guten Tag.").unwrap();
        diktat.id
    };

    // Beim nächsten Start: der Text ist da, das Audio auch, alles wiederaufnehmbar.
    let mut store = Store::open(&db).unwrap();
    let diktat = store.get(id).unwrap();
    assert!(diktat.is_open(), "der Abschluss ist nicht passiert");
    assert_eq!(
        diktat.raw_text, "Guten Tag.",
        "der Text war vor dem Audio da"
    );
    assert!(diktat.audio_dir.as_ref().unwrap().is_dir());

    let resume = store.resume(id).unwrap();
    assert!(
        resume.pending_fragments.is_empty(),
        "nichts mehr zu transkribieren"
    );
    assert_eq!(resume.assembled_text.as_deref(), Some("Guten Tag."));

    // Der Abschluss läuft weiter und ist idempotent.
    let finished = store
        .finish_transcription(id, resume.assembled_text.as_deref().unwrap())
        .unwrap();
    assert_eq!(finished.audio_dir, None);
    assert_eq!(finished.raw_text, "Guten Tag.");
    assert!(!audio_root(&tmp).join(id.to_string()).exists());

    // Und der umgekehrte Abbruch — Verzeichnis weg, audio_dir noch gesetzt —
    // liefert denselben Text statt eines leeren Ersatzes.
    let mut store2 = Store::open(&db).unwrap();
    let second = store2.begin_dictation(&audio_root(&tmp), 10).unwrap();
    store2
        .commit_raw_text(second.id, "Schon committet.")
        .unwrap();
    fs::remove_dir_all(second.audio_dir.as_ref().unwrap()).unwrap();
    let resume = store2.resume(second.id).unwrap();
    assert_eq!(
        resume.assembled_text.as_deref(),
        Some("Schon committet."),
        "Invariante 3 hat den Text vorher geschrieben"
    );
}

// ---------------------------------------------------------------------------
// Invariante 4
// ---------------------------------------------------------------------------

#[test]
fn raw_text_is_one_text() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    let dir = working_dir(&diktat);

    for number in 1..=5 {
        dir.stage_audio(number, b"x").unwrap();
        dir.transcribe(number, &format!("Satz {number}. ")).unwrap();
    }
    let raw = dir.assemble().unwrap();
    store.finish_transcription(diktat.id, &raw).unwrap();

    // Ein Diktat, ein Text: keine Zeile je Block, keine zweite Tabelle.
    assert_eq!(
        tables(&database(&tmp)),
        ["diktat", "prompt"],
        "zwei Tabellen, nicht mehr"
    );
    assert_eq!(
        store.list(10).unwrap().len(),
        1,
        "fünf Fragmente sind ein Diktat"
    );
    assert_eq!(store.get(diktat.id).unwrap().raw_text, raw);
    assert_eq!(
        raw, "Satz 1. Satz 2. Satz 3. Satz 4. Satz 5. ",
        "verkettet in Fragmentreihenfolge"
    );
}

// ---------------------------------------------------------------------------
// Invariante 5
// ---------------------------------------------------------------------------

#[test]
fn append_requires_finished_transcription() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();

    match store.append_raw(diktat.id, "zu früh") {
        Err(Error::TranscriptionOpen { id, audio_dir }) => {
            assert_eq!(id, diktat.id);
            assert!(audio_dir.contains(&diktat.id.to_string()));
        }
        other => panic!("offenes Diktat muss Append verweigern, war: {other:?}"),
    }
    assert_eq!(
        store.get(diktat.id).unwrap().raw_text,
        "",
        "der verweigerte Append hat nichts geschrieben"
    );

    store.finish_transcription(diktat.id, "Text").unwrap();
    let appended = store.append_raw(diktat.id, " mehr").unwrap();
    assert_eq!(appended.raw_text, "Text mehr");
}

// ---------------------------------------------------------------------------
// Invariante 6
// ---------------------------------------------------------------------------

#[test]
fn raw_edit_discards_polished() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    store.finish_transcription(diktat.id, "Rohtext").unwrap();

    // Append verwirft.
    store.set_polished(diktat.id, "Poliert").unwrap();
    let appended = store.append_raw(diktat.id, " mehr").unwrap();
    assert_eq!(appended.polished_text, None);

    // Korrektur verwirft.
    store.set_polished(diktat.id, "Wieder poliert").unwrap();
    let corrected = store.correct_raw(diktat.id, "Korrigiert").unwrap();
    assert_eq!(corrected.raw_text, "Korrigiert");
    assert_eq!(corrected.polished_text, None);

    // Keine automatische Neupolitur: der Slot bleibt leer.
    assert_eq!(store.get(diktat.id).unwrap().polished_text, None);
    assert_eq!(corrected.view_text(View::Polished), "");

    // Auch der Abschluss selbst ändert das rohe Transkript und verwirft deshalb
    // das polierte — im Ablauf gibt es diesen Fall nicht, die Regel gilt trotzdem.
    let third = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    store.set_polished(third.id, "vorzeitig poliert").unwrap();
    let committed = store
        .commit_raw_text(third.id, "Roh nach Abschluss")
        .unwrap();
    assert_eq!(committed.polished_text, None);
}

// ---------------------------------------------------------------------------
// Invariante 7
// ---------------------------------------------------------------------------

#[test]
fn one_polished_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    store.finish_transcription(diktat.id, "Rohtext").unwrap();

    store.set_polished(diktat.id, "erste Fassung").unwrap();
    let second = store.set_polished(diktat.id, "zweite Fassung").unwrap();
    assert_eq!(second.polished_text.as_deref(), Some("zweite Fassung"));

    // Genau ein Slot, keine Historie: keine Tabelle, keine Spalte mehr.
    assert_eq!(tables(&database(&tmp)), ["diktat", "prompt"]);
    assert_eq!(
        columns(&database(&tmp), "diktat"),
        ["id", "created_at", "audio_dir", "raw_text", "polished_text"]
    );
}

// ---------------------------------------------------------------------------
// Invariante 8
// ---------------------------------------------------------------------------

#[test]
fn delivered_text_is_the_visible_text() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();

    // Eine Zwischenablage, die nichts tut, außer festzuhalten, was die Schale
    // schreiben würde. Vier Auslöser, ein Leseweg.
    let mut clipboard: String;

    // 1. Abschluss → roher Reiter.
    let finished = store.finish_transcription(diktat.id, "Roh.").unwrap();
    clipboard = finished.view_text(View::Raw).to_string();
    assert_eq!(clipboard, "Roh.");

    // 2. Prompt anwenden → polierter Reiter.
    let polished = store.set_polished(diktat.id, "Poliert.").unwrap();
    clipboard = polished.view_text(View::Polished).to_string();
    assert_eq!(clipboard, "Poliert.");

    // 3. Verlassen einer bearbeiteten Ansicht → genau der bearbeitete Reiter.
    let corrected = store.correct_raw(diktat.id, "Roh, bearbeitet.").unwrap();
    clipboard = corrected.view_text(View::Raw).to_string();
    assert_eq!(clipboard, "Roh, bearbeitet.");
    assert_eq!(
        corrected.view_text(View::Polished),
        "",
        "der polierte Reiter ist leer, nicht veraltet"
    );

    // 4. Knopf „In die Zwischenablage" → der sichtbare Reiter, ohne Bearbeitung.
    let reloaded = store.get(diktat.id).unwrap();
    assert_eq!(reloaded.view_text(View::Raw), clipboard);
    assert_eq!(reloaded, corrected, "es gibt keinen zweiten Fensterstand");
}

// ---------------------------------------------------------------------------
// Invariante 9
// ---------------------------------------------------------------------------

#[test]
fn fifo_holds_at_most_n_dictations() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);

    let mut dictations = Vec::new();
    for _ in 0..4 {
        dictations.push(store.begin_dictation(&audio_root(&tmp), 3).unwrap());
    }

    let oldest = &dictations[0];
    assert!(matches!(
        store.get(oldest.id),
        Err(Error::DiktatNotFound { id }) if id == oldest.id
    ));
    assert!(
        !oldest.audio_dir.as_ref().unwrap().exists(),
        "das Arbeitsverzeichnis fällt mit dem Diktat"
    );

    let remaining = store.list(10).unwrap();
    assert_eq!(remaining.len(), 3);
    assert_eq!(
        remaining.iter().map(|diktat| diktat.id).collect::<Vec<_>>(),
        vec![dictations[3].id, dictations[2].id, dictations[1].id]
    );
    assert!(dictations[3].audio_dir.as_ref().unwrap().is_dir());
    assert!(!columns(&database(&tmp), "diktat").contains(&"pinned".to_string()));
}

// ---------------------------------------------------------------------------
// Invariante 10
// ---------------------------------------------------------------------------

#[test]
fn raw_text_is_never_reformatted() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);

    let raw =
        "Erste Zeile\tmit Tab  und  doppelten Leerzeichen\n\nZweite Zeile — mit Gedankenstrich 😀 ";
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    let finished = store.finish_transcription(diktat.id, raw).unwrap();
    assert_eq!(finished.raw_text, raw);
    assert_eq!(store.get(diktat.id).unwrap().raw_text, raw);

    // Auch die Verkettung formatiert nichts: kein Trenner, kein Umbruch.
    let second = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    let dir = working_dir(&second);
    dir.transcribe(1, "Guten ").unwrap();
    dir.transcribe(2, "Tag.").unwrap();
    assert_eq!(dir.assemble().unwrap(), "Guten Tag.");
}

// ---------------------------------------------------------------------------
// Wiederaufnahme und Bibliothek
// ---------------------------------------------------------------------------

#[test]
fn resume_names_the_open_fragments() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);
    let diktat = store.begin_dictation(&audio_root(&tmp), 10).unwrap();
    let dir = working_dir(&diktat);

    dir.stage_audio(1, b"a").unwrap();
    dir.stage_audio(2, b"b").unwrap();
    dir.transcribe(1, "Erster").unwrap();

    let resume = store.resume(diktat.id).unwrap();
    assert_eq!(resume.pending_fragments, vec![2]);
    assert_eq!(resume.assembled_text, None);
    assert_eq!(store.open_dictations().unwrap(), vec![resume.diktat]);
}

#[test]
fn prompt_library_keeps_aliases_unique() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = store(&tmp);

    let system = store
        .add_prompt("System: sachlich", PromptKind::System, "Du bist Lektor.")
        .unwrap();
    let user = store
        .add_prompt("DE-sachlich", PromptKind::User, "Poliere: {{text}}")
        .unwrap();
    assert_eq!(system.kind, PromptKind::System);
    assert_eq!(user.kind, PromptKind::User);

    assert!(matches!(
        store.add_prompt("DE-sachlich", PromptKind::User, "noch einmal"),
        Err(Error::AliasTaken { alias }) if alias == "DE-sachlich"
    ));

    let renamed = store
        .update_prompt(user.id, "DE-knapp", PromptKind::User, "Kürze: {{text}}")
        .unwrap();
    assert_eq!(renamed.alias, "DE-knapp");
    assert_eq!(store.prompts().unwrap().len(), 2);

    store.delete_prompt(renamed.id).unwrap();
    assert_eq!(store.prompts().unwrap(), vec![system]);
}
