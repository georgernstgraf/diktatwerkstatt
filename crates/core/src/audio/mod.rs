//! Der hardwarefreie DSP-Kern: Rahmen, Stillemaß, Naht und Fragmentierung.
//!
//! Vier Bausteine, alle ohne Plattform und ohne Abhängigkeit außer `std`:
//!
//! - [`pcm`] — Hochpass, Rahmenpegel, Upload-Konditionierung und WAV.
//! - [`silence`] — das Stillemaß `Q` und der Schnitt im Suchfenster.
//! - [`seam`] — die Naht: Policies, Pre-roll, Prompt, Echo-Trim, Überlappung.
//! - [`fragmenter`] — Slice, Spool, `NNN.wav` und der Arbeitsfaden.
//!
//! Alles rechnet auf `&[i16]` (16 kHz mono): keine Audio-Hardware,
//! deterministisch prüfbar.

pub mod fragmenter;
pub mod pcm;
pub mod seam;
pub mod silence;

pub use fragmenter::{Committed, Config, DEFAULT_SEARCH_PERCENT, Preparer, Worker};
pub use pcm::{FRAME_MS, condition, frame_levels, high_pass, wav_bytes, wav_samples};
pub use seam::{
    ECHO_MAX_TOKENS, PRE_ROLL_MS, PROMPT_MAX_CHARS, QUALITY_MIN, SeamPolicy, overlap, pre_roll,
    prompt, stitched, trim_echo, upload,
};
pub use silence::{Axis, Cut, evaluate};
