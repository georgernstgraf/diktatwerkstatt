# Persistenz-Log

## 2026-11-16 — Referenz: Tauri/Windows-Integration

* **Creation**: [INTEGRATION.md](INTEGRATION.md) — wie Handy (Tauri 2.11.5) und
  OpenWhispr die Windows-Bausteine lösen: Single Instance, Hotkey-Backends,
  Low-Level-Hook (`handy-keys`), Delayed-Render-Paste mit Read-Receipt,
  UIPI-Erkennung, Desired-State-Tray, Autostart, Bundler, Audio-Architektur.
  Belege als `sources` im Frontmatter und als Fußnoten im Text.
* **Update**: [index.md](index.md) um das neue Konzept ergänzt.
* **Verworfen**: Handys Legacy-Paste-Pfad als Vorbild (liest und schreibt nur
  Text, verliert also alle anderen Clipboard-Formate); `handy-keys` und
  `handy-recorder` als Dependencies (Ein-Personen-Crates mit Wochen-Churn).
* **Korrektur an der eigenen Annahme**: Der Low-Level-Hook ist bei fokussierter
  Admin-App blind — das Schlucken der Leertaste funktioniert nur zwischen
  Prozessen gleicher Integritätsstufe, `RegisterHotKey` dagegen weiter.
* **Verifikation**: `okf-check --root docs/ai` grün.

## 2026-11-16 — Epic #3 ausgegrillt, Bundle auf OKF v0.2 gebracht

* **Creation**: [DOMAIN.md](DOMAIN.md), [DECISIONS.md](DECISIONS.md),
  [HANDOFF.md](HANDOFF.md) und [STATE.md](STATE.md) neu geschrieben; dazu
  [index.md](index.md), [log.md](log.md) und dieses Envelope-Verzeichnis.
  `CONTEXT.md` auf das neue Vokabular umgestellt und mit Envelope versehen.
* **Update**: Datenmodell von drei Tabellen auf zwei geschrumpft. Gestrichen:
  `chunk`, `seq`, `handed_over_seq`, `quelle`, `pinned`, `status`, `closed_at`,
  `zielsprache`, `ton`, `archiviert`, `default_model`, `polished_prompt_id`,
  `polished_model`, `polished_at`. Neu: `audio_dir` als einziges Zustandsfeld,
  Arbeitsverzeichnis mit `NNN.wav`/`NNN.txt`, zehn Invarianten, FIFO statt Ring,
  „Prompt anwenden" statt Play.
* **Ausgeführt**: Kommentar an #3 gesetzt; Issues #1, #4, #5, #6, #7, #8, #9 mit
  neuen Bodies, Titel von #4/#7/#8 angepasst; #2 geschlossen (duale Lizenz
  `MIT OR Apache-2.0`, `LICENSE-MIT` + `LICENSE-APACHE`).
* **OKF-Korrektur**: `status: active` ist kein OKF-Wert (§5.4) — alle Dateien auf
  `stable`; `generated.by` auf die Actor-Konvention `<producer>/<version>`
  umgestellt; `index.md` und `log.md` ergänzt, ohne die `docs/ai/` kein
  konformes Bundle ist (§11).
* **Verworfen**: die drei Recherchen, die zu diesem Ergebnis führten, als
  Dateien abzulegen — die Belege stehen als `sources` in
  [HANDOFF.md](HANDOFF.md).
* **Verifikation**: 20 Entscheidungen, 10 Invarianten; grep auf Restvokabeln
  (`Ring`, `Play`, `Delta`, `Rohtextblock`, `pinned`, `handed_over_seq`,
  `zielsprache`, `archiviert`, `chunk`) außerhalb der bewussten Abgrenzungen
  ohne Treffer. `cargo`-Prüfungen entfallen, solange kein Crate existiert
  (`AGENTS.md`, Abschnitt Verifikation).

## 2026-11-15 — Spezifikation angelegt

* **Creation**: `README.md`, `CONTEXT.md`, `docs/ai/DOMAIN.md`,
  `docs/ai/DECISIONS.md`, `docs/ai/HANDOFF.md`, `docs/ai/STATE.md`,
  `AGENTS.md`; Epic #3 mit acht Sub-Issues (#1, #2, #4–#9).
