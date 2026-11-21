//! Die Lieferung: constrained Markdown → semantisches Fragment und Reintext.
//!
//! [`deliver`] ist eine **reine Funktion**: kein Zwischenablage-Zugriff, kein
//! Fenster, kein Netz, keine Plattform. Sie bekommt das polierte Transkript als
//! Markdown und gibt die beiden Nutzlasten zurück, die eine Plattformschale in
//! die Systemzwischenablage schreibt (`docs/ai/DOMAIN.md`, „Lieferung").
//!
//! # Die vier erlaubten Konstrukte
//!
//! Absatz, Überschrift `h1`–`h6`, ungeordnete Liste und fett. Alles andere ist
//! ein typisierter Fehler ([`DeliveryError`]) — keine stille Reparatur und kein
//! Durchreichen von HTML. **Geordnete Listen** sind erlaubt, aber sie sind
//! Text: jeder Eintrag wird ein eigener Absatz mit seiner Ziffer, nie ein
//! `<ol>` — deutsche Rubrum-Nummerierung ist Text, keine Struktur.
//!
//! ```
//! use diktatwerkstatt_core::deliver;
//!
//! let delivery = deliver("## Betreff\n\nSehr geehrte **Damen** und Herren.").unwrap();
//! assert_eq!(delivery.html, "<h2>Betreff</h2>\n<p>Sehr geehrte <strong>Damen</strong> und Herren.</p>");
//! assert_eq!(delivery.text, "Betreff\nSehr geehrte Damen und Herren.");
//! ```
//!
//! # Warum der Umschlag fehlt
//!
//! `html` ist das **nackte** Fragment. Den `CF_HTML`-Umschlag (ASCII-Kopfzeile,
//! Byte-Offsets, Zehnstellig-Nullfüllung) baut der Windows-Schreiber (#8/#16,
//! Entscheidung E10); Linux (`text/html`) und macOS (`public.html`) nehmen das
//! Fragment so, wie es hier entsteht. Die Offsets zählen dort **Bytes**, nicht
//! Zeichen — deshalb bleibt Umlaut-Text hier byte-gleich und wird nirgends
//! umkodiert.

use std::fmt;
use std::result::Result as StdResult;

/// Was die Zwischenablage bekommt: das Fragment und derselbe Inhalt als Text.
///
/// Beide Formate gehen **zusammen** in die Zwischenablage; reiner Text
/// transportiert keine Struktur, HTML allein lässt textonly-Ziele leer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    /// Das semantische HTML-Fragment — Absatz, Überschrift, ungeordnete Liste,
    /// `strong`. Ohne `CF_HTML`-Umschlag, ohne CSS, ohne Klassen.
    pub html: String,
    /// Derselbe Inhalt ohne Markdown, Blöcke mit `\n` getrennt, geordnete
    /// Listen als `1. …`. Kein abschließender Zeilenumbruch.
    pub text: String,
}

/// Ein Konstrukt, das die Lieferung nicht kennt.
///
/// Die Unterscheidung existiert, damit der Fehler sagt, **was** zu viel war —
/// der Nutzer sieht „Zeile 4: eine Tabelle …" statt eines Sammelfehlers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Construct {
    /// Eine Tabelle (Pipe-Tabelle mit Kopfzeile, mit oder ohne Randpipes).
    Table,
    /// Ein Codeblock — eingezäunt, eingerückt oder als `code` im Text.
    Code,
    /// Ein Bild, `![…](…)`.
    Image,
    /// Ein Link — `[…](…)`, `[…][…]` oder ein Autolink.
    Link,
    /// Kursiver Text: ein einzelnes `*` oder `_`.
    Emphasis,
    /// Rohes HTML oder eine HTML-Entity (`<b>`, `&amp;`).
    Html,
    /// Ein Blockzitat, `> …`.
    BlockQuote,
    /// Eine Trennlinie, `---`.
    ThematicBreak,
    /// Eine eingerückte Liste — die Lieferung kennt keine Verschachtelung.
    NestedList,
    /// Eine Überschrift, die nicht `# …` folgt (Ebene 7 oder ohne Leerzeichen).
    Heading,
}

impl fmt::Display for Construct {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Table => "eine Tabelle",
            Self::Code => "ein Codeblock",
            Self::Image => "ein Bild",
            Self::Link => "ein Link",
            Self::Emphasis => "kursiver Text",
            Self::Html => "rohes HTML",
            Self::BlockQuote => "ein Blockzitat",
            Self::ThematicBreak => "eine Trennlinie",
            Self::NestedList => "eine verschachtelte Liste",
            Self::Heading => "eine Überschrift dieser Form",
        };
        formatter.write_str(name)
    }
}

/// Warum aus Markdown keine Lieferung wurde.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DeliveryError {
    /// Ein Konstrukt außerhalb der vier erlaubten.
    #[error(
        "Zeile {line}: {construct} ist in der Lieferung nicht erlaubt \
         (nur Absatz, Überschrift, ungeordnete Liste und fett)"
    )]
    Unsupported {
        /// Die 1-basierte Zeile im Markdown.
        line: usize,
        /// Was dort stand.
        construct: Construct,
    },

    /// Eine Betonung ist nicht geschlossen — `**` ohne Gegenstück.
    #[error("Zeile {line}: {delimiter} findet kein schließendes Gegenstück")]
    Unbalanced {
        /// Die 1-basierte Zeile im Markdown.
        line: usize,
        /// Der offene Delimiter, `**` oder `__`.
        delimiter: &'static str,
    },
}

/// Übersetzt constrained Markdown in die beiden Zwischenablage-Nutzlasten.
///
/// Die Grammatik ist absichtlich klein und streng (`docs/ai/DOMAIN.md`,
/// „Lieferung"):
///
/// - **Überschrift:** `#` bis `######`, danach ein Leerzeichen.
/// - **Ungeordnete Liste:** `-`, `*` oder `+`, danach ein Leerzeichen; je eine
///   Zeile ein Eintrag. Keine Verschachtelung.
/// - **Geordnete Liste:** `1.` oder `1)` — bleibt Text mit ihrer Ziffer.
/// - **Absatz:** alles andere; Zeilen eines Absatzes werden mit einem
///   Leerzeichen verbunden.
/// - **Fett:** `**…**` oder `__…__`, nicht verschachtelt.
///
/// Alles andere ist ein [`DeliveryError`]. Leerer Text ergibt eine leere,
/// gültige Nutzlast.
pub fn deliver(markdown: &str) -> StdResult<Delivery, DeliveryError> {
    let lines: Vec<&str> = markdown
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();

    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let trimmed = lines[index].trim_end();
        if trimmed.trim().is_empty() {
            index += 1;
            continue;
        }

        let number = index + 1;
        let (indent, rest) = split_indent(trimmed);

        if indent >= 4 || rest.starts_with('\t') || is_fence(rest) {
            return Err(unsupported(number, Construct::Code));
        }
        if is_table(rest) || is_table_header(rest, lines.get(index + 1).copied()) {
            return Err(unsupported(number, Construct::Table));
        }
        if is_thematic_break(rest) {
            return Err(unsupported(number, Construct::ThematicBreak));
        }
        if rest.starts_with('>') {
            return Err(unsupported(number, Construct::BlockQuote));
        }
        if let Some(heading) = parse_atx(rest) {
            let (level, content) = heading.map_err(|construct| unsupported(number, construct))?;
            let (html, text) = render_inline(content, number)?;
            blocks.push((format!("<h{level}>{html}</h{level}>"), text));
            index += 1;
            continue;
        }
        if let Some((kind, _, _)) = parse_list_item(rest) {
            if indent > 0 {
                return Err(unsupported(number, Construct::NestedList));
            }
            index = match kind {
                Kind::Unordered => unordered_list(&lines, index, &mut blocks)?,
                Kind::Ordered => ordered_list(&lines, index, &mut blocks)?,
            };
            continue;
        }

        index = paragraph(&lines, index, &mut blocks)?;
    }

    let html = blocks
        .iter()
        .map(|(html, _)| html.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let text = blocks
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(Delivery { html, text })
}

/// Eine ungeordnete Liste: ein `<ul>` mit einem `<li>` je Zeile, im Reintext
/// eine Zeile je Eintrag.
fn unordered_list(
    lines: &[&str],
    mut index: usize,
    blocks: &mut Vec<(String, String)>,
) -> StdResult<usize, DeliveryError> {
    let mut html = String::from("<ul>");
    let mut text: Vec<String> = Vec::new();

    while index < lines.len() {
        let candidate = lines[index].trim_end();
        let (indent, rest) = split_indent(candidate);
        let Some((kind, _, content)) = parse_list_item(rest) else {
            break;
        };
        if kind != Kind::Unordered {
            break;
        }
        if indent > 0 {
            return Err(unsupported(index + 1, Construct::NestedList));
        }
        let (item_html, item_text) = render_inline(content, index + 1)?;
        html.push_str("\n<li>");
        html.push_str(&item_html);
        html.push_str("</li>");
        text.push(item_text);
        index += 1;
    }

    html.push_str("\n</ul>");
    blocks.push((html, text.join("\n")));
    Ok(index)
}

/// Eine geordnete Liste: **kein** `<ol>`. Jeder Eintrag wird ein eigener Absatz
/// mit seinem Ziffernmarker, im Reintext eine Zeile.
fn ordered_list(
    lines: &[&str],
    mut index: usize,
    blocks: &mut Vec<(String, String)>,
) -> StdResult<usize, DeliveryError> {
    while index < lines.len() {
        let candidate = lines[index].trim_end();
        let (indent, rest) = split_indent(candidate);
        let Some((kind, marker, content)) = parse_list_item(rest) else {
            break;
        };
        if kind != Kind::Ordered {
            break;
        }
        if indent > 0 {
            return Err(unsupported(index + 1, Construct::NestedList));
        }
        let (item_html, item_text) = render_inline(content, index + 1)?;
        let html = if item_html.is_empty() {
            format!("<p>{marker}</p>")
        } else {
            format!("<p>{marker} {item_html}</p>")
        };
        let text = if item_text.is_empty() {
            marker.to_string()
        } else {
            format!("{marker} {item_text}")
        };
        blocks.push((html, text));
        index += 1;
    }
    Ok(index)
}

/// Ein Absatz: aufeinanderfolgende Zeilen bis zur Leerzeile oder zum nächsten
/// Block, mit einem Leerzeichen verbunden.
fn paragraph(
    lines: &[&str],
    mut index: usize,
    blocks: &mut Vec<(String, String)>,
) -> StdResult<usize, DeliveryError> {
    let mut html: Vec<String> = Vec::new();
    let mut text: Vec<String> = Vec::new();

    while index < lines.len() {
        let candidate = lines[index].trim_end();
        if candidate.trim().is_empty() {
            break;
        }
        let (indent, rest) = split_indent(candidate);
        if !html.is_empty() && starts_block(indent, rest) {
            break;
        }
        if indent >= 4 || rest.starts_with('\t') {
            return Err(unsupported(index + 1, Construct::Code));
        }
        let (line_html, line_text) = render_inline(rest.trim(), index + 1)?;
        html.push(line_html);
        text.push(line_text);
        index += 1;
    }

    blocks.push((format!("<p>{}</p>", html.join(" ")), text.join(" ")));
    Ok(index)
}

/// Beginnt diese Zeile einen Block, der einen Absatz unterbricht?
fn starts_block(indent: usize, rest: &str) -> bool {
    indent >= 4
        || rest.starts_with('\t')
        || is_fence(rest)
        || is_table(rest)
        || is_thematic_break(rest)
        || rest.starts_with('>')
        || parse_atx(rest).is_some()
        || parse_list_item(rest).is_some()
}

/// Art einer Liste; die Ziffern einer geordneten Liste bleiben Text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Unordered,
    Ordered,
}

/// Liest einen Listenmarker: Art, wörtlicher Marker und Inhalt.
///
/// `- x` → `(Unordered, "", "x")`; `1. x` → `(Ordered, "1.", "x")`.
fn parse_list_item(rest: &str) -> Option<(Kind, &str, &str)> {
    let first = rest.chars().next()?;

    if matches!(first, '-' | '*' | '+') {
        let after = &rest[1..];
        let content = if after.is_empty() {
            ""
        } else {
            after.strip_prefix(' ')?.trim_start()
        };
        return Some((Kind::Unordered, "", content));
    }

    if first.is_ascii_digit() {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 9 {
            return None;
        }
        let after_digits = &rest[digits..];
        let closing = match after_digits.chars().next()? {
            '.' | ')' => 1,
            _ => return None,
        };
        let marker = &rest[..digits + closing];
        let after_marker = &after_digits[closing..];
        let content = if after_marker.is_empty() {
            ""
        } else {
            after_marker.strip_prefix(' ')?.trim_start()
        };
        return Some((Kind::Ordered, marker, content));
    }

    None
}

/// Liest `# …` als Überschrift; `None` heißt: keine Überschrift.
///
/// Ebene 7 oder eine Raute ohne Leerzeichen ist ein Fehler, kein Absatz: still
/// als Text durchzureichen wäre eine Reparatur.
fn parse_atx(rest: &str) -> Option<StdResult<(usize, &str), Construct>> {
    let hashes = rest.chars().take_while(|&ch| ch == '#').count();
    if hashes == 0 {
        return None;
    }
    if hashes > 6 {
        return Some(Err(Construct::Heading));
    }
    let after = &rest[hashes..];
    if after.is_empty() {
        return Some(Ok((hashes, "")));
    }
    let Some(content) = after.strip_prefix(' ') else {
        return Some(Err(Construct::Heading));
    };
    Some(Ok((hashes, strip_closing_sequence(content))))
}

/// Entfernt die optionale schließende Rautenfolge: `## Titel ##` → `Titel`.
fn strip_closing_sequence(content: &str) -> &str {
    let trimmed = content.trim_end();
    let without_hashes = trimmed.trim_end_matches('#');
    if without_hashes.len() == trimmed.len() {
        return trimmed;
    }
    if without_hashes.is_empty() {
        return "";
    }
    match without_hashes.strip_suffix(' ') {
        Some(stem) => stem.trim_end(),
        None => trimmed,
    }
}

/// Setzt Inline-Inhalt in HTML und Reintext um: nur `strong` und Text.
fn render_inline(content: &str, line: usize) -> StdResult<(String, String), DeliveryError> {
    let mut html = String::new();
    let mut text = String::new();
    let mut rest = content;

    while let Some(ch) = rest.chars().next() {
        match ch {
            '*' | '_' => {
                let delimiter = if ch == '*' { "**" } else { "__" };
                let Some(after) = rest.strip_prefix(delimiter) else {
                    return Err(unsupported(line, Construct::Emphasis));
                };
                let Some(close) = after.find(delimiter) else {
                    return Err(DeliveryError::Unbalanced { line, delimiter });
                };
                let inner = &after[..close];
                if inner.trim().is_empty() || inner.starts_with(' ') || inner.ends_with(' ') {
                    return Err(DeliveryError::Unbalanced { line, delimiter });
                }
                if let Some(construct) = first_forbidden(inner) {
                    return Err(unsupported(line, construct));
                }
                html.push_str("<strong>");
                push_escaped(&mut html, inner);
                html.push_str("</strong>");
                text.push_str(inner);
                rest = &after[close + delimiter.len()..];
            }
            '`' => return Err(unsupported(line, Construct::Code)),
            '!' if rest.starts_with("![") => return Err(unsupported(line, Construct::Image)),
            '[' if is_link(&rest[1..]) => return Err(unsupported(line, Construct::Link)),
            '<' => {
                let construct = if is_autolink(rest) {
                    Construct::Link
                } else {
                    Construct::Html
                };
                return Err(unsupported(line, construct));
            }
            '&' if is_entity(&rest[1..]) => return Err(unsupported(line, Construct::Html)),
            _ => {
                push_escaped_char(&mut html, ch);
                text.push(ch);
                rest = &rest[ch.len_utf8()..];
            }
        }
    }

    Ok((html, text))
}

/// Findet das erste verbotene Konstrukt in einem Textstück — für den Inhalt
/// einer fetten Stelle, die selbst kein Markup mehr tragen darf.
fn first_forbidden(inner: &str) -> Option<Construct> {
    let mut rest = inner;
    while let Some(ch) = rest.chars().next() {
        match ch {
            '*' | '_' => return Some(Construct::Emphasis),
            '`' => return Some(Construct::Code),
            '!' if rest.starts_with("![") => return Some(Construct::Image),
            '[' if is_link(&rest[1..]) => return Some(Construct::Link),
            '<' => return Some(Construct::Html),
            '&' if is_entity(&rest[1..]) => return Some(Construct::Html),
            _ => {}
        }
        rest = &rest[ch.len_utf8()..];
    }
    None
}

/// Steht nach einem `[` ein Linkziel? `[1]` ohne Ziel bleibt Text.
fn is_link(after_bracket: &str) -> bool {
    after_bracket.contains("](") || after_bracket.contains("][")
}

/// Ein Autolink `<https://…>` ist ein Link, alles andere mit `<` ist HTML.
fn is_autolink(rest: &str) -> bool {
    let Some(end) = rest.find('>') else {
        return false;
    };
    let inner = &rest[1..end];
    let schemes = ["http://", "https://", "ftp://", "mailto:"];
    schemes.iter().any(|scheme| inner.starts_with(scheme))
        || (inner.contains('@') && inner.contains('.'))
}

/// Steht nach einem `&` eine HTML-Entity? `R&D` bleibt Text.
fn is_entity(after_ampersand: &str) -> bool {
    let Some(end) = after_ampersand.find(';') else {
        return false;
    };
    let name = &after_ampersand[..end];
    (2..=32).contains(&name.len())
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '#')
}

/// Maskiert Text für HTML. `<` erreicht diese Stelle nie — es ist ein Fehler.
fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        push_escaped_char(out, ch);
    }
}

fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '>' => out.push_str("&gt;"),
        '<' => out.push_str("&lt;"),
        _ => out.push(ch),
    }
}

/// Anzahl der führenden Leerzeichen und der Rest der Zeile.
fn split_indent(line: &str) -> (usize, &str) {
    let indent = line.len() - line.trim_start_matches(' ').len();
    (indent, &line[indent..])
}

/// Zählt ein Codeblock-Zaun (```` ``` ```` oder `~~~`).
fn is_fence(rest: &str) -> bool {
    rest.starts_with("```") || rest.starts_with("~~~")
}

/// Eine Pipe-Tabelle beginnt mit `|`.
fn is_table(rest: &str) -> bool {
    rest.starts_with('|')
}

/// Eine GFM-Tabelle ohne Randpipes erkennt man an der Trennzeile darunter.
fn is_table_header(rest: &str, next: Option<&str>) -> bool {
    rest.contains('|') && next.is_some_and(is_table_delimiter)
}

/// `--- | ---` — nur Pipes, Bindestriche, Doppelpunkte und Leerraum, aber
/// mindestens je eine Pipe und ein Bindestrich.
fn is_table_delimiter(line: &str) -> bool {
    let mut pipes = 0;
    let mut dashes = 0;
    for ch in line.trim().chars() {
        match ch {
            '|' => pipes += 1,
            '-' => dashes += 1,
            ':' | ' ' | '\t' => {}
            _ => return false,
        }
    }
    pipes >= 1 && dashes >= 1
}

/// `---`, `***`, `___` — dieselbe Zierde in drei Zeichen, mit oder ohne
/// Zwischenräume.
fn is_thematic_break(rest: &str) -> bool {
    for decoration in ['-', '*', '_'] {
        let mut count = 0;
        let mut only_decoration = true;
        for ch in rest.chars() {
            if ch == decoration {
                count += 1;
            } else if ch != ' ' && ch != '\t' {
                only_decoration = false;
                break;
            }
        }
        if only_decoration && count >= 3 {
            return true;
        }
    }
    false
}

fn unsupported(line: usize, construct: Construct) -> DeliveryError {
    DeliveryError::Unsupported { line, construct }
}
