//! Portierte Fragmentierungs-Tests: Slice, Suchfenster, Naht, Rest, Worker.
//!
//! Aus `aitranscribe/tests/test_fragmenter.py` und den portierbaren Fällen von
//! `FragmentPreparerTest.kt` (Flüsterregime, Suchfenster, Policies). Die
//! Nummern in den Testnamen sind die Fälle der Testmatrix aus der
//! Spezifikation zu #5.
//!
//! Nicht portiert: OGG/Transcoder, Manifest, WAV-Fallback, Chunk-Assembly,
//! `prune`, Room-Latenz und die Pre-roll-Seitendateien — Android- bzw.
//! `polished-recognition`-Infrastruktur, die es in der Diktatwerkstatt nicht
//! gibt. Der Pre-roll ist hier ein Speicherbereich, keine Datei; deshalb
//! entfällt auch `test_worker_error_is_raised_by_stop` (Python-`monkeypatch`) —
//! ersetzt durch Fall 16, einen echten Schreibfehler.

use std::fs;
use std::thread;
use std::time::Duration;

use diktatwerkstatt_core::{
    Axis, Committed, Config, Error, Preparer, QUALITY_MIN, SeamPolicy, Worker, WorkingDir,
    wav_samples,
};
use tempfile::TempDir;

/// Die Abtastrate aller Tests.
const SAMPLE_RATE: u32 = 16_000;

/// Samples eines 10-ms-Rahmens bei [`SAMPLE_RATE`].
const FRAME_SAMPLES: usize = 160;

// ---------------------------------------------------------------------------
// Testmaterial
// ---------------------------------------------------------------------------

/// Ein Sinus als Samples (`np.sin(...).astype(np.int16)` schneidet gegen Null ab).
fn speech(seconds: f64, amplitude: f64, frequency: f64) -> Vec<i16> {
    let count = (seconds * f64::from(SAMPLE_RATE)) as usize;
    (0..count)
        .map(|index| {
            let time = index as f64 / f64::from(SAMPLE_RATE);
            (amplitude * (2.0 * std::f64::consts::PI * frequency * time).sin()).trunc() as i16
        })
        .collect()
}

/// Eine Zeit in Sekunden als Sample-Nummer.
fn at(seconds: f64) -> u64 {
    (seconds * f64::from(SAMPLE_RATE)) as u64
}

/// Stille als Samples.
fn silence(seconds: f64) -> Vec<i16> {
    vec![0i16; (seconds * f64::from(SAMPLE_RATE)) as usize]
}

/// `repeats` mal Sprache, dann Stille.
fn burst_train(pause_seconds: f64, speech_seconds: f64, repeats: usize) -> Vec<i16> {
    let mut audio = Vec::new();
    for _ in 0..repeats {
        audio.extend(speech(speech_seconds, 6000.0, 180.0));
        audio.extend(silence(pause_seconds));
    }
    audio
}

/// `n` volle Rahmen Nyquist-Rechteck (Sample `k` abwechselnd `±amp`).
fn frame(n: usize, amp: i16) -> Vec<i16> {
    (0..n * FRAME_SAMPLES)
        .map(|index| if index % 2 == 0 { amp } else { -amp })
        .collect()
}

/// Ein enges Flüsterband: 5-Rahmen-Blöcke abwechselnd 380 und 420.
fn whisper_band(blocks: usize) -> Vec<i16> {
    let mut audio = Vec::new();
    for block in 0..blocks {
        audio.extend(frame(5, if block % 2 == 0 { 380 } else { 420 }));
    }
    audio
}

// ---------------------------------------------------------------------------
// Gerüst
// ---------------------------------------------------------------------------

/// Ein frisches, leeres Arbeitsverzeichnis.
fn working_dir(tmp: &TempDir) -> WorkingDir {
    let dir = WorkingDir::new(tmp.path().join("audio"));
    dir.create().expect("Arbeitsverzeichnis anlegen");
    dir
}

/// Eine Geometrie mit Default-Achse, Default-Schwelle und Default-Policy.
fn config(slice_ms: u32, search_ms: u32) -> Config {
    Config {
        sample_rate: SAMPLE_RATE,
        slice_ms,
        search_ms,
        ..Config::default()
    }
}

/// Push, alle entscheidbaren Fragmente, dann den Rest.
fn prepare(dir: &WorkingDir, config: Config, pcm: &[i16]) -> Vec<Committed> {
    let mut preparer = Preparer::new(dir.clone(), config);
    preparer.push(pcm);

    let mut fragments = Vec::new();
    while let Some(committed) = preparer.commit_ready().expect("ein Fragment") {
        fragments.push(committed);
    }
    while let Some(committed) = preparer.finish().expect("der Rest") {
        fragments.push(committed);
    }
    fragments
}

/// Die Samples einer geschriebenen Datei.
fn payload(committed: &Committed) -> Vec<i16> {
    wav_samples(&fs::read(&committed.path).expect("Datei lesen")).expect("gültiges WAV")
}

/// Die Dateinamen im Arbeitsverzeichnis, sortiert.
fn file_names(dir: &WorkingDir) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.root())
        .expect("Verzeichnis lesen")
        .map(|entry| {
            entry
                .expect("Eintrag")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// Ein See von Sprache mit eingestreuten Stillen (Byte-Bereiche).
fn speech_with_silence(total: f64, gaps: &[(f64, f64)]) -> Vec<i16> {
    let mut audio = speech(total, 6000.0, 180.0);
    for &(from, to) in gaps {
        let from = (from * f64::from(SAMPLE_RATE)) as usize;
        let to = (to * f64::from(SAMPLE_RATE)) as usize;
        for sample in &mut audio[from..to] {
            *sample = 0;
        }
    }
    audio
}

// ---------------------------------------------------------------------------
// Fall 1 — ein Fragment je voller Slice
// ---------------------------------------------------------------------------

#[test]
fn a_full_slice_commits_a_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 8);

    let fragments = prepare(&dir, config(3_000, 1_000), &pcm);

    assert!(
        fragments.len() >= 2,
        "nur {} Fragmente aus {:.1} s",
        fragments.len(),
        pcm.len() as f64 / f64::from(SAMPLE_RATE)
    );
}

// ---------------------------------------------------------------------------
// Fall 2 — lückenlos und byte-exakt
// ---------------------------------------------------------------------------

#[test]
fn fragments_are_gapless_and_reconstruct_the_recording() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 8);

    let fragments = prepare(&dir, config(3_000, 1_000), &pcm);

    let mut rebuilt = Vec::new();
    for fragment in &fragments {
        rebuilt.extend(payload(fragment));
    }
    assert_eq!(
        rebuilt, pcm,
        "die WAV-Nutzlasten ergeben nicht die Aufnahme"
    );

    for pair in fragments.windows(2) {
        assert_eq!(
            pair[0].end_sample, pair[1].start_sample,
            "Lücke zwischen Fragment {} und {}",
            pair[0].number, pair[1].number
        );
    }
    assert_eq!(
        fragments.last().unwrap().end_sample,
        pcm.len() as u64,
        "der Rest endet nicht am Aufnahmeende"
    );
}

// ---------------------------------------------------------------------------
// Fall 3 — Slice und Suchfenster eingehalten
// ---------------------------------------------------------------------------

#[test]
fn every_searched_fragment_sits_inside_slice_and_search_window() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 10);

    let fragments = prepare(&dir, config(3_000, 1_000), &pcm);
    let slice = 3_000 * SAMPLE_RATE / 1_000;
    let search = 1_000 * SAMPLE_RATE / 1_000;

    assert!(fragments.len() >= 2, "zu wenige Fragmente");
    for fragment in &fragments[..fragments.len() - 1] {
        let length = (fragment.end_sample - fragment.start_sample) as usize;
        assert!(
            length >= (slice - search) as usize,
            "Fragment {} ist {} Samples lang, kürzer als {}",
            fragment.number,
            length,
            slice - search
        );
        assert!(
            length < slice as usize,
            "Fragment {} ist {} Samples lang, nicht unter {}",
            fragment.number,
            length,
            slice
        );
    }
}

// ---------------------------------------------------------------------------
// Fall 4 — kein Commit vor der vollen Slice
// ---------------------------------------------------------------------------

#[test]
fn nothing_commits_before_the_full_slice_has_arrived() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let mut preparer = Preparer::new(dir.clone(), config(2_000, 1_000));

    // 1,9 s von 2,0 s: die Slice ist noch nicht voll.
    preparer.push(&speech(1.9, 6000.0, 180.0));
    assert!(
        preparer.commit_ready().expect("kein Fehler").is_none(),
        "vor der vollen Slice wurde geschnitten"
    );

    let rest = preparer.finish().expect("der Rest").expect("ein Rest");
    assert_eq!(rest.number, 1, "der Rest ist das erste Fragment");
    assert_eq!(rest.end_sample, preparer.pcm_size());
    assert!(
        preparer.finish().expect("idempotent").is_none(),
        "der zweite finish liefert noch ein Fragment"
    );

    // Die Grenze selbst: bei vollen 2,0 s ist der Schnitt entscheidbar.
    let boundary_tmp = tempfile::tempdir().unwrap();
    let mut exact = Preparer::new(working_dir(&boundary_tmp), config(2_000, 1_000));
    exact.push(&speech(2.0, 6000.0, 180.0));
    assert!(
        exact.commit_ready().expect("kein Fehler").is_some(),
        "mit voller Slice muss ein Fragment entstehen"
    );
}

// ---------------------------------------------------------------------------
// Fall 5 — der Rest ist ganz und ungesucht
// ---------------------------------------------------------------------------

#[test]
fn the_rest_is_taken_whole_and_unsearched() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 2);

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);
    let rest = fragments.last().expect("der Rest");

    assert_eq!(rest.quality, None, "der Rest ist nicht gemessen");
    assert!(rest.forced, "der Rest ist kein gesuchter Schnitt");
    assert_eq!(rest.cut, None, "der Rest hat keine Schnitt-Evidenz");
    assert_eq!(rest.end_sample, pcm.len() as u64, "der Rest endet zu früh");
    assert!(
        rest.end_sample - rest.start_sample < 2_000 * u64::from(SAMPLE_RATE) / 1_000,
        "der Rest ist keine Restlänge"
    );
}

// ---------------------------------------------------------------------------
// Fall 6 — nur NNN.wav im Verzeichnis
// ---------------------------------------------------------------------------

#[test]
fn the_directory_holds_exactly_one_wav_per_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 6);

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);

    let expected: Vec<String> = (1..=fragments.len())
        .map(|number| format!("{number:03}.wav"))
        .collect();
    assert_eq!(
        file_names(&dir),
        expected,
        "das Verzeichnis enthält nicht genau NNN.wav-Dateien"
    );
    assert_eq!(
        fragments
            .iter()
            .map(|fragment| fragment
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned())
            .collect::<Vec<_>>(),
        expected,
        "die Pfade passen nicht zur Nummerierung"
    );
}

// ---------------------------------------------------------------------------
// Fall 7 — eine Pause im Suchfenster wird benutzt
// ---------------------------------------------------------------------------

#[test]
fn a_pause_inside_the_search_window_takes_the_cut() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let mut pcm = speech(1.8, 6000.0, 180.0);
    pcm.extend(silence(1.0));
    pcm.extend(speech(2.0, 6000.0, 180.0));

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);
    let first = &fragments[0];

    assert!(
        first.quality.expect("gemessen") > 0.0,
        "Q = {:?} — die Pause wurde nicht gefunden",
        first.quality
    );
    assert!(
        (1.8 * f64::from(SAMPLE_RATE)) as u64 <= first.end_sample && first.end_sample <= at(2.0),
        "Schnitt bei {} liegt nicht in der Pause 1,8–2,0 s",
        first.end_sample
    );
    assert!(!first.stitched, "Fragment 1 hat keinen Vorgänger");
    assert!(
        !fragments[1].stitched,
        "Q >= quality_min muss die Naht kahl lassen"
    );
}

// ---------------------------------------------------------------------------
// Fall 8 — eine frühe Senke ist unsichtbar
// ---------------------------------------------------------------------------

#[test]
fn an_early_dip_outside_the_window_is_invisible() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    // Stille bei 0,5–1,0 s, danach durchgehende Sprache im Suchfenster.
    let pcm = speech_with_silence(4.0, &[(0.5, 1.0)]);

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);
    let end = fragments[0].end_sample;

    assert!(
        !(at(0.5)..at(1.0)).contains(&end),
        "der Schnitt bei {end} liegt in der frühen Senke"
    );
    assert!(
        end >= at(1.0),
        "der Schnitt bei {end} liegt vor dem Suchfenster"
    );
}

// ---------------------------------------------------------------------------
// Fall 9 — kürzer als eine Slice, keine Pausen
// ---------------------------------------------------------------------------

#[test]
fn a_recording_shorter_than_one_slice_is_a_single_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = frame(100, 3000);

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);

    assert_eq!(fragments.len(), 1, "mehr als ein Fragment");
    assert_eq!(
        fragments[0].end_sample - fragments[0].start_sample,
        pcm.len() as u64
    );
    assert_eq!(
        payload(&fragments[0]),
        pcm,
        "der Rest ist nicht byte-gleich"
    );
}

// ---------------------------------------------------------------------------
// Fall 10 — Flüsterregime erzwingt den Schnitt
// ---------------------------------------------------------------------------

#[test]
fn a_whisper_band_takes_the_forced_path_and_arms_the_next_seam() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = whisper_band(80);

    let fragments = prepare(&dir, config(2_000, 1_000), &pcm);

    assert!(
        fragments.len() >= 3,
        "zu wenige Fragmente: {}",
        fragments.len()
    );
    assert!(
        fragments[0].forced,
        "der Flüsterschnitt ist nicht erzwungen"
    );
    assert_eq!(
        fragments[0].quality,
        Some(0.0),
        "der erzwungene Schnitt trägt keine Fläche"
    );
    assert!(
        fragments[1].stitched,
        "der erzwungene Schnitt muss die nächste Naht konditionieren"
    );
}

// ---------------------------------------------------------------------------
// Fall 11 — das Q-Gate und sein Grenzfall
// ---------------------------------------------------------------------------

#[test]
fn the_quality_gate_decides_the_next_seam() {
    let pcm = {
        let mut audio = speech(1.8, 6000.0, 180.0);
        audio.extend(silence(1.0));
        audio.extend(speech(2.0, 6000.0, 180.0));
        audio
    };

    // 1. Lauf: die Schwelle misst sich an Q selbst.
    let first = {
        let tmp = tempfile::tempdir().unwrap();
        prepare(&working_dir(&tmp), config(2_000, 1_000), &pcm)
    };
    let measured = first[0].quality.expect("der Endschnitt ist gemessen");
    assert!(
        measured > QUALITY_MIN,
        "Q = {measured} ist kein solider Kandidat"
    );
    assert!(!first[1].stitched, "Q > quality_min muss kahl bleiben");

    // 2. Grenzfall: quality_min == Q — strikt kleiner, also kahl.
    let at_threshold = {
        let tmp = tempfile::tempdir().unwrap();
        let config = Config {
            quality_min: measured,
            ..config(2_000, 1_000)
        };
        prepare(&working_dir(&tmp), config, &pcm)
    };
    assert_eq!(
        at_threshold[0].quality,
        Some(measured),
        "die Schwelle darf den Schnitt nicht verändern"
    );
    assert!(
        !at_threshold[1].stitched,
        "Q == quality_min ist gut genug und bleibt kahl"
    );

    // 3. Knapp darüber: jetzt konditioniert das Gate.
    let above = {
        let tmp = tempfile::tempdir().unwrap();
        let config = Config {
            quality_min: measured + 0.5,
            ..config(2_000, 1_000)
        };
        prepare(&working_dir(&tmp), config, &pcm)
    };
    assert!(
        above[1].stitched,
        "Q < quality_min muss die Naht konditionieren"
    );

    // 4. Ein erzwungener Schnitt (Q == 0) konditioniert immer.
    let forced = {
        let tmp = tempfile::tempdir().unwrap();
        prepare(&working_dir(&tmp), config(2_000, 1_000), &whisper_band(80))
    };
    assert_eq!(forced[0].quality, Some(0.0));
    assert!(forced[1].stitched, "Q == 0 konditioniert");
}

// ---------------------------------------------------------------------------
// Fall 12 — die Policies
// ---------------------------------------------------------------------------

#[test]
fn policy_off_never_conditions_a_seam() {
    let tmp = tempfile::tempdir().unwrap();
    let pcm = burst_train(0.6, 3.0, 6);
    let config = Config {
        policy: SeamPolicy::Off,
        ..config(2_000, 1_000)
    };

    let fragments = prepare(&working_dir(&tmp), config, &pcm);

    assert!(fragments.len() >= 2);
    for fragment in &fragments[1..] {
        assert!(
            !fragment.stitched,
            "Fragment {} wurde konditioniert",
            fragment.number
        );
    }
}

#[test]
fn policy_all_conditions_every_seam_but_the_first() {
    let tmp = tempfile::tempdir().unwrap();
    let pcm = burst_train(0.6, 3.0, 6);
    let config = Config {
        policy: SeamPolicy::All,
        ..config(2_000, 1_000)
    };

    let fragments = prepare(&working_dir(&tmp), config, &pcm);

    assert!(fragments.len() >= 2);
    assert!(!fragments[0].stitched, "Fragment 1 hat keinen Vorgänger");
    for fragment in &fragments[1..] {
        assert!(fragment.stitched, "Fragment {} blieb kahl", fragment.number);
    }
}

#[test]
fn policy_forced_only_conditions_only_a_forced_cut() {
    // Ein gefundener Kandidat bleibt kahl …
    let candidate = {
        let tmp = tempfile::tempdir().unwrap();
        let mut pcm = speech(1.8, 6000.0, 180.0);
        pcm.extend(silence(1.0));
        pcm.extend(speech(2.0, 6000.0, 180.0));
        let config = Config {
            policy: SeamPolicy::ForcedOnly,
            ..config(2_000, 1_000)
        };
        prepare(&working_dir(&tmp), config, &pcm)
    };
    assert!(candidate[0].quality.expect("gemessen") > 0.0);
    assert!(!candidate[1].stitched, "ein Kandidat bleibt kahl");

    // … ein erzwungener Schnitt konditioniert.
    let forced = {
        let tmp = tempfile::tempdir().unwrap();
        let config = Config {
            policy: SeamPolicy::ForcedOnly,
            ..config(2_000, 1_000)
        };
        prepare(&working_dir(&tmp), config, &whisper_band(80))
    };
    assert!(forced[0].forced, "das Flüsterband ist kein Kandidat");
    assert!(forced[1].stitched, "ein erzwungener Schnitt konditioniert");
}

// ---------------------------------------------------------------------------
// Fall 13 — die 999-Grenze und ihr sticky Fehler
// ---------------------------------------------------------------------------

#[test]
fn fragment_number_above_the_limit_is_sticky() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let mut preparer = Preparer::new(dir.clone(), config(100, 30));
    preparer.push(&vec![0i16; 100 * SAMPLE_RATE as usize]);

    let mut committed = 0u32;
    let mut failure = None;
    loop {
        match preparer.commit_ready() {
            Ok(Some(_)) => committed += 1,
            Ok(None) => break,
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    assert_eq!(committed, 999, "nicht genau 999 Fragmente geschrieben");
    assert!(
        matches!(failure, Some(Error::FragmentNumber { number: 1000 })),
        "falscher Fehler: {failure:?}"
    );

    let again = preparer.commit_ready().expect_err("der Fehler ist sticky");
    assert!(
        matches!(again, Error::FragmentNumber { number: 1000 }),
        "der zweite Aufruf liefert einen anderen Fehler: {again:?}"
    );
    assert_eq!(preparer.pcm_size(), 100 * u64::from(SAMPLE_RATE));
}

// ---------------------------------------------------------------------------
// Fall 14 — der Spool wird getrimmt
// ---------------------------------------------------------------------------

#[test]
fn the_spool_is_trimmed_after_every_commit() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 8);
    let mut preparer = Preparer::new(dir, config(3_000, 1_000));

    preparer.push(&pcm);
    let mut committed = 0;
    while preparer.commit_ready().expect("kein Fehler").is_some() {
        committed += 1;
    }

    assert!(committed >= 2, "zu wenige Commits");
    assert_eq!(
        preparer.pcm_size(),
        pcm.len() as u64,
        "pcm_size zählt nicht alles"
    );
    let bound = (3_000 + 1_000) * SAMPLE_RATE as usize / 1_000;
    assert!(
        preparer.spool_samples() <= bound,
        "der Spool hält {} Samples, mehr als {}",
        preparer.spool_samples(),
        bound
    );
}

// ---------------------------------------------------------------------------
// Fall 15 — der Worker committet während der Aufnahme
// ---------------------------------------------------------------------------

#[test]
fn the_worker_commits_while_audio_arrives() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = working_dir(&tmp);
    let pcm = burst_train(0.6, 3.0, 4);
    let (worker, receiver) = Worker::start(
        Preparer::new(dir, config(2_000, 1_000)),
        Duration::from_millis(10),
    );

    thread::scope(|scope| {
        scope.spawn(|| {
            for chunk in pcm.chunks(SAMPLE_RATE as usize / 10) {
                worker.push(chunk);
                thread::sleep(Duration::from_millis(1));
            }
        });
    });

    let mut fragments = Vec::new();
    while fragments.len() < 2 {
        let committed = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("mindestens zwei Fragmente");
        fragments.push(committed);
    }

    worker.finish().expect("der Faden endet sauber");
    while let Ok(committed) = receiver.recv() {
        fragments.push(committed);
    }

    let mut rebuilt = Vec::new();
    for fragment in &fragments {
        rebuilt.extend(payload(fragment));
    }
    assert_eq!(rebuilt, pcm, "der Worker verliert oder verdoppelt Samples");
    assert_eq!(
        fragments.last().unwrap().end_sample,
        pcm.len() as u64,
        "der Worker nimmt den Rest nicht mit"
    );
}

// ---------------------------------------------------------------------------
// Fall 16 — der Worker meldet einen Schreibfehler
// ---------------------------------------------------------------------------

#[test]
fn the_worker_reports_a_write_error() {
    let tmp = tempfile::tempdir().unwrap();
    let blocker = tmp.path().join("datei");
    fs::write(&blocker, b"kein Verzeichnis").expect("Datei anlegen");
    // `<datei>/unter` kann nicht entstehen: das Elternverzeichnis ist eine Datei.
    let dir = WorkingDir::new(blocker.join("unter"));

    let (worker, _receiver) = Worker::start(
        Preparer::new(dir, config(1_000, 500)),
        Duration::from_millis(10),
    );
    worker.push(&vec![1000i16; 24_000]);

    let error = worker.finish().expect_err("der Schreibfehler kommt zurück");
    assert!(matches!(error, Error::Io(_)), "falscher Fehler: {error:?}");
}

// ---------------------------------------------------------------------------
// Die Default-Geometrie
// ---------------------------------------------------------------------------

#[test]
fn the_default_geometry_is_the_agreed_one() {
    let config = Config::default();

    assert_eq!(config.sample_rate, 16_000);
    assert_eq!(config.slice_ms, 21_000, "die Obergrenze eines Fragments");
    assert_eq!(config.search_ms, 6_300, "30 % von 21 s");
    assert_eq!(config.margin_ms, 150);
    assert_eq!(config.axis, Axis::default());
    assert_eq!(config.quality_min, QUALITY_MIN);
    assert_eq!(config.policy, SeamPolicy::QualityGate);
}
