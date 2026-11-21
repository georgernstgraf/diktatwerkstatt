//! Tests der Lieferung: constrained Markdown → semantisches Fragment und
//! Reintext.
//!
//! Die neun Fälle aus #10, dazu die Grenzen, an denen der Parser streng sein
//! muss. Die `CF_HTML`-Offset-Tests stehen seit E10 beim Windows-Schreiber
//! (#8/#16); hier bleibt der Gegenbeweis, dass der Kern **keinen** Umschlag
//! baut.

use diktatwerkstatt_core::{Construct, Delivery, DeliveryError, deliver};

fn fragment(markdown: &str) -> Delivery {
    deliver(markdown).expect("gültiges constrained Markdown")
}

fn construct(markdown: &str) -> Construct {
    match deliver(markdown) {
        Ok(delivery) => panic!("erwartet: Fehler, bekommen: {delivery:?}"),
        Err(DeliveryError::Unsupported { construct, .. }) => construct,
        Err(error) => panic!("erwartet: nicht erlaubtes Konstrukt, bekommen: {error:?}"),
    }
}

// ---------------------------------------------------------------------------
// Die neun Fälle aus #10
// ---------------------------------------------------------------------------

#[test]
fn umlaute_gehen_byte_gleich_durch() {
    let text = "Grüße aus der Werkstatt: Ärger, Öl, Übermaß, Straße — zählt.";
    let delivery = fragment(text);

    assert_eq!(delivery.text, text);
    assert_eq!(delivery.html, format!("<p>{text}</p>"));
    // Der Unterschied, den der Windows-Schreiber rechnen muss: die Nutzlast hat
    // mehr Bytes als Zeichen, Offsets zählen Bytes.
    assert!(delivery.html.len() > delivery.html.chars().count());
}

#[test]
fn ueberschriften_werden_zu_h1_bis_h6() {
    let delivery =
        fragment("# eins\n\n## zwei\n\n### drei\n\n#### vier\n\n##### fünf\n\n###### sechs");

    for (level, word) in [
        (1, "eins"),
        (2, "zwei"),
        (3, "drei"),
        (4, "vier"),
        (5, "fünf"),
        (6, "sechs"),
    ] {
        assert!(
            delivery
                .html
                .contains(&format!("<h{level}>{word}</h{level}>")),
            "Ebene {level} fehlt im Fragment: {:?}",
            delivery.html
        );
    }
    assert_eq!(delivery.text, "eins\nzwei\ndrei\nvier\nfünf\nsechs");
}

#[test]
fn ungeordnete_liste_wird_zu_ul_und_li() {
    let delivery = fragment("Einkauf:\n\n- Milch\n- Brot\n- Äpfel");

    assert_eq!(
        delivery.html,
        "<p>Einkauf:</p>\n<ul>\n<li>Milch</li>\n<li>Brot</li>\n<li>Äpfel</li>\n</ul>"
    );
    assert_eq!(delivery.text, "Einkauf:\nMilch\nBrot\nÄpfel");
    assert!(!delivery.html.contains("<ol"));
}

#[test]
fn fett_wird_zu_strong() {
    let delivery = fragment("Das ist **wichtig** und __auch__ fett.");

    assert_eq!(
        delivery.html,
        "<p>Das ist <strong>wichtig</strong> und <strong>auch</strong> fett.</p>"
    );
    assert_eq!(delivery.text, "Das ist wichtig und auch fett.");
}

#[test]
fn geordnete_liste_wird_text_mit_ihren_ziffern() {
    let delivery = fragment("1. Antragsteller\n2. Sachverhalt\n3) Rechtliches");

    assert!(
        !delivery.html.contains("<ol"),
        "eine geordnete Liste darf kein <ol> werden: {:?}",
        delivery.html
    );
    assert_eq!(
        delivery.html,
        "<p>1. Antragsteller</p>\n<p>2. Sachverhalt</p>\n<p>3) Rechtliches</p>"
    );
    assert_eq!(
        delivery.text,
        "1. Antragsteller\n2. Sachverhalt\n3) Rechtliches"
    );
}

#[test]
fn nicht_erlaubte_konstrukte_sind_typisierte_fehler() {
    // Tabelle mit Randpipes …
    assert_eq!(construct("| a | b |\n| --- | --- |"), Construct::Table);
    // … und ohne.
    assert_eq!(
        construct("Spalte | Spalte\n--- | ---\n1 | 2"),
        Construct::Table
    );
    // Codeblock, eingezäunt und eingerückt, dazu `code` im Text.
    assert_eq!(construct("```\nlet x = 1;\n```"), Construct::Code);
    assert_eq!(construct("    eingerückt"), Construct::Code);
    assert_eq!(construct("Ein `Wort` im Text"), Construct::Code);
    // Bild.
    assert_eq!(construct("![Ein Bild](bild.png)"), Construct::Image);
    // Link, mit und ohne Autolink.
    assert_eq!(construct("[Text](https://example.com)"), Construct::Link);
    assert_eq!(construct("Siehe <https://example.com>"), Construct::Link);
    // Kursiv.
    assert_eq!(construct("Das ist *kursiv*."), Construct::Emphasis);
    assert_eq!(construct("Das ist _kursiv_."), Construct::Emphasis);
    // Roh-HTML und HTML-Entity.
    assert_eq!(construct("<b>fett</b>"), Construct::Html);
    assert_eq!(construct("Firma &amp; Co"), Construct::Html);
    // Blockzitat, Trennlinie, verschachtelte Liste, Überschrift über Ebene 6.
    assert_eq!(construct("> Ein Zitat"), Construct::BlockQuote);
    assert_eq!(construct("---"), Construct::ThematicBreak);
    assert_eq!(construct("- eins\n  - zwei"), Construct::NestedList);
    assert_eq!(construct("####### sieben"), Construct::Heading);
}

#[test]
fn leerer_text_ergibt_eine_leere_gueltige_nutzlast() {
    let empty = Delivery {
        html: String::new(),
        text: String::new(),
    };

    assert_eq!(deliver("").expect("leer ist gültig"), empty);
    assert_eq!(
        deliver("\n \n\t\n").expect("nur Leerraum ist gültig"),
        empty
    );
}

#[test]
fn kein_abschliessender_zeilenumbruch() {
    let delivery = fragment("Ein Absatz.\n\n# Titel\n\n- Punkt\n");

    assert!(!delivery.text.ends_with('\n'), "text: {:?}", delivery.text);
    assert!(!delivery.html.ends_with('\n'), "html: {:?}", delivery.html);
}

#[test]
fn kein_cf_html_umschlag() {
    // Ersatz für die Offset-Tests: die sind seit E10 Sache des
    // Windows-Schreibers (#8/#16). Hier steht der Gegenbeweis.
    let delivery = fragment("# Titel\n\nText");

    assert!(delivery.html.starts_with("<h1>Titel</h1>"));
    assert!(!delivery.html.contains("Version:"));
    assert!(!delivery.html.contains("StartHTML"));
    assert!(!delivery.html.contains("StartFragment"));
}

// ---------------------------------------------------------------------------
// Grenzen des Parsers
// ---------------------------------------------------------------------------

#[test]
fn der_fehler_nennt_die_zeile() {
    let error = deliver("Erster Absatz.\n\n| a | b |\n| --- | --- |").unwrap_err();

    assert_eq!(
        error,
        DeliveryError::Unsupported {
            line: 3,
            construct: Construct::Table,
        }
    );
    assert!(error.to_string().starts_with("Zeile 3: eine Tabelle"));
}

#[test]
fn ungeschlossenes_fett_ist_ein_eigener_fehler() {
    assert_eq!(
        deliver("Das ist **offen").unwrap_err(),
        DeliveryError::Unbalanced {
            line: 1,
            delimiter: "**",
        }
    );
    assert_eq!(
        deliver("Und __so").unwrap_err(),
        DeliveryError::Unbalanced {
            line: 1,
            delimiter: "__",
        }
    );
}

#[test]
fn eine_leere_fette_stelle_ist_kein_fett() {
    assert!(matches!(
        deliver("Text mit ** ** Fett").unwrap_err(),
        DeliveryError::Unbalanced { .. }
    ));
}

#[test]
fn ein_minus_ohne_leerzeichen_ist_kein_listenmarker() {
    let delivery = fragment("-5 Grad sind kalt.");

    assert_eq!(delivery.text, "-5 Grad sind kalt.");
    assert_eq!(delivery.html, "<p>-5 Grad sind kalt.</p>");
}

#[test]
fn ein_abzaehliger_punkt_bleibt_text() {
    // Ein nummerierter Punkt mitten im Absatz ist eine geordnete Liste — und
    // damit Text, keine Struktur.
    let delivery = fragment("Der Antrag lautet:\n1. Der Antrag wird abgewiesen.");

    assert_eq!(
        delivery.text,
        "Der Antrag lautet:\n1. Der Antrag wird abgewiesen."
    );
    assert!(!delivery.html.contains("<ol"));
}

#[test]
fn zeilenumbruch_im_absatz_wird_ein_leerzeichen() {
    let delivery = fragment("Ein Satz\nüber zwei Zeilen.");

    assert_eq!(delivery.text, "Ein Satz über zwei Zeilen.");
    assert_eq!(delivery.html, "<p>Ein Satz über zwei Zeilen.</p>");
}

#[test]
fn mehrere_absaetze_bleiben_getrennt() {
    let delivery = fragment("Eins.\n\nZwei.\n\nDrei.");

    assert_eq!(delivery.text, "Eins.\nZwei.\nDrei.");
    assert_eq!(delivery.html, "<p>Eins.</p>\n<p>Zwei.</p>\n<p>Drei.</p>");
}

#[test]
fn crlf_aus_einer_windows_datei_wird_verstanden() {
    let delivery = fragment("# Titel\r\n\r\nText mit **fett**.\r\n");

    assert_eq!(delivery.text, "Titel\nText mit fett.");
    assert!(delivery.html.contains("<strong>fett</strong>"));
}

#[test]
fn fett_steht_auch_in_ueberschrift_und_liste() {
    let delivery = fragment("## **Fett** im Titel\n\n- **Punkt** im Text");

    assert_eq!(
        delivery.html,
        "<h2><strong>Fett</strong> im Titel</h2>\n<ul>\n<li><strong>Punkt</strong> im Text</li>\n</ul>"
    );
    assert_eq!(delivery.text, "Fett im Titel\nPunkt im Text");
}

#[test]
fn kaufmannsund_und_klammer_werden_maskiert() {
    let delivery = fragment("Umsatz > 1000 & mehr");

    assert_eq!(delivery.html, "<p>Umsatz &gt; 1000 &amp; mehr</p>");
    assert_eq!(delivery.text, "Umsatz > 1000 & mehr");
}

#[test]
fn eine_krumme_klammer_ohne_ziel_bleibt_text() {
    let delivery = fragment("Der Vermerk [1] gilt.");

    assert_eq!(delivery.text, "Der Vermerk [1] gilt.");
}

#[test]
fn eine_schliessende_rautenfolge_gehoert_nicht_zur_ueberschrift() {
    let delivery = fragment("## Titel ##");

    assert_eq!(delivery.html, "<h2>Titel</h2>");
    assert_eq!(delivery.text, "Titel");
}

#[test]
fn ein_fehlendes_konstrukt_ist_ein_kernfehler() {
    // Der Kernfehler nimmt den Lieferfehler auf, damit Aufrufer `?` schreiben.
    let error: diktatwerkstatt_core::Error = deliver("| a |").unwrap_err().into();

    assert!(matches!(error, diktatwerkstatt_core::Error::Delivery(_)));
    assert!(error.to_string().starts_with("Lieferung: Zeile 1"));
}
