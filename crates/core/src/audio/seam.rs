//! Die Naht an einer Fragmentgrenze: Policies, Pre-roll, Prompt und Echo-Trim.
//!
//! Drei Maßnahmen hängen an jeder Naht und alle an der Transkription des
//! Vorfragments:
//!
//! - **Pre-roll** — die letzte Sekunde des Vorfragments wird dem Upload
//!   vorangestellt, damit der Dienst die Wörter beiderseits des Schnitts hört.
//! - **Prompt-Carry-over** — ein nahtlokaler Prompt aus dem Ende des
//!   Vorfragtranskripts (≤ 96 Zeichen, an einer Wortgrenze) gibt dem Modell
//!   Schreibweise und Kontext der letzten Wörter.
//! - **Echo-Trim** — der Kopf des neuen Transkripts wird gegen das Ende des
//!   Vorfragtranskripts geprüft (≤ 8 Token, das erste Token muss passen, ab
//!   zwei Token ist ein Fehlgriff erlaubt); ein Treffer wird entfernt, damit
//!   der Pre-roll keine doppelten Wörter erzeugt.
//!
//! Ob eine Naht überhaupt konditioniert wird, entscheidet die
//! [`SeamPolicy`] — im Produktivbetrieb das Q-Gate: der Schnitt ist
//! best-effort, die Naht ist die Garantie. Ein wortfinaler Frikativ, der den
//! Schnitt falsch legt, wird durch den Pre-roll aufgefangen; ein solider
//! Stillenschnitt braucht ihn nicht.

/// Länge des Pre-rolls in Millisekunden.
pub const PRE_ROLL_MS: u32 = 1_000;

/// Zeichenbudget des Naht-Prompts.
pub const PROMPT_MAX_CHARS: usize = 96;

/// Höchstzahl Token, die der Echo-Trim entfernen darf.
pub const ECHO_MAX_TOKENS: usize = 8;

/// Mindest-Q, ab dem eine Naht unter dem Q-Gate **kahl** bleibt.
///
/// In Rahmen-Einheiten: `Q` ist die Mangel-Fläche in 10-ms-Rahmen, ein solider
/// Stillenschnitt liegt bei rund 40, ein durch die Achse gefädelter
/// Flüsterkandidat bei rund 2.
pub const QUALITY_MIN: f64 = 10.0;

/// Wie die Naht an einer Fragmentgrenze behandelt wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeamPolicy {
    /// Jede Naht wird konditioniert (das Verhalten vor dem Q-Gate).
    All,
    /// Nur erzwungene Schnitte werden konditioniert; ein gefundener Kandidat
    /// bleibt kahl.
    ForcedOnly,
    /// Nie konditionieren — kein Pre-roll, kein Prompt.
    Off,
    /// Das Q-Gate: konditioniert wird, solange die Qualität des Schnitts
    /// **unter** der Schwelle liegt. Der Produktionsdefault.
    #[default]
    QualityGate,
}

/// Konditioniert die Naht an einem Fragmentanfang?
///
/// - [`SeamPolicy::QualityGate`] — `quality` fehlt (nicht gemessen, etwa beim
///   Rest) oder liegt **strikt** unter `quality_min`. `Q == quality_min` ist
///   gut genug und bleibt kahl; `None` konditioniert — die konservative
///   Richtung.
/// - [`SeamPolicy::ForcedOnly`] — `quality` fehlt oder der Schnitt war
///   erzwungen.
/// - [`SeamPolicy::All`] — immer, [`SeamPolicy::Off`] — nie.
pub fn stitched(policy: SeamPolicy, quality: Option<f64>, forced: bool, quality_min: f64) -> bool {
    match policy {
        SeamPolicy::All => true,
        SeamPolicy::Off => false,
        SeamPolicy::ForcedOnly => quality.is_none() || forced,
        SeamPolicy::QualityGate => quality.is_none_or(|q| q < quality_min),
    }
}

/// Die **letzten** `pre_roll_samples` von `previous`, geklemmt an den Anfang
/// des Streams.
///
/// Ist das Vorfragment kürzer als der Pre-roll, ist der Pre-roll das ganze
/// Vorfragment; `0` ergibt eine leere Scheibe.
pub fn pre_roll(previous: &[i16], pre_roll_samples: usize) -> &[i16] {
    &previous[previous.len().saturating_sub(pre_roll_samples)..]
}

/// Der Upload eines Fragments: mit Pre-roll, wenn die Naht konditioniert ist,
/// sonst das Fragment allein.
pub fn upload(
    previous: &[i16],
    fragment: &[i16],
    stitched: bool,
    pre_roll_samples: usize,
) -> Vec<i16> {
    if !stitched {
        return fragment.to_vec();
    }
    let mut audio = pre_roll(previous, pre_roll_samples).to_vec();
    audio.extend_from_slice(fragment);
    audio
}

/// Der Naht-Prompt: das Ende des Vorfragtranskripts, an einer Wortgrenze
/// abgeschnitten.
///
/// Gezählt werden **Zeichen**, nicht Bytes. Der Text wird getrimmt; leer ergibt
/// `None`. Passt er ins Budget, kommt er ganz zurück. Sonst werden die letzten
/// `max_chars` Zeichen genommen und beim **ersten** Leerzeichen abgeschnitten
/// (nur wenn es nicht an Position 0 liegt), damit kein halbes Wort am Anfang
/// steht; danach erneut getrimmt. Bleibt nichts übrig, `None`.
pub fn prompt(previous_text: &str, max_chars: usize) -> Option<String> {
    let text = previous_text.trim();
    if text.is_empty() {
        return None;
    }

    let count = text.chars().count();
    if count <= max_chars {
        return Some(text.to_string());
    }

    let tail: String = text.chars().skip(count - max_chars).collect();
    let tail = match tail.find(' ') {
        Some(space) if space > 0 => tail[space + 1..].to_string(),
        _ => tail,
    };
    let tail = tail.trim();
    if tail.is_empty() {
        None
    } else {
        Some(tail.to_string())
    }
}

/// Entfernt einen Transkriptkopf, der das Ende des Vorfragtranskripts
/// wiederholt.
///
/// Token sind `split_whitespace`-Wörter. Geprüft werden Kandidatengrößen von
/// `min(max_tokens, Token, Vorgänger)` abwärts. Das **erste** Token muss
/// case-insensitiv passen; ab zwei Token ist **ein** Fehlgriff erlaubt. Ein
/// Treffer liefert den Rest (mit einem Leerzeichen verbunden) und die Zahl der
/// entfernten Token; ohne Treffer bleibt `text` unverändert. Leerer Text oder
/// leerer Vorgänger ergibt `(text, 0)`.
pub fn trim_echo(text: &str, previous_text: &str, max_tokens: usize) -> (String, usize) {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let previous: Vec<&str> = previous_text.split_whitespace().collect();
    if tokens.is_empty() || previous.is_empty() {
        return (text.to_string(), 0);
    }

    let limit = max_tokens.min(tokens.len()).min(previous.len());
    for size in (1..=limit).rev() {
        let head = &tokens[..size];
        let tail = &previous[previous.len() - size..];
        if head[0].to_lowercase() != tail[0].to_lowercase() {
            continue;
        }
        let mismatches = head
            .iter()
            .zip(tail)
            .filter(|(left, right)| left.to_lowercase() != right.to_lowercase())
            .count();
        if mismatches == 0 || (size >= 2 && mismatches <= 1) {
            return (tokens[size..].join(" "), size);
        }
    }
    (text.to_string(), 0)
}

/// Länge der längsten Tokenfolge, die Suffix von `previous` und Präfix von
/// `next` ist; `0` ohne gemeinsamen Lauf.
///
/// Token sind Folgen aus Buchstaben und Ziffern (`[\p{L}\p{N}]+`), klein
/// geschrieben — Satzzeichen und Groß-/Kleinschreibung verschieben das Maß
/// nicht. Ein Wert über 0 an einer Naht ist der Fingerabdruck eines Wortes, das
/// beide Fragmente transkribiert haben: die Evidenz, kein Steuerpfad.
pub fn overlap(previous: &str, next: &str) -> usize {
    let left = word_tokens(previous);
    let right = word_tokens(next);
    let limit = left.len().min(right.len());
    for size in (1..=limit).rev() {
        if left[left.len() - size..] == right[..size] {
            return size;
        }
    }
    0
}

/// Wörter aus Buchstaben und Ziffern, klein geschrieben.
fn word_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
