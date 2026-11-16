---
type: Reference
title: Tauri/Windows-Integration
description: Wie die Referenzprojekte Hotkey, Hook, Einfügen, Tray und Installer unter Windows lösen — und was wir übernehmen.
tags: [referenz, tauri, windows]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T05:20:00Z }
status: draft
sources:
  - id: handy-lib
    resource: https://github.com/cjpais/Handy/blob/main/src-tauri/src/lib.rs
    title: Handy — Composition Root (Tauri 2.11.5)
  - id: handy-keys
    resource: https://github.com/handy-computer/handy-keys/blob/main/src/platform/windows/listener.rs
    title: handy-keys — Low-Level-Hook in Rust
  - id: handy-paste
    resource: https://github.com/cjpais/Handy/tree/main/src-tauri/src/paste_tx
    title: Handy — paste_tx (Delayed Render + Read-Receipt)
  - id: handy-uipi
    resource: https://github.com/cjpais/Handy/pull/2502
    title: Handy PR #2502 — UIPI-Erkennung beim Einfügen
  - id: handy-tray
    resource: https://github.com/cjpais/Handy/blob/main/src-tauri/src/tray.rs
    title: Handy — Tray (Desired-State-Applier)
  - id: handy-recorder
    resource: https://docs.rs/handy-recorder/0.1.1/handy_recorder/
    title: handy-recorder — Aufnahme-Architektur
  - id: openwhispr-paste
    resource: https://github.com/OpenWhispr/openwhispr/blob/main/resources/windows-fast-paste.c
    title: OpenWhispr — Fast-Paste-Helper (Ziel-HWND, Terminal-Erkennung)
  - id: ms-clipboard
    resource: https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats
    title: Microsoft — Clipboard Formats (Cloud Clipboard, History)
  - id: tauri-installer
    resource: https://tauri.app/distribute/windows-installer/
    title: Tauri — Windows Installer
  - id: tauri-updater
    resource: https://tauri.app/plugin/updater/
    title: Tauri — Updater
  - id: arboard
    resource: https://docs.rs/arboard/3.7.0/arboard/struct.Set.html
    title: arboard 3.7 — Clipboard-API
---

# Warum dieses Konzept existiert

Der Stack ist **Rust + Tauri 2 + WebView2**. Referenzprojekt ist **Handy**
(`cjpais/Handy`, Tauri 2.11.5, React) — es löst die Bausteine, die bei uns
riskant sind, bereits im Produktivbetrieb. Daneben **OpenWhispr**, das kein
Tauri-, sondern ein Electron-Projekt ist, dessen Windows-Helfer aber als
C-Programm nachlesbar sind.

Geteilt wird **kein Code** (siehe [DECISIONS.md](DECISIONS.md), „Keine geteilte
Codebasis") — geteilt werden die Muster.

# Baustein für Baustein

| Baustein | Handy | Was wir übernehmen |
|---|---|---|
| Single Instance | `tauri-plugin-single-instance`, Callback zeigt das Fenster und wertet CLI-Flags aus; im CLI-Modus komplett deaktiviert, sonst wäre die Zweitinstanz ein No-op[^handy-lib] | Plugin **als erstes** registrieren; alles, was im `setup` ein Fenster erzeugt, danach |
| Hotkey | Zwei Backends: `RegisterHotKey` (Plugin) und eigener Hook, umschaltbar; ein fehlgeschlagener Hook wird als Einstellung persistiert, damit der nächste Start nicht endlos retryt[^handy-lib] | Plugin als Default — es funktioniert **auch bei fokussierter Admin-App**, der Hook nicht |
| Taste schlucken | `handy-keys`: `WH_KEYBOARD_LL` auf eigenem Thread, Message-Loop mit `MsgWaitForMultipleObjects(10 ms)` statt `sleep`, `LRESULT(1)` schluckt, Re-Install nach Session-/Power-Events, weil Windows einen Hook mit überschrittenem Zeitbudget **still** verwirft[^handy-keys] | Eigenes Modul (~300 Zeilen), gleiche Mechanik; nichts Blockierendes im Callback |
| Einfügen | Zwei Pfade. *Legacy*: Text lesen, Text schreiben, Strg+V, feste Wartezeit, Restore — verliert alle Formate außer Text. *Reliable*: Transkript als **Delayed-Render-Promise** (`SetClipboardData(CF_UNICODETEXT, NULL)`), `WM_RENDERFORMAT` ist das Read-Receipt, Restore nur solange `GetClipboardSequenceNumber()` unverändert ist[^handy-paste] | v0.1: Legacy **mit vollem Format-Snapshot** (`EnumClipboardFormats` je Format, `CopyImage` für `CF_BITMAP`). Receipt-Pfad als eigenes Modul danach |
| Ziel-Fenster | Handy merkt es sich nicht — `SendInput` geht ans fokussierte Fenster, das Overlay ist nicht fokussierbar. OpenWhispr merkt sich beim Aufnahmebeginn `TARGET <hwnd>` und reaktiviert es beim Einfügen, plus Terminal-Erkennung für Strg+Shift+V[^openwhispr-paste] | HWND beim Aufnahmebeginn merken und beim Einfügen reaktivieren |
| UIPI | `TokenIntegrityLevel` des Vordergrundfensters vergleichen (`0x2000` normal, `0x3000` elevated), `E_ACCESSDENIED` gilt als blockierend, im Zweifel fail-open; Ergebnis wird als Notice gemeldet, der Text bleibt in der History[^handy-uipi] | Übernehmen — das ist die Vorlage für unser Abnahmekriterium „erhöhtes Ziel führt zu sichtbarer Meldung" |
| Tray | **Desired-State-Applier**: Aufrufe schreiben nur einen Wunschzustand, *ein* Applier diffed gegen den zuletzt angewandten und fasst nur Geändertes an. Grund war ein Bug mit verschwindendem Tray-Symbol bei mehreren RebUILDs aus mehreren Threads[^handy-tray] | Applier-Muster; Tray-Fehler **nie** propagieren (ein `E_FAIL` beim Icon-Update hat dort die ganze Transkriptions-Pipeline per Panic getötet) |
| Windows-11-Überlauf | Wird gar nicht behandelt — es gibt keine API, ein Symbol programmatisch sichtbar zu machen | Tray ist Komfort, nie der einzige Zugang: Hotkey funktioniert immer, Onboarding zeigt „Symbol aus dem Überlauf ziehen" |
| Autostart | Plugin, aber die **Einstellung** ist die Wahrheit, nicht ein OS-Query; bei jedem Start wird der OS-Zustand nachgezogen, Fehler nur geloggt — ein von einem Cleaner entfernter Eintrag heilt sich selbst | Übernehmen; der Autostart-Start muss ohne Fenster erfolgen |
| Installer | MSI **und** NSIS, eigenes NSIS-Template nur wegen Portable-Mode; Signatur über Azure Trusted Signing; Updater mit minisign[^handy-lib] | **NSIS** als primäres Ziel (Per-User ohne Elevation, updater-freundlich); MSI nur, wenn eine Enterprise-Verteilung es verlangt |
| WebView2 | Setzt nichts — Tauri-Default `downloadBootstrapper` | Dito: auf Win10/11 vorinstalliert, Bootstrapper nur als Netz-Fallback[^tauri-installer] |
| Fenster-Start | Fenster **eager im `setup` erzeugt und versteckt**, gezeigt per `show()`; kein Fallback, wenn die WebView2-Erzeugung scheitert[^handy-lib] | Eager + versteckt, aber **mit** Reparaturweg: schlägt die Erzeugung fehl, bleibt Tray und Hotkey funktionsfähig und das Fenster wird bei Bedarf neu erzeugt |
| Audio | Eigene Crate: Gerät läuft **im OS-Format**, `rubato` resampelt auf 16 kHz mono — 16 kHz wird nicht von WASAPI verlangt. Stirbt das Mikrofon mitten in der Aufnahme, bleibt das bis dahin aufgenommene Audio erhalten[^handy-recorder] | `cpal` + `rubato`, Gerät pro Aufnahme öffnen, Partial-Audio behalten |
| Zwischenablage | Eigenes Win32-Modul (`windows`-Crate) für den Paste-Pfad; `arboard` kann **keine** Privatformate und kein Delayed Rendering[^arboard] | `arboard` für einfache Fälle, eigenes Win32-Modul für den Paste-Pfad |
| Logging | Datei + Stdout, Webview nur im Debug-Build — „normal runs never broadcast log records (which can include file paths or transcribed text)" | Übernehmen; das ist die Datenschutzentscheidung, die zu Mandantendaten passt |

# Zwei Korrekturen an unserer eigenen Annahme

1. **Der Low-Level-Hook ist bei fokussierter Admin-App blind.** Handys README
   sagt es ausdrücklich, und das trifft unsere Anforderung „Leertaste während
   der Aufnahme schlucken" direkt: das Schlucken funktioniert nur zwischen
   Prozessen gleicher Integritätsstufe. `RegisterHotKey` funktioniert dagegen
   weiter. Konsequenz: für das Diktieren den Hotkey, für Push-to-Talk den Hook —
   und bei erhöhtem Ziel in den Toggle-Modus fallen.
2. **Die Ausschlussformate gehören nur an den transienten Paste.** Handy
   unterscheidet `ClipboardHandling::Paste` (Marker setzen, Win+V bleibt sauber)
   von `CopyToClipboard` (Marker **nicht** setzen, sonst ist der Text nach einem
   bewussten Kopieren nicht in der History des Nutzers). Unsere Regel „immer
   Ausschlussformate" ist damit zu grob, sobald es ein „Transkript kopieren"
   gibt, das der Nutzer behalten will.

# Crates, die wir voraussichtlich brauchen

| Zweck | Crate | Stand 2026-11-16 |
|---|---|---|
| App, Build | `tauri` 2.12.5, `tauri-build` 2 | — |
| Single Instance | `tauri-plugin-single-instance` 2.5.5 | — |
| Hotkey | `tauri-plugin-global-shortcut` 2.4.1 | — |
| Hook, Clipboard, Injection | `windows` 0.62.2 | Handy pinnt 0.61.3 + `webview2-com 0.38` |
| Tastatur-Injektion | `enigo` 0.6.1 | seit 08/2025 unverändert |
| Zwischenablage | `arboard` 3.7.0 | Text + HTML, keine Privatformate |
| Audio, Resampling | `cpal` 0.18.2, `rubato` 5.1.0 | shared mode |
| HTTP, Async | `reqwest` 0.13.5 (`rustls`), `tokio` 1.54.1 | — |
| DB | `rusqlite` 0.43.0 (`bundled`) | kein ORM |
| Log, Autostart, Updater | `tauri-plugin-log` 2.10.2, `-autostart` 2.7.0, `-updater` 2.14.1 | Updater verlangt signierte Manifeste[^tauri-updater] |
| Typed IPC | `tauri-specta` 1.0.2 + `specta` 1.0.5 | lohnt ab ~15 Commands |

# Offene Entscheidungen

1. **Hotkey-Mechanik:** `RegisterHotKey` als einziger Trigger und das Schlucken
   nur dort, wo es funktioniert — oder eigener Hook plus dokumentierte Grenze
   bei Admin-Apps?
2. **Paste-Qualität in v0.1:** Legacy mit vollem Format-Snapshot (ehrlich, aber
   mit fester Wartezeit) oder sofort die ~600 Zeilen Delayed-Render + Receipt?
3. **Installer:** NSIS allein oder MSI zusätzlich für Enterprise-Verteilung?
4. **Wiederverwendung:** `handy-keys` und `handy-recorder` als Dependencies
   (schnell, aber Ein-Personen-Crates mit Wochen-Churn: `handy-keys` 0.3.0 →
   0.3.5 in vier Monaten) oder selbst schreiben, um die Kern-Testbarkeit ohne
   laufende App zu behalten?

[^handy-lib]: Handy `src-tauri/src/lib.rs` — https://github.com/cjpais/Handy/blob/main/src-tauri/src/lib.rs
[^handy-keys]: `handy-keys` `src/platform/windows/listener.rs` — https://github.com/handy-computer/handy-keys/blob/main/src/platform/windows/listener.rs
[^handy-paste]: Handy `src-tauri/src/paste_tx/` — https://github.com/cjpais/Handy/tree/main/src-tauri/src/paste_tx
[^handy-uipi]: Handy PR #2502 — https://github.com/cjpais/Handy/pull/2502
[^handy-tray]: Handy `src-tauri/src/tray.rs` — https://github.com/cjpais/Handy/blob/main/src-tauri/src/tray.rs
[^handy-recorder]: `handy-recorder` 0.1.1 — https://docs.rs/handy-recorder/0.1.1/handy_recorder/
[^openwhispr-paste]: OpenWhispr `resources/windows-fast-paste.c` — https://github.com/OpenWhispr/openwhispr/blob/main/resources/windows-fast-paste.c
[^tauri-installer]: Tauri Windows Installer — https://tauri.app/distribute/windows-installer/
[^tauri-updater]: Tauri Updater — https://tauri.app/plugin/updater/
[^arboard]: arboard 3.7 `Set` — https://docs.rs/arboard/3.7.0/arboard/struct.Set.html
