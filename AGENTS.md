# AGENTS.md

Arbeitsregeln für dieses Repository. Globale Regeln stehen in
`~/.config/opencode/AGENTS.md` und `~/AGENTS.md` und gelten zusätzlich.

## Projekt

Diktatwerkstatt ist ein Diktierwerkzeug für Menschen, die professionelle Texte
schreiben, ohne zu tippen — **für Windows, Linux und macOS**: ein geteilter
Rust-Kern und drei dünne Plattformschalen. Nicht fokussiert steuern drei globale
Hotkeys (diktieren roh, diktieren poliert, pausieren/fortsetzen), im fokussierten
Fenster zusätzlich Space und Enter; **kein globaler Tastenhook**. Aufnahme,
Transkription über eine OpenAI-kompatible API, optionale Politur durch ein LLM,
Ausgabe über die Systemzwischenablage in jede Zielanwendung. Das Tray ist die
Bedienoberfläche; das Fenster dient dem Nachlesen, Polieren und **Korrigieren** —
wer dort eine Ansicht bearbeitet, liefert sie beim Verlassen in die
Zwischenablage, und ein Knopf **In die Zwischenablage** liefert den sichtbaren
Reiter jederzeit. FIFO der letzten Diktate, kein Archiv, keine Cloud außer der
ausdrücklich konfigurierten API.

## Knowledge Bootstrap

Vor jeder Aufgabe zuerst den Wissenseinstieg lesen:

0. `docs/ai/index.md` — OKF-Index des Bundes — **hier anfangen**
1. `docs/ai/HANDOFF.md` — **zuerst lesen, dann handeln**
2. `docs/ai/STATE.md` — was existiert, was nicht
3. `docs/ai/DOMAIN.md` — Tabellen, Invarianten, Abläufe
4. `docs/ai/DECISIONS.md` — Entscheidungen mit verworfener Alternative
5. `docs/ai/PITFALLS.md` — Fallstricke, bevor du einen Baustein anfasst
6. `docs/ai/INTEGRATION.md` — wie die Referenzprojekte es unter Windows, Linux und macOS lösen
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

**Doku-Phase, kein Code.** Es wird kein Produktcode geschrieben, bevor die
Spikes die Bausteine belegt haben — Reihenfolge: Provisioning (#19), Linux-Spike
(#12), macOS-Spike (#13), Windows-Spike (#1); nach dem Provisioning laufen
#4/#5/#10 hardwarefrei parallel zu den Spikes.
Das Grill-Ergebnis steht als Kommentar an Epic #3 und ist in `CONTEXT.md`,
`DOMAIN.md` und `DECISIONS.md` eingearbeitet; die Beschlusslage vom 2026-11-16
(drei Ziele, kein Tastenhook, Fähigkeitsleiter) in `docs/ai/DECISIONS.md`, E1–E10.

## Stack

- **Rust** für Kern und Plattformnähe: Audio (`cpal`), globale Hotkeys (kein
  Hook), Zwischenablage (eigene Plattformmodule; `arboard` kann weder
  Privatformate noch den `CF_HTML`-Umschlag), Tray (`tray-icon`), Win32
  (`windows`).
- **Tauri mit drei Webviews** für die Workbench — WebView2 (Windows), WebKitGTK
  (Linux), WKWebView (macOS) —, mit `windows: []` als Startzustand. Der Kern
  läuft ohne jede Webview.
- **SQLite** über `rusqlite` (gebündelt). Kein ORM.

## Aufbau

```
crates/core/        Domäne. Keine UI-, keine Plattform-Abhängigkeit, kein
                    cfg(target_os), kein enum Platform.
apps/desktop/       Tauri-App: Tray, Hotkeys, Zwischenablage, Fenster — je Ziel
                    eine dünne Schale um denselben Kern.
```

`crates/core` muss ohne laufende App und auf jedem Ziel testbar sein. Was eine
Plattform kann, meldet die Schale beim Start als **Fähigkeitsleiter**
(`docs/ai/DOMAIN.md`); der Kern leitet sein Verhalten daraus ab. Wer eine
Abhängigkeit nach `core` zieht, die nur auf einer Plattform kompiliert, oder
dort `cfg(target_os)` schreibt, hat die Schichtung gebrochen.

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

Die Toolchain ist nicht überall vorhanden. **#19 hat claw provisioniert**
(2026-11-20): rustc/cargo 1.100.0 mit `clippy` und `rustfmt`,
`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`,
`libayatana-appindicator3-dev`, `patchelf`, `libfuse2t64`, `libasound2-dev`,
`appimagetool`; die drei Befehle oben laufen dort grün. Zwei Fallen dabei (Details
in `docs/ai/PITFALLS.md`): `--profile minimal` bringt **weder `clippy` noch
`rustfmt`**, und `ssh claw 'cargo …'` findet `cargo` nicht, weil sshd eine
Non-Login-Shell startet.

**elite hat seit 2026-11-21 ebenfalls cargo 1.100.0 mit `clippy`/`rustfmt`**
(#12, Spike-Prüfstand X11) — ohne `libwebkit2gtk-4.1-dev`/`libgtk-3-dev`, taugt
also für Kern und Spikes, nicht für Tauri-Schalen. **think** hat kein `cargo`;
dort gilt die Dokumentenprüfung, bis es provisioniert ist.

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
  Manager, unter Linux der Secret-Service, unter macOS der Schlüsselbund.
- Diktate sind Mandantendaten. Keine Telemetrie, keine Protokolle mit
  Textinhalten, keine Cloud außer der ausdrücklich konfigurierten STT-/LLM-API.
- Beim Schreiben in die **Systemzwischenablage** die History-/Cloud-Ausschlussformate
  setzen, soweit die Plattform sie kennt (siehe `docs/ai/DOMAIN.md`, „Formate und
  Ausschluss je Plattform") — **auch beim Knopf „In die Zwischenablage"**. Der
  Knopf ist kein bewusstes „Transkript kopieren": die Quelle des Textes ist
  dieselbe wie bei der Automatik, ein Mandantendiktat gehört nicht in `Win+V`.
  Unter Windows sind die Marker erzwungen (`CanIncludeInClipboardHistory=0`,
  `CanUploadToCloudClipboard=0`, `ExcludeClipboardContentFromMonitorProcessing`);
  macOS kennt die De-facto-Konvention `org.nspasteboard.ConcealedType`; unter
  Linux gibt es praktisch nichts (KDE: `x-kde-passwordManagerHint`; gemessen am
  2026-11-21: `xfce4-clipman` ignoriert den Hinweis) — dort wird **nicht**
  behauptet, dass nichts in einer History landet. Eine bewusste Aktion
  „Transkript kopieren" gibt es derzeit nicht; sobald es sie gibt, gilt der
  Ausschluss für sie **nicht** — sonst ist der Text nach dem Kopieren nicht in
  der History des Nutzers (siehe `docs/ai/INTEGRATION.md`).

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
- Issues: GitHub Issues; Epic #3 mit den Sub-Issues #1, #4–#19 (#2, #11 und #19
  geschlossen).
  Reihenfolge: #19 Provisioning → #12 Linux-Spike → #13 macOS-Spike → #1
  Windows-Spike; nach dem Provisioning hardwarefrei parallel dazu #4/#5/#10.
- Eigentümer: Georg Graf <grafg@spengergasse.at>

## Sprache

Deutsch für Dokumentation, Begriffe und Commit-Texte. Englisch nur für
Bezeichner im Code.
