//! Das Stillemaß Q und die Schnittentscheidung im Suchfenster.
//!
//! Stille ist hier keine Schwelle, sondern eine **Qualität**: über einer
//! angepassten Achse `X = k·M + c` trägt jeder Rahmen unter der Achse den
//! Mangel `1 − level/X` bei. Die Summe dieser Beiträge ist `Q` — Tiefe und
//! Dauer in einer Zahl. Der Schnitt liegt im **Flächenzentroid** des besten
//! zusammenhängenden Kandidaten, geklemmt um einen Rand von 150 ms zu den
//! Achsenkanten.
//!
//! Warum das die Schwelle ersetzt: ein wortfinaler Frikativ, ein
//! Verschluss und die folgende Pause verschmelzen unter einer absoluten
//! Schwelle zu **einem** „Stillenlauf", und der geometrische Mittelpunkt des
//! Laufs landet im Wort. Ein Frikativ über der Achse trägt dagegen **null**
//! Fläche und kann den Zentroid nicht verwässern.
//!
//! Ohne Kandidaten (Dauersprache, oder ein Flüsterraum, in dem das absolute
//! Ankergewicht `c` die Achse über allem hält) wird trotzdem geschnitten: am
//! **leisesten** Rahmen, mit `Q = 0`. Ein Schnitt wird immer gesetzt; `Q`
//! entscheidet erst später über die Naht.

use crate::audio::pcm::{FRAME_MS, frame_levels, high_pass};

/// Die Achse `X = k·M + c` über dem Mittelpegel `M` des Suchfensters.
///
/// `k` ist multiplikativ und damit verstärkungsinvariant (Pegel und Achse
/// skalieren gemeinsam), `c` ist das einzige absolute Element — der Anker für
/// Flüstern und stille Räume. `c > 0` bricht die Skaleninvarianz bewusst.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    /// Der multiplikative Anteil über dem Mittelpegel.
    pub k: f64,
    /// Der absolute Anteil in Pegel-Einheiten (RMS eines `i16`-Samples).
    pub c: f64,
}

impl Default for Axis {
    /// `k = 0.65`, `c = 100.0` — die abgestimmten Startwerte.
    fn default() -> Self {
        Self { k: 0.65, c: 100.0 }
    }
}

/// Das Ergebnis einer Fensterbewertung: wo geschnitten wird und wie gut.
///
/// `centroid_sample` ist der rohe Flächenzentroid, `cut_sample` der
/// randgeklemmte Schnitt, den der Aufrufer tatsächlich benutzt. Beide sind
/// Rahmenmitten und damit sample-genau. Ohne Kandidaten sind beide die Mitte
/// des leisesten Rahmens und `quality` ist `0.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    /// Mittelpegel über alle Rahmen des Fensters.
    pub mean_level: f64,
    /// Der P5-nahe Bodenwert der sortierten Rahmenpegel (kein Einzelminimum).
    pub floor_level: f64,
    /// Der lauteste Rahmenpegel im Fenster.
    pub max_level: f64,
    /// Die wirksame Achse `X`.
    pub axis: f64,
    /// Die Mangel-Fläche `Q` des besten Kandidaten (`0.0` ohne Kandidaten).
    pub quality: f64,
    /// Der Flächenzentroid als Sample-Offset im Fenster.
    pub centroid_sample: usize,
    /// Der tatsächlich benutzte Schnitt als Sample-Offset im Fenster.
    pub cut_sample: usize,
    /// Ob ein Rahmen unter der Achse lag (sonst ist der Schnitt erzwungen).
    pub candidate: bool,
}

/// Der Anteil der leisesten Rahmen, der als Boden zählt: „P5-artig".
const FLOOR_PERCENTILE: f64 = 5.0;

/// Bewertet das Suchfenster `span` und liefert den Schnitt — oder `None`, wenn
/// das Fenster keinen vollen Rahmen enthält.
///
/// `sample_rate / 100` ist die Rahmengröße (16 kHz → 160 Samples),
/// `margin_ms / 10` der Mindestabstand zu den Achsenkanten in Rahmen.
/// Gleichstände lösen immer auf den **frühesten** Kandidaten und den
/// **frühesten** Rahmen auf.
pub fn evaluate(span: &[i16], sample_rate: u32, axis: Axis, margin_ms: u32) -> Option<Cut> {
    let frame_samples = (sample_rate as usize * FRAME_MS as usize) / 1_000;
    if frame_samples == 0 {
        return None;
    }

    let levels = frame_levels(&high_pass(span, sample_rate), frame_samples);
    if levels.is_empty() {
        return None;
    }

    let frames = levels.len() as i64;
    let mean_level = levels.iter().sum::<f64>() / frames as f64;
    let max_level = levels.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let floor_level = floor_percentile(&levels);
    let axis_level = axis.k * mean_level + axis.c;

    if axis_level <= 0.0 {
        // Eine nicht positive Achse kann keinen Mangel tragen.
        return Some(forced(
            &levels,
            frame_samples,
            mean_level,
            floor_level,
            max_level,
            axis_level,
        ));
    }

    // Kandidaten: größte zusammenhängende Läufe **strikt** unter der Achse.
    // Der beste ist die größte Fläche; bei Gleichstand der früheste.
    let mut best_quality = 0.0;
    let mut best_weighted = 0.0;
    let mut best_start = -1;
    let mut best_end = -1;
    let mut index = 0;
    while index < frames {
        if levels[index as usize] >= axis_level {
            index += 1;
            continue;
        }
        let mut end = index;
        let mut quality = 0.0;
        let mut weighted = 0.0;
        while end < frames && levels[end as usize] < axis_level {
            let deficit = 1.0 - levels[end as usize] / axis_level;
            quality += deficit;
            weighted += deficit * end as f64;
            end += 1;
        }
        if quality > best_quality {
            best_quality = quality;
            best_weighted = weighted;
            best_start = index;
            best_end = end;
        }
        index = end;
    }

    if best_start < 0 {
        return Some(forced(
            &levels,
            frame_samples,
            mean_level,
            floor_level,
            max_level,
            axis_level,
        ));
    }

    let centroid_frame = best_weighted / best_quality;
    let centroid_sample = frame_centre(round_half_up(centroid_frame), frame_samples);

    // Der Rand hält den Schnitt von den Achsenkanten des Kandidaten fern. Ein
    // Kandidat, der zu kurz für den Rand ist, wird trotzdem geschnitten — in
    // seiner Mitte; der Rand ist eine Vorliebe, keine Ablehnung.
    let margin_frames = i64::from(margin_ms / FRAME_MS);
    let low = best_start + margin_frames;
    let high = best_end - 1 - margin_frames;
    let cut_frame = if low > high {
        (best_start + best_end) / 2
    } else {
        round_half_up(centroid_frame).clamp(low, high)
    };

    Some(Cut {
        mean_level,
        floor_level,
        max_level,
        axis: axis_level,
        quality: best_quality,
        centroid_sample,
        cut_sample: frame_centre(cut_frame, frame_samples),
        candidate: true,
    })
}

/// Der Notweg ohne Kandidaten: der leiseste Rahmen (bei Gleichstand der
/// früheste), `Q = 0`.
fn forced(
    levels: &[f64],
    frame_samples: usize,
    mean_level: f64,
    floor_level: f64,
    max_level: f64,
    axis_level: f64,
) -> Cut {
    let mut quietest = 0;
    for (frame, &level) in levels.iter().enumerate() {
        if level < levels[quietest] {
            quietest = frame;
        }
    }
    let offset = frame_centre(quietest as i64, frame_samples);
    Cut {
        mean_level,
        floor_level,
        max_level,
        axis: axis_level,
        quality: 0.0,
        centroid_sample: offset,
        cut_sample: offset,
        candidate: false,
    }
}

/// Die Sample-Nummer der Rahmenmitte.
fn frame_centre(frame: i64, frame_samples: usize) -> usize {
    (frame * frame_samples as i64 + (frame_samples / 2) as i64) as usize
}

/// Der P5-artige Bodenwert: das 5. Perzentil der sortierten Rahmenpegel.
fn floor_percentile(levels: &[f64]) -> f64 {
    let mut sorted = levels.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let count = sorted.len() as i64;
    let raw = (count as f64 * FLOOR_PERCENTILE / 100.0).ceil() as i64 - 1;
    sorted[raw.clamp(0, count - 1) as usize]
}

/// Kaufmännisch runden: `(x + 0.5).floor()` — Kotlins `roundToInt`.
///
/// Nicht Rusts `f64::round`, das von der Null weg rundet: hier sind alle Werte
/// nicht negativ, aber die Formel ist die Wahrheit, nicht die Koinzidenz.
fn round_half_up(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}
