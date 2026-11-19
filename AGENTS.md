# AGENTS.md

Arbeitsregeln für dieses Repository. Globale Regeln stehen in
`~/.config/opencode/AGENTS.md` und `~/AGENTS.md` und gelten zusätzlich.

## Projekt

Diktatwerkstatt ist ein Windows-Diktierwerkzeug für Menschen, die professionelle
Texte schreiben, ohne zu tippen: globaler Hotkey, Aufnahme, Transkription über
eine OpenAI-kompatible API, optionale Politur durch ein LLM, Ausgabe über die
Zwischenablage in jede Zielanwendung. Das Tray ist die Bedienoberfläche; das
Fenster dient dem Nachlesen, Polieren und **Korrigieren** — wer dort eine
Ansicht bearbeitet, liefert sie beim Verlassen in die Zwischenablage. FIFO der
letzten Diktate, kein Archiv, keine Cloud außer der ausdrücklich konfigurierten
API.

## Knowledge Bootstrap

Vor jeder Aufgabe zuerst den Wissenseinstieg lesen:

0. `docs/ai/index.md` — OKF-Index des Bundes — **hier anfangen**
1. `docs/ai/HANDOFF.md` — **zuerst lesen, dann handeln**
2. `docs/ai/STATE.md` — was existiert, was nicht
3. `docs/ai/DOMAIN.md` — Tabellen, Invarianten, Abläufe
4. `docs/ai/DECISIONS.md` — Entscheidungen mit verworfener Alternative
5. `docs/ai/PITFALLS.md` — Fallstricke, bevor du einen Baustein anfasst
6. `docs/ai/INTEGRATION.md` — wie die Referenzprojekte es unter Windows lösen
7. `CONTEXT.md` — Vokabular (liegt außerhalb des Bundes)

Enthält `HANDOFF.md` offene Aufgaben, werden sie vor neuer Arbeit erledigt,
außer der Nutzer sagt ausdrücklich etwas anderes. Bei „weiter", „mach weiter",
„wo waren wir" ohne Rückfrage den Handoff lesen und handeln.

## Skill-Trigger

Diese Skills haben keinen Slash-Befehl; sie werden über natürliche Sprache
geladen:

- **knowledge-persistence** — „merk dir das", „remember", „don't forget",
  „save context", „persist knowledge", an einem Sitzungs-Checkpoint. Ergebnis ist
  dieses OKF-Bundle.
- **issue-workflow** — gilt **dauerhaft**: Issue-Bewusstsein, proaktiver
  Commit und Push mit Issue-Referenz, Abschlusskriterien. Die natürlichen
  Auslöser sind manuelle Overrides: „issue start" / „neues issue",
  „issue commit" / „checkpoint", „issue finish" / „issue fertig".
- **grilling** — „grill mich", „grill this plan", „hinterfrage das",
  „stress-test this". Für Entscheidungen, die teuer werden, wenn sie falsch
  sind; das Ergebnis gehört als Kommentar an das zugehörige Issue.

## Status

**Spezifikationsphase, ausgegrillt.** Es wird kein Produktcode geschrieben, bevor
der Spike (#1) die vier Windows-Bausteine belegt hat. Das Grill-Ergebnis steht
als Kommentar an Epic #3 und ist in `CONTEXT.md`, `DOMAIN.md` und `DECISIONS.md`
eingearbeitet.

## Stack

- **Rust** für Kern und Plattformnähe: Audio (`cpal`), Hook
  (`global-hotkey` bzw. `WH_KEYBOARD_LL`), Zwischenablage (`arboard`), Tray
  (`tray-icon`), Win32 (`windows`).
- **Tauri + WebView2** für die Workbench, mit `windows: []` als Startzustand.
- **SQLite** über `rusqlite` (gebündelt). Kein ORM.

## Aufbau

```
crates/core/        Domäne. Keine UI-, keine windows-*, keine Tauri-Abhängigkeit.
apps/desktop/       Tauri-App: Tray, Hotkeys, Clipboard, Fenster.
```

`crates/core` muss ohne laufende App testbar sein. Wer eine Abhängigkeit nach
`core` zieht, die nur auf Windows kompiliert, hat die Schichtung gebrochen.

## Verifikation vor jedem Commit

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Nur grün wird committet. Keine Ausnahmen.

**Solange kein Crate existiert** (also bis zum ersten Code-Commit), treten an
diese Stelle die Konsistenzprüfungen der Dokumente: `CONTEXT.md` ist die Quelle
der Begriffe, `DOMAIN.md` die Quelle der Invarianten, `DECISIONS.md` darf keine
dazu widersprüchliche Entscheidung enthalten. Ein Doku-Commit ist grün, wenn
keine Vokabel und keine Invariante an zwei Stellen verschieden definiert ist.

## Commits und Issues

- Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`), auf
  Deutsch oder Englisch, aber innerhalb eines Commits einheitlich.
- Jede Commit-Message referenziert das Issue mit `#<Nummer>`.
- Trunk-basiert auf `main`, keine Feature-Branches ohne Grund.

## gh-Konventionen

- Immer `gh issue view <N> --json title,body,comments` — **nie** das nackte
  `gh issue view`. Die nackte Form scheitert in diesem Repository an der
  Projects-classic-Abstimmung (belegt am 2026-11-16).
- Für alle lesenden `gh`-Befehle `--json` mit expliziten Feldern verwenden; das
  vermeidet Deprecation-Warnungen und hält die Ausgabe schlank.

## Daten und Geheimnisse

- API-Schlüssel nie in der Datenbank, nie im Repository. Windows Credential
  Manager, unter Linux der Secret-Service.
- Diktate sind Mandantendaten. Keine Telemetrie, keine Protokolle mit
  Textinhalten, keine Cloud außer der ausdrücklich konfigurierten STT-/LLM-API.
- Beim Schreiben in die Zwischenablage immer die History-/Cloud-Ausschlussformate
  setzen (siehe `docs/ai/DOMAIN.md`, Abschnitt Lieferung). Sobald es eine
  **bewusste** „Transkript kopieren"-Aktion gibt, gilt das für sie **nicht** —
  sonst ist der Text nach dem Kopieren nicht in der History des Nutzers (siehe
  `docs/ai/INTEGRATION.md`). Die Lieferung beim Verlassen einer bearbeiteten
  Ansicht ist **automatisch** und darum keine solche Aktion: sie schließt die
  History weiter aus.

## Dokumentation

- `docs/ai/` ist ein **OKF-v0.2-Bundle**, nicht nur ein Verzeichnis mit
  Markdown. Jede Konzeptdatei trägt das Envelope:

  ```yaml
  ---
  type: <Konzepttyp>
  title: <Titel>
  description: <eine Zeile>
  tags: [<tag>, ...]
  generated: { by: <producer>/<version>, at: <ISO-8601-UTC> }
  status: stable        # draft | stable | deprecated — sonst nichts
  sources:              # wenn eine Aussage auf externe Quellen zurückgeht
    - id: <kurzschlüssel>
      resource: <URL oder Pfad>
      title: <Kurzbezeichnung>
  ---
  ```

  `status` kennt nur `draft`, `stable`, `deprecated` — `active` ist kein
  OKF-Wert. `generated.by` folgt der Actor-Konvention `<producer>/<version>`
  (Menschen: `human:<id>`). Behauptungen mit Zahlen oder Zitaten bekommen eine
  Fußnote, deren Label die `sources[].id` ist; die Belege stehen im
  Frontmatter, nicht im Fließtext.
- `docs/ai/index.md` trägt `okf_version: "0.2"` und wird bei jeder neuen oder
  entfernten Konzeptdatei mitgezogen. `docs/ai/log.md` bekommt bei jeder
  Änderung einen datumsgruppierten Eintrag (neueste oben).
- `index.md` und `log.md` sind reservierte Dateinamen und dürfen nicht als
  Konzeptdatei verwendet werden.
- `CONTEXT.md` liegt bewusst außerhalb des Bundles (es ist das Glossar des
  ganzen Repositories) und trägt trotzdem ein Envelope.
- Neue Begriffe gehören in `CONTEXT.md`, neue Regeln in `docs/ai/DOMAIN.md`,
  neue Entscheidungen nach `docs/ai/DECISIONS.md` — mit verworfener Alternative.
- Beobachtungen, die diesen Dokumenten widersprechen, werden im selben Commit
  korrigiert.

## Repository und Kontakt

- GitHub: `georgernstgraf/diktatwerkstatt`
- Issues: GitHub Issues; Epic #3 mit den Sub-Issues #1, #4–#9 (#2 geschlossen)
- Eigentümer: Georg Graf <grafg@spengergasse.at>

## Sprache

Deutsch für Dokumentation, Begriffe und Commit-Texte. Englisch nur für
Bezeichner im Code.
