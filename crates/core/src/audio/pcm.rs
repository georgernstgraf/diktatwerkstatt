//! PCM-Werkzeuge: Hochpass, Rahmenpegel, Upload-Konditionierung und WAV.
//!
//! Die Analyse arbeitet auf **Samples** (`&[i16]`), nicht auf Bytes: 16 kHz mono
//! ist die Domäne, Little-Endian ist eine Frage der Datei. Der Hochpass ist
//! derselbe Einpoler, den der Upload-Weg benutzt — ein Filter, zwei Aufrufer
//! (Messung und Konditionierung), damit die Entscheidung sieht, was der Server
//! hört.
//!
//! Der Hochpass **quantisiert auf `i16`**, sampleweise, und rechnet intern
//! weiter mit dem unquantisierten Wert. Das ist keine Nachlässigkeit, sondern
//! die Rechnung der Referenz: die Rahmenpegel entstehen über die quantisierten
//! Samples, und genau daran hängen die exakten Schnitt-Offsets.

use crate::error::{Error, Result};

/// Länge eines Analyse-Rahmens in Millisekunden.
///
/// 10 ms bei 16 kHz sind 160 Samples; ein Schnitt liegt immer auf einer
/// Rahmenmitte und ist damit sample-, aber nicht byte-genau beliebig.
pub const FRAME_MS: u32 = 10;

/// Grenzfrequenz des Einpolers — entfernt Rumpeln und Netzbrummen.
const HIGH_PASS_HZ: f64 = 80.0;

/// Zielspitze der Normierung: ≈ 0,8 der Vollaussteuerung.
const TARGET_PEAK: f64 = 26214.0;

/// Mehr als das Zehnfache wird nie verstärkt — sonst hebt man Rauschen an.
const MAX_GAIN: f32 = 10.0;

/// Unter dieser Spitze gilt der Puffer als Stille und bleibt unangetastet.
const MIN_PEAK: i32 = 200;

/// Ab diesem Faktor lohnt die Normierung nicht mehr (schon laut genug).
const GAIN_THRESHOLD: f32 = 1.01;

/// Einpoliger Hochpass mit 80 Hz, **ohne** Normierung.
///
/// Genau die Rechnung der Referenz, einschließlich der Quantisierung auf `i16`:
/// der Filterzustand bleibt der unquantisierte Wert, gespeichert wird der
/// abgeschnittene. `sample_rate == 0` gibt die Eingabe unverändert zurück — ohne
/// Abtastrate gibt es keine Grenzfrequenz.
pub fn high_pass(samples: &[i16], sample_rate: u32) -> Vec<i16> {
    if sample_rate == 0 {
        return samples.to_vec();
    }

    let rc = 1.0 / (2.0 * std::f64::consts::PI * HIGH_PASS_HZ);
    let alpha = rc / (rc + 1.0 / f64::from(sample_rate));
    let mut previous_in = 0.0;
    let mut previous_out = 0.0;

    samples
        .iter()
        .map(|&sample| {
            let x = f64::from(sample);
            let y = alpha * (previous_out + x - previous_in);
            previous_in = x;
            previous_out = y;
            quantize(y)
        })
        .collect()
}

/// Der Pegel jedes **vollen** Rahmens: `sqrt(Σ sample² / frame_samples)`.
///
/// Der Rest am Ende zählt nicht; weniger als ein voller Rahmen ergibt einen
/// leeren Vektor. `frame_samples == 0` ebenfalls. Die Summe läuft über `i64`,
/// damit sie exakt bleibt.
pub fn frame_levels(samples: &[i16], frame_samples: usize) -> Vec<f64> {
    if frame_samples == 0 {
        return Vec::new();
    }

    samples
        .chunks_exact(frame_samples)
        .map(|frame| {
            let sum: i64 = frame.iter().map(|&s| i64::from(s) * i64::from(s)).sum();
            (sum as f64 / frame_samples as f64).sqrt()
        })
        .collect()
}

/// Hochpass plus Spitzennormierung — der Weg zum STT-Endpunkt.
///
/// Verstärkt leise Aufnahmen auf [`TARGET_PEAK`], dämpft aber **nie**: ab einer
/// Spitze von 200 und einem nötigen Faktor über 1,01 bleibt der Puffer, wie der
/// Hochpass ihn geliefert hat.
pub fn condition(samples: &[i16], sample_rate: u32) -> Vec<i16> {
    let filtered = high_pass(samples, sample_rate);
    if filtered.is_empty() {
        return filtered;
    }

    let peak = filtered
        .iter()
        .map(|&sample| i32::from(sample).abs())
        .max()
        .unwrap_or(0);
    if peak < MIN_PEAK {
        return filtered;
    }

    let gain = (TARGET_PEAK as f32 / peak as f32).min(MAX_GAIN);
    if gain <= GAIN_THRESHOLD {
        return filtered;
    }

    filtered
        .iter()
        .map(|&sample| {
            let amplified = (f32::from(sample) * gain).trunc() as i32;
            amplified.clamp(-32768, 32767) as i16
        })
        .collect()
}

/// Eine WAV-Datei (44-Byte-RIFF-Kopf, PCM, mono, 16 Bit) als Bytes.
///
/// Alle Längen und Werte Little-Endian; die Nutzlast sind die Samples in
/// Reihenfolge. Gegenstück ist [`wav_samples`].
pub fn wav_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_length = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_length as usize);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_length).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");

    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());

    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_length.to_le_bytes());
    for &sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

/// Liest die `data`-Nutzlast einer WAV-Datei und prüft das Format.
///
/// Akzeptiert wird nur 16-Bit-Mono-PCM; alles andere ist
/// [`Error::Wav`]. Die Chunks werden der Reihe nach abgelaufen, damit auch
/// Dateien mit Zusatz-Chunks (etwa `LIST`) gelesen werden — geschrieben wird
/// freilich nur der 44-Byte-Kopf aus [`wav_bytes`].
pub fn wav_samples(bytes: &[u8]) -> Result<Vec<i16>> {
    if bytes.len() < 12 {
        return Err(Error::Wav("Kopf ist kürzer als 12 Byte"));
    }
    if &bytes[0..4] != b"RIFF" {
        return Err(Error::Wav("kein RIFF-Kopf"));
    }
    if &bytes[8..12] != b"WAVE" {
        return Err(Error::Wav("keine WAVE-Datei"));
    }

    let mut format: Option<&[u8]> = None;
    let mut payload: Option<&[u8]> = None;
    let mut offset = 12;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let length = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let start = offset + 8;
        let end = start
            .checked_add(length)
            .filter(|&end| end <= bytes.len())
            .ok_or(Error::Wav("Chunk reicht über das Dateiende"))?;

        match id {
            b"fmt " => format = Some(&bytes[start..end]),
            b"data" => payload = Some(&bytes[start..end]),
            _ => {}
        }
        offset = end + (length & 1);
    }

    let format = format.ok_or(Error::Wav("fmt-Chunk fehlt"))?;
    if format.len() < 16
        || u16::from_le_bytes([format[0], format[1]]) != 1
        || u16::from_le_bytes([format[2], format[3]]) != 1
        || u16::from_le_bytes([format[14], format[15]]) != 16
    {
        return Err(Error::Wav("kein 16-Bit-Mono-PCM"));
    }

    let payload = payload.ok_or(Error::Wav("data-Chunk fehlt"))?;
    if payload.len() % 2 != 0 {
        return Err(Error::Wav("data-Länge ist ungerade"));
    }

    Ok((0..payload.len() / 2)
        .map(|index| i16::from_le_bytes([payload[2 * index], payload[2 * index + 1]]))
        .collect())
}

/// Klemmt einen Filterwert auf den Wertebereich von `i16`, abgeschnitten gegen
/// Null — Kotlins `coerceIn(...).toInt().toShort()`.
fn quantize(value: f64) -> i16 {
    value.clamp(-32768.0, 32767.0).trunc() as i32 as i16
}
