//! Portierte DSP-Tests: Stillemaß `Q`, Hochpass, Konditionierung und WAV.
//!
//! Quellen, wörtlich übersetzt (nur Bytes → Samples):
//!
//! - `polished-recognition`, `SilenceQualityTest.kt` — die exakten
//!   Schnitt-Offsets der Kotlin-Tests sind Byte-Offsets und damit das
//!   **Doppelte** der hier geprüften Sample-Offsets.
//! - `polished-recognition`, `PcmConditionerTest.kt` — Samples statt
//!   Little-Endian-Bytes.
//! - `aitranscribe`, `tests/test_fragmenter.py` — `frame_levels` und
//!   `high_pass`.
//!
//! Testmaterial sind **Nyquist-Rechtecke**: Sample `k` abwechselnd `+amp` /
//! `−amp`. Jedes volle Fenster hat denselben RMS, der Einpoler lässt rund
//! `0,985·amp` durch, und die Segmentlängen sind Vielfache eines Rahmens.
//!
//! Nicht portiert: der Kotlin-Test „odd-length input is trimmed" — die Eingabe
//! sind hier Samples, kein Byte-Puffer; eine halbe Probe gibt es nicht. Ebenso
//! nicht portiert: OGG, Transcoder, Manifest und Room-Latenz
//! (`polished-recognition`-Infrastruktur, die es in der Diktatwerkstatt nicht
//! gibt).

use diktatwerkstatt_core::{
    Axis, Error, FRAME_MS, condition, evaluate, frame_levels, high_pass, wav_bytes, wav_samples,
};

/// Die Abtastrate des Kerns.
const SAMPLE_RATE: u32 = 16_000;

/// Samples eines 10-ms-Rahmens bei [`SAMPLE_RATE`].
const FRAME_SAMPLES: usize = 160;

/// Der Rand der Schnittklemmung in Millisekunden (Referenz: 15 Rahmen).
const MARGIN_MS: u32 = 150;

/// Der Referenzrand in Samples (150 ms).
const MARGIN_SAMPLES: usize = 2_400;

// ---------------------------------------------------------------------------
// Testmaterial
// ---------------------------------------------------------------------------

/// Nyquist-Rechteck: Sample `k` abwechselnd `+amp` / `-amp`.
fn seg(samples: usize, amp: i16) -> Vec<i16> {
    (0..samples)
        .map(|k| if k % 2 == 0 { amp } else { -amp })
        .collect()
}

/// `n` volle Rahmen Nyquist-Rechteck.
fn frame(n: usize, amp: i16) -> Vec<i16> {
    seg(n * FRAME_SAMPLES, amp)
}

/// Ein 25-Hz-Sinus, beginnend bei `start_sample` mit Phasenversatz.
fn hum(samples: usize, amp: f64, phase: f64, start_sample: usize) -> Vec<i16> {
    (0..samples)
        .map(|i| {
            let sample = start_sample + i;
            let value =
                amp * (2.0 * std::f64::consts::PI * 25.0 * sample as f64 / 16_000.0 + phase).sin();
            // Kotlin `Double.toInt()` schneidet gegen Null ab.
            value.trunc() as i16
        })
        .collect()
}

/// Sampleweise sättigende Summe zweier gleich langer Segmente.
fn mix(a: &[i16], b: &[i16]) -> Vec<i16> {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| (i32::from(x) + i32::from(y)).clamp(-32768, 32767) as i16)
        .collect()
}

/// Bewertet mit der Default-Achse und dem Referenzrand.
fn cut(span: &[i16]) -> diktatwerkstatt_core::Cut {
    evaluate(span, SAMPLE_RATE, Axis::default(), MARGIN_MS).expect("das Fenster hat einen Rahmen")
}

/// Der Schnitt liegt im Stillenbereich und hält den Rand zu beiden
/// Achsenkanten.
fn cut_inside_pause_with_margins(
    cut: &diktatwerkstatt_core::Cut,
    pause: std::ops::Range<usize>,
    left_edge: usize,
    right_edge: usize,
) {
    assert!(
        cut.cut_sample > pause.start,
        "Schnitt {} liegt nicht hinter dem Stillenbeginn {}",
        cut.cut_sample,
        pause.start
    );
    assert!(
        cut.cut_sample < pause.end,
        "Schnitt {} liegt nicht vor dem Stillenende {}",
        cut.cut_sample,
        pause.end
    );
    assert!(
        cut.cut_sample >= left_edge + MARGIN_SAMPLES,
        "Schnitt {} hält den Rand zur linken Achsenkante {} nicht",
        cut.cut_sample,
        left_edge
    );
    assert!(
        cut.cut_sample + MARGIN_SAMPLES <= right_edge,
        "Schnitt {} hält den Rand zur rechten Achsenkante {} nicht",
        cut.cut_sample,
        right_edge
    );
}

/// Der "bist"-Aufbau: Sprache, Frikativ, Verschluss, Pause, Sprache.
fn bist_span() -> Vec<i16> {
    [
        frame(20, 3500),
        frame(10, 250),
        frame(10, 0),
        frame(30, 0),
        frame(20, 3500),
    ]
    .concat()
}

// ---------------------------------------------------------------------------
// SilenceQuality — die "bist"-Regression
// ---------------------------------------------------------------------------

#[test]
fn bist_fixture_cuts_strictly_inside_the_true_pause() {
    let span = bist_span();

    let cut = cut(&span);

    assert!(cut.candidate, "es gibt einen Kandidaten");
    assert!(
        cut.quality > 40.0,
        "Q = {} ist kein solider Kandidat",
        cut.quality
    );
    assert!(
        cut.quality <= 50.0,
        "Q = {} überschreitet die Kandidatenlänge",
        cut.quality
    );
    assert_eq!(cut.cut_sample, 7_280, "Schnitt (14560 B in der Referenz)");
    cut_inside_pause_with_margins(&cut, 6_400..11_200, 19 * FRAME_SAMPLES, 70 * FRAME_SAMPLES);
    // Das ganze erste Wort (Sprache + Frikativ endet bei 4800) liegt davor.
    assert!(cut.cut_sample > 4_800, "der Schnitt liegt im ersten Wort");
}

#[test]
fn fricative_above_the_axis_is_hard_excluded() {
    let span = [frame(15, 400), frame(40, 0)].concat();

    let cut = cut(&span);

    assert!(cut.candidate, "es gibt einen Kandidaten");
    // Die 40 Pausenrahmen tragen die ganze Fläche; die Frikativrahmen tragen
    // rund 0,01 (der Übergangsrahmen des Einpolers).
    assert!(cut.quality > 39.0, "Q = {} ist zu klein", cut.quality);
    assert!(
        cut.quality <= 40.0,
        "Q = {} verwässert den Kandidaten",
        cut.quality
    );
    assert_eq!(cut.cut_sample, 5_680, "Schnitt (11360 B in der Referenz)");
    cut_inside_pause_with_margins(&cut, 2_400..8_800, 14 * FRAME_SAMPLES, 55 * FRAME_SAMPLES);
}

// ---------------------------------------------------------------------------
// 25-Hz-Brummen: der Hochpass trägt die Entscheidung
// ---------------------------------------------------------------------------

#[test]
fn hum_does_not_fabricate_beat_candidates() {
    let phase = std::f64::consts::FRAC_PI_4;
    let speech_hum_first = mix(&frame(25, 3000), &hum(25 * FRAME_SAMPLES, 4000.0, phase, 0));
    let pause_hum = hum(40 * FRAME_SAMPLES, 4000.0, phase, 25 * FRAME_SAMPLES);
    let speech_hum_last = mix(
        &frame(25, 3000),
        &hum(25 * FRAME_SAMPLES, 4000.0, phase, 65 * FRAME_SAMPLES),
    );
    let span = [speech_hum_first, pause_hum, speech_hum_last].concat();

    let cut = cut(&span);

    assert!(cut.candidate, "die Pause bleibt ein Kandidat");
    assert!(
        cut.quality > 10.0,
        "Q = {} ist ein zerschlagenes Kandidatenfragment",
        cut.quality
    );
    assert_eq!(cut.cut_sample, 7_280, "Schnitt (14560 B in der Referenz)");
    cut_inside_pause_with_margins(&cut, 4_000..10_400, 24 * FRAME_SAMPLES, 65 * FRAME_SAMPLES);
}

// ---------------------------------------------------------------------------
// Flüsterregime
// ---------------------------------------------------------------------------

#[test]
fn whisper_band_threaded_by_the_axis_yields_a_shallow_candidate() {
    let mut span = Vec::new();
    for block in 0..12 {
        span.extend(frame(5, if block % 2 == 0 { 120 } else { 240 }));
    }

    let cut = cut(&span);

    assert!(cut.candidate, "ein flacher Kandidat");
    assert!(
        cut.quality < 5.0,
        "Q = {} ist nicht mehr flach",
        cut.quality
    );
    assert_eq!(
        cut.cut_sample, 400,
        "Mitte des ersten stillen Blocks (800 B)"
    );
}

#[test]
fn tight_whisper_band_takes_the_forced_path() {
    let mut span = Vec::new();
    for block in 0..12 {
        span.extend(frame(5, if block % 2 == 0 { 380 } else { 420 }));
    }

    let cut = cut(&span);

    assert!(!cut.candidate, "kein Rahmen liegt unter der Achse");
    assert_eq!(cut.quality, 0.0, "der Notweg trägt keine Fläche");
    assert!(
        cut.axis < cut.floor_level,
        "Achse {} liegt nicht unter dem Boden {}",
        cut.axis,
        cut.floor_level
    );
    assert_eq!(cut.cut_sample, 80, "leisester Rahmen (160 B)");
}

// ---------------------------------------------------------------------------
// Verstärkungsinvarianz und Raumwechsel
// ---------------------------------------------------------------------------

#[test]
fn gain_invariance_moves_neither_quality_nor_centroid() {
    let full = bist_span();
    let half = [
        frame(20, 1750),
        frame(10, 125),
        frame(10, 0),
        frame(30, 0),
        frame(20, 1750),
    ]
    .concat();

    let no_anchor = Axis { k: 0.65, c: 0.0 };
    let first = evaluate(&full, SAMPLE_RATE, no_anchor, MARGIN_MS).expect("volles Fenster");
    let second = evaluate(&half, SAMPLE_RATE, no_anchor, MARGIN_MS).expect("halbes Fenster");

    assert!(
        (second.quality - first.quality).abs() <= 0.01,
        "Q verschoben: {} gegen {}",
        second.quality,
        first.quality
    );
    assert_eq!(second.centroid_sample, first.centroid_sample, "Zentroid");
    assert_eq!(second.cut_sample, first.cut_sample, "Schnitt");
    assert!(
        (second.axis - first.axis / 2.0).abs() <= 0.5,
        "Achse {} ist nicht die halbe Achse {}",
        second.axis,
        first.axis
    );
}

#[test]
fn the_absolute_anchor_deliberately_breaks_scale_invariance() {
    let full = bist_span();
    let half = [
        frame(20, 1750),
        frame(10, 125),
        frame(10, 0),
        frame(30, 0),
        frame(20, 1750),
    ]
    .concat();

    let first = cut(&full);
    let second = cut(&half);

    assert!(
        (second.quality - first.quality).abs() > 0.01,
        "Q blieb gleich ({} gegen {}) — der Anker wirkt nicht",
        second.quality,
        first.quality
    );
}

#[test]
fn room_change_adapts_the_axis_and_keeps_the_cut() {
    let quiet = bist_span();
    let loud = [
        frame(20, 7000),
        frame(10, 500),
        frame(10, 0),
        frame(30, 0),
        frame(20, 7000),
    ]
    .concat();

    let first = cut(&quiet);
    let second = cut(&loud);

    assert!(second.candidate, "auch im lauten Raum ein Kandidat");
    assert!(second.quality > 40.0, "Q = {} ist zu klein", second.quality);
    assert!(
        second.axis > first.axis,
        "die Achse {} ist nicht gewachsen ({})",
        second.axis,
        first.axis
    );
    assert!(
        second.axis < 2.0 * first.axis,
        "die Achse {} hat sich verdoppelt ({})",
        second.axis,
        first.axis
    );
    assert_eq!(
        second.cut_sample, first.cut_sample,
        "Schnitt im Raumwechsel"
    );
    cut_inside_pause_with_margins(
        &second,
        6_400..11_200,
        19 * FRAME_SAMPLES,
        70 * FRAME_SAMPLES,
    );
}

// ---------------------------------------------------------------------------
// Dauersprache und Tiefe × Dauer
// ---------------------------------------------------------------------------

#[test]
fn continuous_speech_has_no_candidate() {
    let cut = cut(&frame(60, 3000));

    assert!(!cut.candidate, "Dauersprache hat keinen Kandidaten");
    assert_eq!(cut.quality, 0.0);
    assert!(
        (cut.floor_level - cut.mean_level).abs() <= 1.0,
        "Boden {} und Mittel {} fallen auseinander",
        cut.floor_level,
        cut.mean_level
    );
    // Der Einschwing-Überschwinger macht Rahmen 0 etwas lauter als Rahmen 1.
    assert_eq!(cut.cut_sample, 240, "leisester Rahmen (480 B)");
}

#[test]
fn a_short_deep_pause_outscores_a_long_shallow_dip() {
    let span = [
        frame(20, 3000),
        frame(25, 800),
        frame(5, 3000),
        frame(15, 0),
    ]
    .concat();

    let cut = cut(&span);

    assert!(cut.candidate, "es gibt einen Kandidaten");
    assert!(
        cut.quality > 14.0,
        "Q = {} — die Senke hat gewonnen",
        cut.quality
    );
    assert!(
        cut.quality <= 15.5,
        "Q = {} überschreitet die Pausenlänge",
        cut.quality
    );
    assert_eq!(cut.cut_sample, 9_200, "die Pausenmitte (18400 B)");
    assert!(cut.cut_sample > 8_000, "der Schnitt liegt vor der Pause");
    assert!(
        cut.cut_sample < 10_400,
        "der Schnitt liegt hinter der Pause"
    );
}

// ---------------------------------------------------------------------------
// Entartete Eingaben
// ---------------------------------------------------------------------------

#[test]
fn spans_below_one_frame_return_none() {
    let under_one_frame = vec![0i16; FRAME_SAMPLES - 1];
    assert!(
        evaluate(&under_one_frame, SAMPLE_RATE, Axis::default(), MARGIN_MS).is_none(),
        "159 Samples sind kein voller Rahmen"
    );
    assert!(
        evaluate(&[], SAMPLE_RATE, Axis::default(), MARGIN_MS).is_none(),
        "ein leeres Fenster hat keinen Rahmen"
    );
}

// ---------------------------------------------------------------------------
// Konditionierung (PcmConditionerTest, auf Samples)
// ---------------------------------------------------------------------------

/// Ein Sinus als Sample-Folge, wie im Kotlin-Test (`toInt().toShort()`).
fn sine(amplitude: f64, periods: u32, count: usize) -> Vec<i16> {
    let step = 2.0 * std::f64::consts::PI * f64::from(periods) / count as f64;
    (0..count)
        .map(|i| (amplitude * (i as f64 * step).sin()).trunc() as i16)
        .collect()
}

/// Die Spitze einer Sample-Folge als Betrag.
fn peak(samples: &[i16]) -> i32 {
    samples
        .iter()
        .map(|&sample| i32::from(sample).abs())
        .max()
        .unwrap_or(0)
}

#[test]
fn quiet_signal_is_amplified_to_the_target_peak() {
    let out = condition(&sine(5000.0, 250, 8000), SAMPLE_RATE);
    assert!(
        (peak(&out) - 26214).abs() <= 60,
        "Spitze {} liegt nicht bei 26214",
        peak(&out)
    );
}

#[test]
fn loud_signal_is_not_attenuated() {
    let out = condition(&sine(32_000.0, 250, 8000), SAMPLE_RATE);
    assert!(peak(&out) >= 31_500, "Spitze {} wurde gedämpft", peak(&out));
}

#[test]
fn silence_is_left_untouched() {
    let out = condition(&vec![0i16; 4000], SAMPLE_RATE);
    assert_eq!(peak(&out), 0, "Stille wurde angehoben");
}

#[test]
fn near_silence_is_not_amplified() {
    let out = condition(&sine(100.0, 250, 8000), SAMPLE_RATE);
    assert!(
        (peak(&out) - 100).abs() <= 60,
        "Spitze {} liegt nicht bei 100",
        peak(&out)
    );
}

#[test]
fn deep_negative_samples_survive_without_wraparound() {
    let out = condition(&[-32768, -30000, 30000, 32767], SAMPLE_RATE);
    let peak = peak(&out);
    assert!(peak >= 31_000, "Spitze {} ist zu klein", peak);
    assert!(peak <= 32_767, "Spitze {} ist übergelaufen", peak);
}

#[test]
fn dc_offset_is_removed_by_the_high_pass() {
    let out = condition(&vec![5000i16; 8000], SAMPLE_RATE);
    let mean = out.iter().map(|&s| f64::from(s)).sum::<f64>() / out.len() as f64;
    assert!(
        mean.abs() <= 200.0,
        "Mittel {} ist kein Quantisierungsrest",
        mean
    );
    assert_eq!(
        out.last().copied(),
        Some(0),
        "der letzte Sample ist nicht 0"
    );
}

#[test]
fn empty_input_returns_empty_output() {
    assert!(condition(&[], SAMPLE_RATE).is_empty());
    assert!(high_pass(&[], SAMPLE_RATE).is_empty());
}

// ---------------------------------------------------------------------------
// frame_levels / high_pass (aus tests/test_fragmenter.py)
// ---------------------------------------------------------------------------

#[test]
fn frame_levels_are_the_rms_of_each_full_frame() {
    let signal = vec![1000i16; 3 * FRAME_SAMPLES];

    let levels = frame_levels(&signal, FRAME_SAMPLES);

    assert_eq!(levels.len(), 3, "drei volle Rahmen");
    for level in &levels {
        assert!(
            (level - 1000.0).abs() < 1e-9,
            "Rahmenpegel {level} ist nicht 1000"
        );
    }
}

#[test]
fn frame_levels_ignore_the_remainder_and_reject_a_zero_frame() {
    let signal = vec![1000i16; 3 * FRAME_SAMPLES + 5];

    assert_eq!(frame_levels(&signal, FRAME_SAMPLES).len(), 3);
    assert!(
        frame_levels(&signal, 0).is_empty(),
        "Rahmen 0 hat keinen Pegel"
    );
}

#[test]
fn high_pass_removes_a_dc_offset() {
    let dc = vec![1000i16; 16_000];

    let filtered = high_pass(&dc, SAMPLE_RATE);

    assert_eq!(filtered.len(), dc.len(), "der Hochpass ist längentreu");
    let tail = &filtered[100..];
    let mean = tail.iter().map(|&s| f64::from(s)).sum::<f64>() / tail.len() as f64;
    assert!(mean.abs() < 1.0, "Mittel {mean} — der Gleichanteil steht");
}

// ---------------------------------------------------------------------------
// WAV
// ---------------------------------------------------------------------------

/// Liest ein `u32` Little-Endian an `offset`.
fn le_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Liest ein `u16` Little-Endian an `offset`.
fn le_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

#[test]
fn wav_header_describes_16_bit_mono_pcm() {
    let samples = [1i16, -2, 3, 4];
    let bytes = wav_bytes(&samples, SAMPLE_RATE);

    assert_eq!(bytes.len(), 44 + 2 * samples.len(), "Kopf plus Nutzlast");
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(le_u32(&bytes, 4) as usize, bytes.len() - 8, "RIFF-Länge");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");
    assert_eq!(le_u32(&bytes, 16), 16, "fmt-Länge");
    assert_eq!(le_u16(&bytes, 20), 1, "PCM");
    assert_eq!(le_u16(&bytes, 22), 1, "mono");
    assert_eq!(le_u32(&bytes, 24), SAMPLE_RATE, "Abtastrate");
    assert_eq!(le_u32(&bytes, 28), 2 * SAMPLE_RATE, "Byte-Rate");
    assert_eq!(le_u16(&bytes, 32), 2, "Blockausrichtung");
    assert_eq!(le_u16(&bytes, 34), 16, "Bits");
    assert_eq!(&bytes[36..40], b"data");
    assert_eq!(le_u32(&bytes, 40) as usize, 2 * samples.len(), "data-Länge");
}

#[test]
fn wav_round_trips_every_sample_value() {
    let samples = [0i16, 1, -1, 12_345, -12_345, 32_767, -32_768, 1000];

    assert_eq!(
        wav_samples(&wav_bytes(&samples, SAMPLE_RATE)).expect("gültige Datei"),
        samples
    );
}

#[test]
fn empty_audio_is_a_header_without_payload() {
    let bytes = wav_bytes(&[], SAMPLE_RATE);

    assert_eq!(bytes.len(), 44, "nur der Kopf");
    assert_eq!(le_u32(&bytes, 40), 0, "leere Nutzlast");
    assert_eq!(
        wav_samples(&bytes).expect("gültige Datei"),
        Vec::<i16>::new()
    );
}

#[test]
fn wav_rejects_foreign_bytes() {
    // Ein strukturell gültiger Kopf mit Stereo-fmt: 16-Bit-Stereo ist kein
    // Fragment der Diktatwerkstatt.
    let mut stereo = wav_bytes(&[1i16, 2], SAMPLE_RATE);
    stereo[22..24].copy_from_slice(&2u16.to_le_bytes());
    let error = wav_samples(&stereo).expect_err("Stereo wird abgelehnt");
    assert!(matches!(error, Error::Wav(_)), "falscher Fehler: {error:?}");

    // Und ein Puffer, der gar kein RIFF ist.
    assert!(matches!(wav_samples(b"RIFF"), Err(Error::Wav(_))));
}

// ---------------------------------------------------------------------------
// Konstanten
// ---------------------------------------------------------------------------

#[test]
fn frame_is_ten_milliseconds() {
    assert_eq!(FRAME_MS, 10);
    assert_eq!(SAMPLE_RATE / (1_000 / FRAME_MS), FRAME_SAMPLES as u32);
}
