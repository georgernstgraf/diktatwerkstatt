//! Die Fragmentierung des laufenden Streams: Slice, Suchfenster, `NNN.wav`.
//!
//! Das **Slice-Modell** (aus `polished-recognition`, nicht das Vorwärtsfenster
//! aus `aitranscribe`): die Slice ist die **Obergrenze** eines Fragments, und
//! ein Schnitt darf nur in ihren **letzten 30 %** liegen — dem Suchfenster.
//! Entschieden wird also, sobald die ganze Slice aufgenommen ist; ein Fragment
//! ist damit zwischen `slice − search` und `slice` lang. Die Obergrenze darf
//! nicht überschritten werden, weil das stabile Whisper-Fenster 20–30 s ist.
//!
//! Drei Regeln tragen das Modul:
//!
//! - **Lückenlos**: Fragment `N + 1` beginnt genau bei `end_sample` von `N`.
//!   Die Verkettung der WAV-Nutzlasten ist die Aufnahme, byte-gleich.
//! - **Genau eine Datei je Fragment** (`NNN.wav` über
//!   [`WorkingDir::stage_audio`]); der Pre-roll ist kein Seitendatei-Produkt,
//!   er wird im Speicher an den Aufrufer gegeben.
//! - **Der Rest am Ende** wird als **ein** letztes Fragment ganz und ungesucht
//!   genommen: `quality = None`, `forced = true`, `cut = None`. Ihm folgt keine
//!   Naht.
//!
//! Ab Fragment 1000 bricht die Fragmentierung mit
//! [`Error::FragmentNumber`] ab; der Fehler ist „sticky" — jeder weitere
//! Aufruf liefert denselben Fehler, statt Metadaten zu verschlucken.

use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::audio::pcm::wav_bytes;
use crate::audio::seam::{self, QUALITY_MIN, SeamPolicy};
use crate::audio::silence::{self, Axis, Cut};
use crate::error::{Error, Result};
use crate::fragments::WorkingDir;

/// Anteil der Slice, den das Suchfenster einnimmt: die letzten 30 %.
///
/// Aus ihm ist [`Config::default`]s `search_ms` abgeleitet (30 % von 21 s =
/// 6,3 s), damit die Obergrenze des Fragments und das Fenster nicht
/// auseinanderlaufen können.
pub const DEFAULT_SEARCH_PERCENT: f64 = 30.0;

/// Die Geometrie und die Schwellen einer Fragmentierung.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    /// Abtastrate in Hz (Default 16 000).
    pub sample_rate: u32,
    /// Obergrenze eines Fragments in Millisekunden (Default 21 000).
    pub slice_ms: u32,
    /// Länge des Suchfensters am Ende der Slice in Millisekunden — die letzten
    /// [`DEFAULT_SEARCH_PERCENT`] Prozent (Default 6 300). Muss kleiner als
    /// [`Config::slice_ms`] sein.
    pub search_ms: u32,
    /// Mindestabstand des Schnitts zu den Achsenkanten in Millisekunden
    /// (Default 150).
    pub margin_ms: u32,
    /// Die Achse des Stillemaßes (Default [`Axis::default`]).
    pub axis: Axis,
    /// Schwelle des Q-Gates: `Q` darunter konditioniert die Naht (Default
    /// [`QUALITY_MIN`]).
    pub quality_min: f64,
    /// Wie die Naht konditioniert wird (Default [`SeamPolicy::QualityGate`]).
    pub policy: SeamPolicy,
}

impl Default for Config {
    fn default() -> Self {
        let slice_ms = 21_000;
        Self {
            sample_rate: 16_000,
            slice_ms,
            search_ms: (slice_ms as f64 * DEFAULT_SEARCH_PERCENT / 100.0) as u32,
            margin_ms: 150,
            axis: Axis::default(),
            quality_min: QUALITY_MIN,
            policy: SeamPolicy::QualityGate,
        }
    }
}

impl Config {
    /// Samples einer Slice.
    fn slice_samples(&self) -> u64 {
        samples_of(self.slice_ms, self.sample_rate)
    }

    /// Samples des Suchfensters.
    fn search_samples(&self) -> u64 {
        samples_of(self.search_ms, self.sample_rate)
    }
}

/// Millisekunden in Samples.
fn samples_of(milliseconds: u32, sample_rate: u32) -> u64 {
    u64::from(milliseconds) * u64::from(sample_rate) / 1_000
}

/// Ein geschriebenes Fragment mit der Evidenz seines Endschnitts.
#[derive(Debug, Clone, PartialEq)]
pub struct Committed {
    /// Die Fragmentnummer, `1` bis `999`.
    pub number: u32,
    /// Der Pfad der geschriebenen Datei, `<dir>/NNN.wav`.
    pub path: PathBuf,
    /// Die absolute Sample-Nummer des Fragmentanfangs.
    pub start_sample: u64,
    /// Die absolute Sample-Nummer hinter dem Fragment.
    pub end_sample: u64,
    /// `Q` des Endschnitts; `None` heißt nicht gemessen (der Rest).
    pub quality: Option<f64>,
    /// Der Endschnitt war erzwungen (kein Kandidat — oder der Rest).
    pub forced: bool,
    /// Wurde die Naht an **diesem** Anfang konditioniert?
    pub stitched: bool,
    /// Die Evidenz des Endschnitts; `None` beim Rest.
    pub cut: Option<Cut>,
}

/// Der Spool und der Schnittzustand einer laufenden Aufnahme.
///
/// Der Spool hält die Samples ab `base` (absolute Sample-Nummer von
/// `spool[0]`); nach jedem Schnitt wird bis zum Schnitt getrimmt, der Speicher
/// bleibt also bei etwa einer Slice. Das Arbeitsverzeichnis muss existieren
/// ([`WorkingDir::create`]) — der Preparer legt es nicht an.
pub struct Preparer {
    dir: WorkingDir,
    config: Config,
    spool: Vec<i16>,
    base: u64,
    start: u64,
    number: u32,
    previous_quality: Option<f64>,
    previous_forced: bool,
    has_previous: bool,
    finishing: bool,
    sticky: Option<Sticky>,
}

impl Preparer {
    /// Nimmt ein Arbeitsverzeichnis und die Geometrie.
    ///
    /// # Panics
    ///
    /// Im Debug-Build, wenn `search_ms >= slice_ms` oder die Abtastrate `0`
    /// ist — beides macht das Suchfenster leer.
    pub fn new(dir: WorkingDir, config: Config) -> Self {
        debug_assert!(
            config.search_ms < config.slice_ms,
            "search_ms ({}) muss kleiner als slice_ms ({}) sein",
            config.search_ms,
            config.slice_ms
        );
        debug_assert!(
            config.sample_rate > 0,
            "eine Abtastrate von 0 hat keinen Rahmen"
        );
        Self {
            dir,
            config,
            spool: Vec::new(),
            base: 0,
            start: 0,
            number: 1,
            previous_quality: None,
            previous_forced: true,
            has_previous: false,
            finishing: false,
            sticky: None,
        }
    }

    /// Die benutzte Geometrie.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Hängt aufgenommene Samples an. Nach [`Preparer::finish`] ein No-op.
    pub fn push(&mut self, samples: &[i16]) {
        if self.finishing {
            return;
        }
        self.spool.extend_from_slice(samples);
    }

    /// Die absolute Zahl aufgenommener Samples.
    pub fn pcm_size(&self) -> u64 {
        self.base + self.spool.len() as u64
    }

    /// Was noch im Spool liegt (eine Slice plus Rest).
    pub fn spool_samples(&self) -> usize {
        self.spool.len()
    }

    /// Committet **höchstens ein** Fragment, wenn eine volle Slice da ist.
    ///
    /// Nach einem Fehler ist der Aufruf „sticky": jeder weitere Aufruf liefert
    /// denselben Fehler, damit weder ein Schreibfehler noch die 999-Grenze
    /// bereits committete Metadaten verschluckt.
    pub fn commit_ready(&mut self) -> Result<Option<Committed>> {
        self.step(false)
    }

    /// Wie [`Preparer::commit_ready`], nimmt aber am Ende den Rest ganz mit.
    ///
    /// Ist keine volle Slice mehr da und liegen noch Samples offen, wird
    /// `[start, pcm_size())` als **ein** letztes Fragment geschrieben —
    /// ungesucht, `quality = None`, `forced = true`, `cut = None`. Danach
    /// liefert jeder weitere Aufruf `None`; mehrfaches `finish` ist erlaubt.
    pub fn finish(&mut self) -> Result<Option<Committed>> {
        self.finishing = true;
        self.step(true)
    }

    /// Ein Schritt mit Sticky-Behandlung.
    fn step(&mut self, tail: bool) -> Result<Option<Committed>> {
        if let Some(sticky) = &self.sticky {
            return Err(sticky.replay());
        }
        match self.next(tail) {
            Ok(committed) => Ok(committed),
            Err(error) => {
                self.sticky = Some(Sticky::capture(&error));
                Err(error)
            }
        }
    }

    /// Der eigentliche Schnitt: höchstens ein Fragment.
    fn next(&mut self, tail: bool) -> Result<Option<Committed>> {
        let slice_end = self.start + self.config.slice_samples();
        if self.pcm_size() < slice_end {
            if !tail {
                return Ok(None);
            }
            let total = self.pcm_size();
            if total <= self.start {
                return Ok(None);
            }
            return self.emit(self.start, total, None, true, None).map(Some);
        }

        // Das Suchfenster sind die letzten `search_ms` der Slice; `evaluate`
        // liefert den Schnitt relativ dazu.
        let window_start = slice_end - self.config.search_samples();
        let window = self.samples(window_start, slice_end);
        let decision = silence::evaluate(
            &window,
            self.config.sample_rate,
            self.config.axis,
            self.config.margin_ms,
        );
        let (end, quality, forced, cut) = match decision {
            Some(cut) => (
                window_start + cut.cut_sample as u64,
                Some(cut.quality),
                !cut.candidate,
                Some(cut),
            ),
            // Fenster unter einem Analyse-Rahmen: harter Schnitt am Slice-Ende.
            None => (slice_end, None, true, None),
        };
        self.emit(self.start, end, quality, forced, cut).map(Some)
    }

    /// Schreibt ein Fragment und führt den Zustand nach.
    fn emit(
        &mut self,
        start: u64,
        end: u64,
        quality: Option<f64>,
        forced: bool,
        cut: Option<Cut>,
    ) -> Result<Committed> {
        debug_assert!(end > start, "ein leeres Fragment wird nicht geschrieben");

        let number = self.number;
        let stitched = self.has_previous
            && seam::stitched(
                self.config.policy,
                self.previous_quality,
                self.previous_forced,
                self.config.quality_min,
            );
        let samples = self.samples(start, end);
        let path = self
            .dir
            .stage_audio(number, &wav_bytes(&samples, self.config.sample_rate))?;

        self.number = number + 1;
        self.start = end;
        self.previous_quality = quality;
        self.previous_forced = forced;
        self.has_previous = true;
        self.trim(end);

        Ok(Committed {
            number,
            path,
            start_sample: start,
            end_sample: end,
            quality,
            forced,
            stitched,
            cut,
        })
    }

    /// Die Samples `[start, end)` aus dem Spool.
    fn samples(&self, start: u64, end: u64) -> Vec<i16> {
        let from = (start - self.base) as usize;
        let to = (end - self.base) as usize;
        self.spool[from..to].to_vec()
    }

    /// Verwirft alles vor `up_to`.
    fn trim(&mut self, up_to: u64) {
        let discard = (up_to.saturating_sub(self.base)) as usize;
        if discard > 0 {
            self.spool.drain(..discard);
            self.base = up_to;
        }
    }
}

/// Ein wiederholbarer Fehler.
///
/// [`Error`] ist nicht `Clone` (er trägt `io::Error` und `rusqlite::Error`),
/// der Sticky-Zustand braucht aber genau das. Deshalb wird der Fehler in seine
/// wiederherstellbaren Bestandteile zerlegt und beim nächsten Aufruf neu
/// gebaut.
#[derive(Debug)]
enum Sticky {
    /// Die Fragmentnummer lag außerhalb von `001` bis `999`.
    Number(u32),
    /// Das Dateisystem hat abgelehnt — Art und Text reichen zum Wiederholen.
    Io(ErrorKind, String),
}

impl Sticky {
    /// Zerlegt einen Fehler in seine wiederholbare Form.
    fn capture(error: &Error) -> Self {
        match error {
            Error::FragmentNumber { number } => Self::Number(*number),
            Error::Io(io) => Self::Io(io.kind(), io.to_string()),
            other => Self::Io(ErrorKind::Other, other.to_string()),
        }
    }

    /// Baut denselben Fehler wieder.
    fn replay(&self) -> Error {
        match self {
            Self::Number(number) => Error::FragmentNumber { number: *number },
            Self::Io(kind, message) => Error::Io(std::io::Error::new(*kind, message.clone())),
        }
    }
}

/// Die Anweisungen an den Arbeitsfaden.
enum Command {
    /// Neue Samples.
    Push(Vec<i16>),
    /// Den Rest ganz mitnehmen und den Faden beenden.
    Finish,
    /// Ohne den Rest beenden (abgebrochene Aufnahme).
    Stop,
}

/// Der Arbeitsfaden neben der Aufnahme.
///
/// Audio geht über einen `mpsc`-Kanal an den Faden, fertige Fragmente über
/// einen zweiten zurück — kein `Arc<Mutex<…>>`. Der Faden pollt höchstens alle
/// `poll` und committet in einer Schleife, bis nichts mehr geht.
pub struct Worker {
    commands: Sender<Command>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl Worker {
    /// Spawnt den Arbeitsfaden; liefert die fertigen Fragmente über den
    /// Receiver.
    ///
    /// Der Receiver endet, wenn der Faden endet — nach [`Worker::finish`] sind
    /// also alle Fragmente samt Rest abgeholt.
    pub fn start(mut preparer: Preparer, poll: Duration) -> (Self, Receiver<Committed>) {
        let (commands, inbox) = mpsc::channel();
        let (outbox, committed) = mpsc::channel();
        let handle = thread::spawn(move || run(&mut preparer, &inbox, &outbox, poll));
        let worker = Self {
            commands,
            handle: Some(handle),
        };
        (worker, committed)
    }

    /// Reicht Samples an den Arbeitsfaden weiter.
    pub fn push(&self, samples: &[i16]) {
        let _ = self.commands.send(Command::Push(samples.to_vec()));
    }

    /// Nimmt den Rest ganz mit, wartet auf den Faden und gibt dessen Fehler
    /// zurück.
    pub fn finish(mut self) -> Result<()> {
        let _ = self.commands.send(Command::Finish);
        self.join()
    }

    /// Beendet ohne den Rest (abgebrochene Aufnahme).
    pub fn stop(mut self) -> Result<()> {
        let _ = self.commands.send(Command::Stop);
        self.join()
    }

    /// Wartet auf den Faden und reicht dessen Ergebnis durch.
    fn join(&mut self) -> Result<()> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        match handle.join() {
            Ok(result) => result,
            Err(_) => Err(Error::Io(std::io::Error::other(
                "der Arbeitsfaden ist abgestürzt",
            ))),
        }
    }
}

/// Die Schleife des Arbeitsfadens.
fn run(
    preparer: &mut Preparer,
    inbox: &Receiver<Command>,
    outbox: &Sender<Committed>,
    poll: Duration,
) -> Result<()> {
    loop {
        let command = match inbox.recv_timeout(poll) {
            Ok(command) => command,
            Err(RecvTimeoutError::Timeout) => {
                drain(preparer, outbox)?;
                continue;
            }
            // Ohne Abschluss verworfen: die Aufnahme ist abgebrochen.
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        match command {
            Command::Push(chunk) => {
                preparer.push(&chunk);
                drain(preparer, outbox)?;
            }
            Command::Finish => loop {
                match preparer.finish()? {
                    Some(committed) => {
                        let _ = outbox.send(committed);
                    }
                    None => return Ok(()),
                }
            },
            Command::Stop => return Ok(()),
        }
    }
}

/// Committet alles, was gerade entscheidbar ist.
fn drain(preparer: &mut Preparer, outbox: &Sender<Committed>) -> Result<()> {
    while let Some(committed) = preparer.commit_ready()? {
        let _ = outbox.send(committed);
    }
    Ok(())
}
