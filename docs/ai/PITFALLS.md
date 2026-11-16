---
type: Reference
title: Fallstricke
description: Hart erarbeitete Fallen aus Audio, STT, Textübergabe und Bedienung — nicht neu entdecken.
tags: [referenz, fallstricke]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T06:08:40Z }
status: stable
sources:
  - id: whisper-silence
    resource: https://github.com/openai/whisper/discussions/928
    title: Whisper — Halluzinationen auf Stille, dokumentierte deutsche Artefakte
  - id: essv-chunks
    resource: https://www.essv.de/pdf/2026_232_239.pdf
    title: ESSV 2026 — faster-whisper auf deutscher Verwaltungssprache, Chunk-Längen
  - id: openai-stt
    resource: https://developers.openai.com/api/docs/guides/speech-to-text
    title: OpenAI — Speech to Text (Prompt-Limit, Abkündigungen)
  - id: prompt-echo
    resource: https://zenodo.org/records/22657843
    title: Prompt-Echo bei Whisper-Biasing (WER 6,5 % → 61 %)
  - id: word-merge
    resource: https://insider.microsoft365.com/en-us/blog/updated-default-paste-option-in-word-for-windows
    title: Microsoft 365 Insider — neuer Standard-Einfügemodus „Merge Formatting"
  - id: ms-clipboard
    resource: https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats
    title: Microsoft — Clipboard Formats (Cloud Clipboard, History)
  - id: commonmark
    resource: https://spec.commonmark.org/0.31.2/
    title: CommonMark Spec 0.31.2
  - id: office-watch
    resource: https://office-watch.com/2026/microsoft-word-markdown-support/
    title: Office Watch — Words Markdown-Unterstützung und die AutoFormat-Falle
  - id: handy-uipi
    resource: https://github.com/cjpais/Handy/pull/2502
    title: Handy PR #2502 — UIPI-Erkennung, stummer Fehlschlag von SendInput
---

# Warum diese Datei existiert

Jede Zeile hier hat einmal Zeit gekostet. Wer sie neu entdeckt, zahlt doppelt.
Der Einstieg in die Arbeit steht in [HANDOFF.md](HANDOFF.md); das Domänenwissen
in [DOMAIN.md](DOMAIN.md).

## Audio und STT

- **Whisper halluziniert auf Stille.** Auf reinem Raumton emittiert `large-v3`
  in 61,9 % der Fälle Wörter, dokumentiert sind deutsche Untertitel-Credits
  („Untertitel der Amara.org-Community"). Ein Praxisbericht: 37 min Meeting plus
  5 h Leerlauf ergab ~600 erfundene von 963 Zeilen.[^whisper-silence] Stille darf
  nie an die API gehen; es braucht einen harten Stopp, keinen
  „läuft weiter"-Modus. Das ist ein Haftungs-, kein Qualitätsproblem.
- **Das stabile Fenster ist 20–30 s.** Auf deutscher Verwaltungssprache gemessen:
  30 s → 4,76 % WER, 45 s → **34,59 %**, 60 s → **51,47 %** (und dabei
  *schneller*, ein Tuning-Fallstrick).[^essv-chunks] Die geerbten ~21-s-Fragmente
  liegen richtig.
- **Prompt-Biasing ist begrenzt und gefährlich.** Der Prompt ist auf 224 Token
  begrenzt (~60–100 Fachwörter);[^openai-stt] naiv injiziert treibt Prompt-Echo
  den WER von 6,5 % auf **61 %**.[^prompt-echo] Nur mit Echo-Scrubbing einsetzen.
- **`condition_on_previous_text` pflanzt Halluzinationen fort.** Eine einmal
  erfundene Phrase wird Fenster für Fenster weitergetragen (gemessen: Anteil der
  Segmente in Wiederholungsläufen 71 % → 1,4 %, als der Kontext auf 0 ging).
- **OpenAI kündigt `whisper-1` und `gpt-4o-transcribe` zum 26.02.2027 ab**; der
  Nachfolger hat einen eigenen `keywords`-Kanal statt des `prompt`-Parameters.[^openai-stt]
  Nicht auf den Prompt als Biasing-Kanal bauen.
- **Die „120-s-Grenze" bei Groq existiert nicht.** Sie war ein ffmpeg-Bug
  (`-reset_timestamps 1` fehlt). Groq dokumentiert 25/100 MB und eine
  Mindestlänge von 0,01 s, kein Zeitlimit.
- **Deutsch auf Spontansprache ist nicht Deutsch auf Lesesprache.** 2,8–3,5 % WER
  gelesen, 8,7–20 % spontan (Verwaltung 9,4 %, Interview mit Akzent 8,8 %,
  österreichische Medizin 11,7–20,2 %). Ziffernlastige Passagen: 27,8 %. Wer mit
  FLEURS-WER argumentiert, mittelt genau die Fehlerklasse weg, die zählt.

## Text und Einfügen

- **Word übernimmt beim Einfügen von reinem Text die Formatierung an der
  Einfügestelle.** Die Briefvorlage liefert das Aussehen. Was fehlt, ist
  *Struktur*, nicht Formatierung.
- **„Merge Formatting" ist seit Word-Build 2405 der Default** für Einfügen aus
  anderen Programmen. Microsofts Beschreibung dessen, was erhalten bleibt, nennt
  Fett, Unterstreichung, Listen und Tabellen — **Überschriften nicht**.[^word-merge]
  Ob `h1`–`h6` verflachen, ist ungemessen und ein Abnahmekriterium von #1.
- **AutoCorrect greift beim Einfügen nicht.** Gerade Anführungszeichen und `--`
  bleiben stehen; das gilt auch für COM-Einfügung.
- **Markdown ist gegen juristischen Text nicht neutral.**[^commonmark]

  | Konstrukt | Wird zu | Folge |
  |---|---|---|
  | Zeile beginnt `1) ` | Aufzählung | die klassische deutsche Enumeration wird zur Liste |
  | `1.` … `2)` … `3.` gemischt | drei getrennte Listen | Nummerierung springt |
  | `5*6*78`, Fußnotenmarker `*` | Kursiv (Intraword ist erlaubt) | Text zerstört |
  | `AZ_2024_1234` | je Renderer unterschiedlich | Aktenzeichen bricht |
  | `\§` | bleibt `\§` | Backslash im Brief |

  Dazu Words alte AutoFormat-Option: sie macht `*fett*` **fett**, wo Markdown
  *kursiv* meint — Bedeutungsumkehr.[^office-watch]
- **Die Clipboard-Ausschlussformate wirken global**, nicht je Datenformat —
  `CF_HTML` daneben zu legen hebt den Win+V-Ausschluss nicht auf.[^ms-clipboard]
  Drittanbieter-Clipboard-Manager halten sich aber nicht zwingend daran.
- **UIPI scheitert stumm.** Erhöht laufende Ziele erreichen weder Hook noch
  Injection, und `SendInput` meldet den Fehlschlag **nicht** — der Text
  verschwindet.[^handy-uipi] Für uns folgenlos: der Text bleibt in der
  Zwischenablage. Relevant bleibt es nur für das Schlucken der Leertaste —
  dort ist der Hook blind.
- **Google Docs ist ein Canvas.** UI Automation ist wertlos; nur echte
  Tastatur-Events oder ein Paste kommen an.

## Bedienung

- **Space darf nur im Zustand `RECORDING` geschluckt werden**, sonst tippt
  niemand mehr Leerzeichen.
- **Doppelt einfügen droht nur bei der Wiederholung.** Das erste Einfügen
  nach einem Diktat ist erlaubt (Einstellung `insert_after_dictation`);
  **Prompt anwenden** fügt nie ein, sonst steht der zweite Lauf im fremden
  Dokument.
- **Rohtext nie gewrappt speichern** (Fehler aus `aitranscribe`: `PromptManager`
  wrappt beim Schreiben).
- **Windows 11 versteckt neue Tray-Symbole im Überlauf** ⇒ Overlay-Pille
  zusätzlich, und das Tray nie als einzigen Zugang behandeln.
- **Der Low-Level-Hook ist bei fokussierter Admin-App blind.** Das Schlucken
  funktioniert nur zwischen Prozessen gleicher Integritätsstufe;
  `RegisterHotKey` funktioniert weiter. Siehe [INTEGRATION.md](INTEGRATION.md).

[^whisper-silence]: Whisper Discussion #928 — https://github.com/openai/whisper/discussions/928
[^essv-chunks]: ESSV 2026, S. 232–239 — https://www.essv.de/pdf/2026_232_239.pdf
[^openai-stt]: OpenAI Speech to Text — https://developers.openai.com/api/docs/guides/speech-to-text
[^prompt-echo]: Prompt-Echo-Messung — https://zenodo.org/records/22657843
[^word-merge]: Microsoft 365 Insider — https://insider.microsoft365.com/en-us/blog/updated-default-paste-option-in-word-for-windows
[^ms-clipboard]: Microsoft Clipboard Formats — https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats
[^commonmark]: CommonMark 0.31.2 — https://spec.commonmark.org/0.31.2/
[^office-watch]: Office Watch, 2026-11-03 — https://office-watch.com/2026/microsoft-word-markdown-support/
[^handy-uipi]: Handy PR #2502 — https://github.com/cjpais/Handy/pull/2502
