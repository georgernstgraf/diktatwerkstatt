//! Das Arbeitsverzeichnis eines offenen Diktats: `NNN.wav` / `NNN.txt`.
//!
//! Drei Ziffern, nullgefüllt, damit lexikografische und numerische Ordnung
//! dasselbe sind. Die Mischung ist kein Fehler, sondern die Anzeige: nur `.wav`
//! heißt offen, `.txt` heißt erledigt (`docs/ai/DOMAIN.md`, „Das
//! Arbeitsverzeichnis").
//!
//! Zwei Regeln tragen das ganze Modul:
//!
//! - **Text vor Audio** (Invariante 2): [`WorkingDir::transcribe`] schreibt erst
//!   `NNN.txt` (über `NNN.txt.tmp` + `rename`) und löscht **danach** `NNN.wav`.
//!   Wer nach einem Abbruch beides vorfindet, wertet die `.txt`: sie ist der
//!   Beweis, dass transkribiert wurde.
//! - **Nie umformatieren** (Invariante 10): der Text wird byte-gleich
//!   geschrieben und byte-gleich verkettet — kein Trenner, kein abschließender
//!   Umbruch.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Höchste Fragmentnummer.
///
/// Ab 1000 Brüchen wird die Transkription mit einem Fehler abgebrochen, statt
/// falsch zu sortieren: drei Ziffern sind der Sortierschlüssel.
pub const MAX_FRAGMENT: u32 = 999;

/// Ein Arbeitsverzeichnis — das Verzeichnis eines Diktats, solange dessen
/// Transkription läuft.
///
/// Die Struktur hält nur den Pfad; sie legt nichts an, solange [`create`] nicht
/// gerufen wurde.
///
/// [`create`]: WorkingDir::create
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingDir {
    root: PathBuf,
}

impl WorkingDir {
    /// Nimmt einen Pfad als Arbeitsverzeichnis. Das Verzeichnis darf noch fehlen.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Der Pfad des Verzeichnisses.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Liegt das Verzeichnis auf der Platte?
    pub fn exists(&self) -> bool {
        self.root.is_dir()
    }

    /// Legt das Verzeichnis an (samt Elternverzeichnissen).
    pub fn create(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        Ok(())
    }

    /// Entfernt das Verzeichnis samt Inhalt. Ein schon fehlendes Verzeichnis ist
    /// kein Fehler — der Abschluss darf wiederholt werden.
    pub fn remove(&self) -> Result<()> {
        match fs::remove_dir_all(&self.root) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Der Pfad von `NNN.wav`.
    pub fn audio_path(&self, number: u32) -> Result<PathBuf> {
        self.numbered(number, "wav")
    }

    /// Der Pfad von `NNN.txt`.
    pub fn transcript_path(&self, number: u32) -> Result<PathBuf> {
        self.numbered(number, "txt")
    }

    /// Legt `NNN.wav` ab — über `NNN.wav.tmp` und `rename`, damit eine halb
    /// geschriebene Datei nie als Fragment zählt.
    pub fn stage_audio(&self, number: u32, bytes: &[u8]) -> Result<PathBuf> {
        let path = self.audio_path(number)?;
        let temporary = with_extra_extension(&path, "tmp");
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &path)?;
        Ok(path)
    }

    /// Schreibt `NNN.txt` — über `NNN.txt.tmp` und `rename`, **ohne** `NNN.wav`
    /// zu löschen.
    ///
    /// Das ist der erste Halbsatz von Invariante 2; der zweite ist
    /// [`drop_audio`]. Beide zusammen sind [`transcribe`]. Der getrennte Weg
    /// existiert, damit Tests den Abbruch zwischen den beiden Schritten
    /// nachstellen können — und weil er der ehrliche Zustand nach einem
    /// Absturz ist.
    ///
    /// [`drop_audio`]: WorkingDir::drop_audio
    /// [`transcribe`]: WorkingDir::transcribe
    pub fn stage_transcript(&self, number: u32, text: &str) -> Result<()> {
        let path = self.transcript_path(number)?;
        let temporary = with_extra_extension(&path, "tmp");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, &path)?;
        Ok(())
    }

    /// Löscht `NNN.wav`. Fehlt sie schon, ist das kein Fehler.
    pub fn drop_audio(&self, number: u32) -> Result<()> {
        let path = self.audio_path(number)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Invariante 2 in einem Weg: erst das Transkript, dann das Audio.
    pub fn transcribe(&self, number: u32, text: &str) -> Result<()> {
        self.stage_transcript(number, text)?;
        self.drop_audio(number)
    }

    /// Nummern mit `.wav` und **ohne** `.txt` — nur sie müssen noch transkribiert
    /// werden.
    ///
    /// Liegt beides da (Abbruch zwischen `rename` und Löschen), gewinnt die
    /// `.txt`: das Fragment gilt als erledigt.
    pub fn pending(&self) -> Result<Vec<u32>> {
        let audio = self.numbers("wav")?;
        let done = self.numbers("txt")?;
        Ok(audio.difference(&done).copied().collect())
    }

    /// Nummern mit `.txt`. Eine **leere** `.txt` ist gültig: das Fragment war
    /// Stille. `NNN.txt.tmp` zählt nicht.
    pub fn done(&self) -> Result<Vec<u32>> {
        Ok(self.numbers("txt")?.into_iter().collect())
    }

    /// Verkettet alle `.txt` in Fragmentreihenfolge — ohne Trenner und ohne
    /// abschließenden Umbruch (Invarianten 2 und 10).
    ///
    /// Solange ein Fragment offen ist, gibt es keine gültige Verkettung; dann
    /// nennt der Fehler die erste Lücke.
    pub fn assemble(&self) -> Result<String> {
        if let Some(&number) = self.pending()?.first() {
            return Err(Error::FragmentMissing { number });
        }

        let done = self.numbers("txt")?;
        let mut text = String::new();
        for number in 1..=done.iter().next_back().copied().unwrap_or(0) {
            if !done.contains(&number) {
                return Err(Error::FragmentMissing { number });
            }
            text.push_str(&fs::read_to_string(self.transcript_path(number)?)?);
        }
        Ok(text)
    }

    fn numbered(&self, number: u32, extension: &str) -> Result<PathBuf> {
        if number == 0 || number > MAX_FRAGMENT {
            return Err(Error::FragmentNumber { number });
        }
        Ok(self.root.join(format!("{number:03}.{extension}")))
    }

    /// Alle Fragmentnummern, die eine Datei mit genau dieser Endung haben.
    ///
    /// Erkannt wird nur `NNN.<endung>` mit genau drei Ziffern; alles andere
    /// (`.tmp`-Reste, fremde Dateien) bleibt unsichtbar.
    fn numbers(&self, extension: &str) -> Result<BTreeSet<u32>> {
        let mut numbers = BTreeSet::new();
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(numbers),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if let Some(number) = fragment_number(name, extension) {
                numbers.insert(number);
            }
        }
        Ok(numbers)
    }
}

/// Liest `NNN.<extension>` und gibt `N` zurück.
fn fragment_number(name: &str, extension: &str) -> Option<u32> {
    let (stem, found) = name.rsplit_once('.')?;
    if found != extension || stem.len() != 3 || !stem.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    stem.parse().ok()
}

/// `NNN.wav` → `NNN.wav.tmp`, damit die Zwischendatei an keiner Stelle
/// versehentlich als Fragment durchgeht.
fn with_extra_extension(path: &Path, extra: &str) -> PathBuf {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.with_extension(extra);
    };
    path.with_file_name(format!("{name}.{extra}"))
}
