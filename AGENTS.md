# AGENTS.md

Arbeitsregeln für dieses Repository. Globale Regeln stehen in
`~/.config/opencode/AGENTS.md` und `~/AGENTS.md` und gelten zusätzlich.

## Status

**Spezifikationsphase.** Es wird kein Produktcode geschrieben, bevor der Spike
(siehe Issues) die vier Windows-Bausteine belegt hat.

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

- `docs/ai/` folgt OKF v0.2: Frontmatter mit `type`, `title`, `description`,
  `tags`, `status`.
- Neue Begriffe gehören in `CONTEXT.md`, neue Regeln in `docs/ai/DOMAIN.md`,
  neue Entscheidungen nach `docs/ai/DECISIONS.md` — mit verworfener Alternative.
- Beobachtungen, die diesen Dokumenten widersprechen, werden im selben Commit
  korrigiert.

## Sprache

Deutsch für Dokumentation, Begriffe und Commit-Texte. Englisch nur für
Bezeichner im Code.
