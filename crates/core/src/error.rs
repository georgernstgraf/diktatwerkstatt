//! Typisierte Fehler des Kerns.
//!
//! Der Kern panict nie an einer Stelle, an der ein Aufrufer etwas falsch machen
//! kann — eine offene Transkription, ein fehlendes Fragment, ein doppelter
//! Prompt-Alias sind Fehlerwerte (`docs/ai/DOMAIN.md`, Fehlerfälle).

use crate::store::{DiktatId, PromptId};

/// Ergebnis eines Kernvorgangs.
pub type Result<T> = std::result::Result<T, Error>;

/// Was im Kern schiefgehen kann.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Append auf ein Diktat, dessen Transkription noch läuft (Invariante 5).
    #[error("Diktat {id} ist noch offen: die Transkription läuft ({audio_dir})")]
    TranscriptionOpen {
        /// Das betroffene Diktat.
        id: DiktatId,
        /// Sein Arbeitsverzeichnis — der Beweis, dass es noch offen ist.
        audio_dir: String,
    },

    /// Ein Vorgang, der ein offenes Diktat braucht, traf ein abgeschlossenes.
    #[error("Diktat {id} ist bereits abgeschlossen: es hat kein Arbeitsverzeichnis mehr")]
    TranscriptionClosed {
        /// Das betroffene Diktat.
        id: DiktatId,
    },

    /// Das Diktat steht nicht in der Datenbank.
    #[error("Diktat {id} existiert nicht")]
    DiktatNotFound {
        /// Die gesuchte Kennung.
        id: DiktatId,
    },

    /// Der Prompt steht nicht in der Bibliothek.
    #[error("Prompt {id} existiert nicht")]
    PromptNotFound {
        /// Die gesuchte Kennung.
        id: PromptId,
    },

    /// Der Alias ist schon vergeben — `prompt.alias` ist `UNIQUE`.
    #[error("Prompt-Alias {alias:?} ist schon vergeben")]
    AliasTaken {
        /// Der abgelehnte Alias.
        alias: String,
    },

    /// Ein Fragment hat weder `.wav` noch `.txt`; die Verkettung wäre lückenhaft.
    #[error("Fragment {number:03} fehlt: es gibt weder eine .wav noch eine .txt")]
    FragmentMissing {
        /// Die fehlende Fragmentnummer.
        number: u32,
    },

    /// Die Fragmentnummer liegt außerhalb von `001` bis `999`.
    #[error("Fragmentnummer {number} liegt außerhalb von 001 bis 999")]
    FragmentNumber {
        /// Die abgelehnte Nummer.
        number: u32,
    },

    /// Die Datenbank stammt aus einer neueren Version der App.
    #[error("Schemaversion {found} der Datenbank ist neuer als die unterstützte {supported}")]
    SchemaTooNew {
        /// Was in `PRAGMA user_version` steht.
        found: i64,
        /// Was dieser Kern kennt.
        supported: i64,
    },

    /// SQLite hat den Vorgang abgelehnt.
    #[error("Datenbank: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Das Dateisystem hat den Vorgang abgelehnt.
    #[error("Dateisystem: {0}")]
    Io(#[from] std::io::Error),
}
