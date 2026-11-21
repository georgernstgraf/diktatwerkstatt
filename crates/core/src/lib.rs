//! Domäne der Diktatwerkstatt: Diktate, Prompts, Arbeitsverzeichnis und FIFO.
//!
//! Der Kern kennt **keine** Plattform: kein `cfg(target_os)`, kein
//! `enum Platform`, kein Netz, keine Zwischenablage, kein Fenster. Er ist mit
//! `cargo test` prüfbar, ohne dass die App läuft und ohne Hardware.
//!
//! Vier Bausteine teilen sich die Arbeit:
//!
//! - [`Store`] — die SQLite-Datei mit den zwei Tabellen `diktat` und `prompt`
//!   (`docs/ai/DOMAIN.md`, „`diktat` — das Diktat" und „`prompt` — die
//!   Bibliothek").
//! - [`WorkingDir`] — das Verzeichnis eines offenen Diktats mit je Fragment
//!   `NNN.wav` (offen) oder `NNN.txt` (erledigt).
//! - [`evaluate`] — das Stillemaß `Q` und die Schnittentscheidung; [`condition`]
//!   der Weg zum STT-Endpunkt, [`wav_bytes`] die Datei eines Fragments.
//! - [`deliver`] — constrained Markdown → [`Delivery`]: das semantische
//!   HTML-Fragment und der reine Text für die Zwischenablage. Ohne
//!   `CF_HTML`-Umschlag — den baut der Windows-Schreiber.
//!
//! # Die zehn Invarianten
//!
//! Die Invarianten stehen in `docs/ai/DOMAIN.md`; hier steht, wo sie
//! durchgesetzt werden und welcher Test sie bricht. Die Testnamen stehen in
//! `crates/core/tests/invarianten.rs`.
//!
//! | # | Invariante | durchgesetzt in | Test |
//! |---|---|---|---|
//! | 1 | `audio_dir` gesetzt ⇔ Transkription nicht abgeschlossen | [`Diktat::is_open`], Schema ohne zweites Zustandsfeld | `audio_dir_is_the_only_state_marker` |
//! | 2 | Text vor Audio auf Fragmentebene | [`WorkingDir::transcribe`] (Schreiben über `.tmp` + `rename`, dann Löschen) | `text_before_audio_on_fragment_level` |
//! | 3 | Text vor Audio auf Diktatebene | [`Store::finish_transcription`] (`commit_raw_text`, dann `remove_working_dir`) | `text_before_audio_on_dictation_level` |
//! | 4 | `raw_text` ist die einzige Speicherung des rohen Transkripts | Schema, [`Store::commit_raw_text`] | `raw_text_is_one_text` |
//! | 5 | Append nur bei abgeschlossener Transkription | [`Store::append_raw`] → [`Error::TranscriptionOpen`] | `append_requires_finished_transcription` |
//! | 6 | Jede Änderung am Rohtext verwirft das Polierte | [`Store::append_raw`], [`Store::correct_raw`], [`Store::commit_raw_text`] | `raw_edit_discards_polished` |
//! | 7 | Ein poliertes Transkript pro Diktat | [`Store::set_polished`] (ein Slot, vollständiges Überschreiben) | `one_polished_transcript` |
//! | 8 | Geliefert wird genau der Text, der im Fenster steht | [`Diktat::view_text`] als einziger Leseweg; jede Mutation gibt den neuen Fensterstand zurück | `delivered_text_is_the_visible_text` |
//! | 9 | FIFO hält höchstens N Diktate, das Arbeitsverzeichnis fällt mit | [`Store::begin_dictation`] | `fifo_holds_at_most_n_dictations` |
//! | 10 | Das rohe Transkript wird nie umformatiert | keine Normalisierung in [`Store::commit_raw_text`] und [`WorkingDir::assemble`] | `raw_text_is_never_reformatted` |
//!
//! [`Error::TranscriptionOpen`]: crate::Error::TranscriptionOpen

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod audio;
mod delivery;
mod error;
mod fragments;
mod store;

pub use audio::{
    Axis, Committed, Config, Cut, DEFAULT_SEARCH_PERCENT, ECHO_MAX_TOKENS, FRAME_MS, PRE_ROLL_MS,
    PROMPT_MAX_CHARS, Preparer, QUALITY_MIN, SeamPolicy, Worker, condition, evaluate, frame_levels,
    high_pass, overlap, pre_roll, prompt, stitched, trim_echo, upload, wav_bytes, wav_samples,
};
pub use delivery::{Construct, Delivery, DeliveryError, deliver};
pub use error::{Error, Result};
pub use fragments::{MAX_FRAGMENT, WorkingDir};
pub use store::{Diktat, DiktatId, Prompt, PromptId, PromptKind, Resume, Store, View};
