# AGENTS.md

Arbeitsregeln für dieses Repository. Globale Regeln stehen in
`~/.config/opencode/AGENTS.md` und `~/AGENTS.md` und gelten zusätzlich.

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

## Daten und Geheimnisse

- API-Schlüssel nie in der Datenbank, nie im Repository. Windows Credential
  Manager, unter Linux der Secret-Service.
- Diktate sind Mandantendaten. Keine Telemetrie, keine Protokolle mit
  Textinhalten, keine Cloud außer der ausdrücklich konfigurierten STT-/LLM-API.
- Beim Schreiben in die Zwischenablage immer die History-/Cloud-Ausschlussformate
  setzen (siehe `docs/ai/DOMAIN.md`, Abschnitt Lieferung).

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

## Sprache

Deutsch für Dokumentation, Begriffe und Commit-Texte. Englisch nur für
Bezeichner im Code.
