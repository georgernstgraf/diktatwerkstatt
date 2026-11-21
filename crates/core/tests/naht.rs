//! Portierte Naht-Tests: Prompt, Echo-Trim, Überlappung, Q-Gate, Pre-roll.
//!
//! Wörtlich übersetzt aus `aitranscribe/tests/test_seam.py` (Prompt,
//! Echo-Trim) und `polished-recognition/.../SeamOverlapTest.kt` (Überlappung);
//! dazu die Fälle der Testmatrix für das Q-Gate und den Pre-roll.
//!
//! Nicht portiert: die OGG-/Pre-roll-Seitendateien der Referenz — der Pre-roll
//! ist hier ein Speicherbereich, keine Datei, also gibt es `writePreRoll`-Tests
//! nicht.

use diktatwerkstatt_core::{
    ECHO_MAX_TOKENS, PROMPT_MAX_CHARS, QUALITY_MIN, SeamPolicy, overlap, pre_roll, prompt,
    stitched, trim_echo, upload,
};

// ---------------------------------------------------------------------------
// Prompt-Carry-over
// ---------------------------------------------------------------------------

#[test]
fn prompt_keeps_a_short_transcript_verbatim() {
    let text = "Der Schnitt liegt in einer Pause.";
    assert_eq!(prompt(text, PROMPT_MAX_CHARS).as_deref(), Some(text));
}

#[test]
fn prompt_is_empty_for_an_empty_transcript() {
    assert_eq!(prompt("", PROMPT_MAX_CHARS), None);
    assert_eq!(prompt("   ", PROMPT_MAX_CHARS), None);
}

#[test]
fn prompt_takes_the_tail_and_cuts_at_a_word_boundary() {
    let text = format!("{}die letzten Worte der Diktate", "wort ".repeat(40));

    let tail = prompt(&text, PROMPT_MAX_CHARS).expect("ein Prompt");

    assert!(
        tail.ends_with("die letzten Worte der Diktate"),
        "der Prompt endet nicht mit dem Transkriptende: {tail:?}"
    );
    assert!(
        !tail.starts_with(' '),
        "der Prompt beginnt mit einem halben Wort: {tail:?}"
    );
    assert!(
        tail.chars().count() <= PROMPT_MAX_CHARS,
        "der Prompt ist {} Zeichen lang",
        tail.chars().count()
    );
}

#[test]
fn prompt_never_exceeds_the_character_budget() {
    let tail = prompt(&"a".repeat(500), PROMPT_MAX_CHARS).expect("ein Prompt");
    assert!(
        tail.chars().count() <= PROMPT_MAX_CHARS,
        "der Prompt ist {} Zeichen lang",
        tail.chars().count()
    );
}

#[test]
fn prompt_counts_characters_not_bytes() {
    // 200 Umlaute sind 200 Zeichen, aber 400 Bytes: das Budget ist in Zeichen.
    let text = "ä".repeat(200);
    let tail = prompt(&text, PROMPT_MAX_CHARS).expect("ein Prompt");
    assert_eq!(tail.chars().count(), PROMPT_MAX_CHARS);
}

// ---------------------------------------------------------------------------
// Echo-Trim
// ---------------------------------------------------------------------------

#[test]
fn echo_trim_removes_an_exact_repeated_head() {
    let previous = "Wir sprechen über die Naht und den Schnitt.";
    let text = "und den Schnitt. Danach geht es weiter.";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        ("Danach geht es weiter.".to_string(), 3)
    );
}

#[test]
fn echo_trim_tolerates_one_mismatch_for_multiple_tokens() {
    let previous = "Wir sprechen über die Naht und den Schnitt";
    let text = "und der Schnitt Danach geht es weiter.";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        ("Danach geht es weiter.".to_string(), 3)
    );
}

#[test]
fn echo_trim_requires_the_head_token_to_match() {
    let previous = "Wir sprechen über die Naht.";
    let text = "Schnitt Danach geht es weiter.";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        ("Schnitt Danach geht es weiter.".to_string(), 0)
    );
}

#[test]
fn echo_trim_keeps_everything_without_a_match() {
    let previous = "Wir sprechen über die Naht.";
    let text = "Ein völlig neuer Satz beginnt hier.";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        (text.to_string(), 0)
    );
}

#[test]
fn echo_trim_keeps_everything_when_nothing_was_heard_before() {
    assert_eq!(
        trim_echo("Erster Satz.", "", ECHO_MAX_TOKENS),
        ("Erster Satz.".to_string(), 0)
    );
    assert_eq!(
        trim_echo("", "Vorheriger Satz.", ECHO_MAX_TOKENS),
        (String::new(), 0)
    );
}

#[test]
fn echo_trim_never_removes_more_than_the_token_window() {
    let previous = (0..30)
        .map(|index| format!("w{index}"))
        .collect::<Vec<_>>()
        .join(" ");
    let text = format!(
        "{} neuer Inhalt",
        (22..30)
            .map(|index| format!("w{index}"))
            .collect::<Vec<_>>()
            .join(" ")
    );

    assert_eq!(
        trim_echo(&text, &previous, ECHO_MAX_TOKENS),
        ("neuer Inhalt".to_string(), ECHO_MAX_TOKENS)
    );
}

#[test]
fn echo_trim_is_case_insensitive() {
    let previous = "Die Naht und der Schnitt";
    let text = "die naht und der schnitt Danach";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        ("Danach".to_string(), 5)
    );
}

#[test]
fn echo_trim_returns_the_whole_text_when_the_echo_is_the_whole_text() {
    let previous = "und den Schnitt";
    let text = "und den Schnitt";

    assert_eq!(
        trim_echo(text, previous, ECHO_MAX_TOKENS),
        (String::new(), 3)
    );
}

// ---------------------------------------------------------------------------
// Überlappung (SeamOverlap)
// ---------------------------------------------------------------------------

#[test]
fn overlap_detects_a_repeated_run_straddling_the_seam() {
    let previous = "Der Patient klagt über Schmerzen und Fieber";
    let next = "und Fieber am Abend stärker";
    assert_eq!(overlap(previous, next), 2);
}

#[test]
fn overlap_ignores_case_and_punctuation() {
    assert_eq!(overlap("Hallo WELT!", "welt, wie gehts"), 1);
}

#[test]
fn overlap_is_zero_without_a_shared_run() {
    assert_eq!(overlap("eins zwei drei", "vier fünf"), 0);
}

#[test]
fn overlap_counts_a_whole_repetition() {
    assert_eq!(overlap("guten Tag", "guten Tag und willkommen"), 2);
}

#[test]
fn overlap_is_safe_for_a_blank_input() {
    assert_eq!(overlap("", "x"), 0);
    assert_eq!(overlap("x", "  "), 0);
}

// ---------------------------------------------------------------------------
// Q-Gate
// ---------------------------------------------------------------------------

#[test]
fn quality_gate_keeps_a_bare_seam_at_the_threshold() {
    // Q == quality_min ist gut genug: strikt kleiner konditioniert.
    assert!(!stitched(
        SeamPolicy::QualityGate,
        Some(QUALITY_MIN),
        false,
        QUALITY_MIN
    ));
    assert!(
        !stitched(SeamPolicy::QualityGate, Some(40.0), false, QUALITY_MIN),
        "ein solider Kandidat bleibt kahl"
    );
}

#[test]
fn quality_gate_stitches_just_below_the_threshold() {
    assert!(stitched(
        SeamPolicy::QualityGate,
        Some(QUALITY_MIN - 0.001),
        false,
        QUALITY_MIN
    ));
}

#[test]
fn quality_gate_stitches_an_unmeasured_cut() {
    assert!(
        stitched(SeamPolicy::QualityGate, None, false, QUALITY_MIN),
        "ein nicht gemessener Schnitt konditioniert — die konservative Richtung"
    );
}

#[test]
fn quality_gate_stitches_a_forced_cut() {
    assert!(stitched(
        SeamPolicy::QualityGate,
        Some(0.0),
        true,
        QUALITY_MIN
    ));
}

#[test]
fn policy_all_always_stitches_and_off_never_does() {
    for quality in [None, Some(0.0), Some(QUALITY_MIN), Some(40.0)] {
        for forced in [false, true] {
            assert!(stitched(SeamPolicy::All, quality, forced, QUALITY_MIN));
            assert!(!stitched(SeamPolicy::Off, quality, forced, QUALITY_MIN));
        }
    }
}

#[test]
fn forced_only_stitches_only_a_forced_cut() {
    assert!(
        !stitched(SeamPolicy::ForcedOnly, Some(40.0), false, QUALITY_MIN),
        "ein Kandidat bleibt unter ForcedOnly kahl"
    );
    assert!(
        stitched(SeamPolicy::ForcedOnly, Some(0.0), true, QUALITY_MIN),
        "ein erzwungener Schnitt konditioniert"
    );
}

#[test]
fn the_policy_default_is_the_quality_gate() {
    assert_eq!(SeamPolicy::default(), SeamPolicy::QualityGate);
}

// ---------------------------------------------------------------------------
// Pre-roll und Upload
// ---------------------------------------------------------------------------

#[test]
fn pre_roll_clamps_at_the_stream_start() {
    let previous = [1i16, 2, 3, 4, 5];

    assert_eq!(
        pre_roll(&previous, 8),
        &previous[..],
        "kürzer als der Pre-roll"
    );
    assert_eq!(pre_roll(&previous, 2), &[4, 5], "die letzten zwei");
    assert_eq!(pre_roll(&previous, 0), &[] as &[i16], "kein Pre-roll");
    assert_eq!(pre_roll(&[], 8), &[] as &[i16], "leerer Vorgänger");
}

#[test]
fn stitched_upload_prepends_the_pre_roll_byte_for_byte() {
    let previous = [10i16, 20, 30, 40];
    let fragment = [50i16, 60];

    let audio = upload(&previous, &fragment, true, 2);

    assert_eq!(audio, vec![30, 40, 50, 60]);
}

#[test]
fn bare_upload_is_the_fragment_alone() {
    let previous = [10i16, 20, 30, 40];
    let fragment = [50i16, 60];

    assert_eq!(upload(&previous, &fragment, false, 2), fragment.to_vec());
}
