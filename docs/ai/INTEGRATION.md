---
type: Reference
title: Tauri-Integration je Plattform
description: Wie die Referenzprojekte Hotkey, Einfügen, Tray und Installer unter Windows, Linux und macOS lösen — und was wir übernehmen.
tags: [referenz, tauri, windows, linux, macos]
generated: { by: opencode/general, at: 2026-11-20T16:23:16Z }
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

Der Stack ist **Rust + Tauri 2** — mit **drei Webviews**: WebView2 (Windows),
WebKitGTK (Linux), WKWebView (macOS). Referenzprojekt ist **Handy**
(`cjpais/Handy`, Tauri 2.11.5, React) — es löst die Bausteine, die bei uns
riskant sind, bereits im Produktivbetrieb, allerdings nur unter Windows. Für
Linux und macOS stehen die Befunde aus dem Provisioning an #11 (claw,
2026-11-16) und die dortige Recherche. Daneben **OpenWhispr**, das kein
Tauri-, sondern ein Electron-Projekt ist, dessen Windows-Helfer aber als
C-Programm nachlesbar sind.

Geteilt wird **kein Code** (siehe [DECISIONS.md](DECISIONS.md), „Keine geteilte
Codebasis") — geteilt werden die Muster. Was von den Windows-Mustern auf die
anderen Ziele übertragbar ist, sagt die Spalte.

# Baustein für Baustein

| Baustein | Handy | Linux | macOS | Was wir übernehmen |
|---|---|---|---|---|
| Single Instance | `tauri-plugin-single-instance`, Callback zeigt das Fenster und wertet CLI-Flags aus; im CLI-Modus komplett deaktiviert, sonst wäre die Zweitinstanz ein No-op[^handy-lib] | — | — | Plugin **als erstes** registrieren; alles, was im `setup` ein Fenster erzeugt, danach |
| Hotkey | Zwei Backends: `RegisterHotKey` (Plugin) und eigener Hook, umschaltbar; ein fehlgeschlagener Hook wird als Einstellung persistiert, damit der nächste Start nicht endlos retryt[^handy-lib] | GlobalShortcuts-Portal (X11 alternativ `XGrabKey`); das Portal verlangt unter GNOME eine App-ID mit installierter `.desktop`-Datei (E7, E8) | Carbon-`RegisterEventHotKey` bzw. Event-Tap | Plugin als Default — auf **allen drei** Zielen, und es funktioniert **auch bei fokussierter Admin-App**, der Hook nicht. Drei Hotkeys: diktieren roh, diktieren poliert, pausieren/fortsetzen |
| Taste schlucken | `handy-keys`: `WH_KEYBOARD_LL` auf eigenem Thread, Message-Loop mit `MsgWaitForMultipleObjects(10 ms)` statt `sleep`, `LRESULT(1)` schluckt, Re-Install nach Session-/Power-Events, weil Windows einen Hook mit überschrittenem Zeitbudget **still** verwirft[^handy-keys] | **entfällt** | **entfällt** | **Nichts.** Historisch: ein eigenes Modul (~300 Zeilen), gleiche Mechanik, nichts Blockierendes im Callback. **Entfällt mit der Hook-Entscheidung (2026-11-20)** — die App fasst keine fremde Taste mehr an (E3) |
| Einfügen | Zwei Pfade. *Legacy*: Text lesen, Text schreiben, Strg+V, feste Wartezeit, Restore — verliert alle Formate außer Text. *Reliable*: Transkript als **Delayed-Render-Promise** (`SetClipboardData(CF_UNICODETEXT, NULL)`), `WM_RENDERFORMAT` ist das Read-Receipt, Restore nur solange `GetClipboardSequenceNumber()` unverändert ist[^handy-paste] | `wtype` nur unter wlroots, `xdotool` nur X11; `enigo` meldet Erfolg und tippt nichts (siehe [PITFALLS.md](PITFALLS.md)) | `CGEventPost` nur mit TCC-Freigabe („Bedienungshilfen"), sonst stummer Fehlschlag | **Nichts davon.** Wir schreiben das Fragment und behalten es; optional **ein** Strg+V (Cmd+V), ohne Snapshot, ohne Delayed Render, ohne Wiederherstellung. Fehlt die Fähigkeit, wird nicht eingefügt |
| Ziel-Fenster | Handy merkt es sich nicht — `SendInput` geht ans fokussierte Fenster, das Overlay ist nicht fokussierbar. OpenWhispr merkt sich beim Aufnahmebeginn `TARGET <hwnd>` und reaktiviert es beim Einfügen, plus Terminal-Erkennung für Strg+Shift+V[^openwhispr-paste] | Wayland: keine API ⇒ `never`; X11: `_NET_ACTIVE_WINDOW` | `frontmostApplication` | Nur die Prüfung: derselbe Vordergrund wie beim Abschluss. **Kein** Reaktivieren. Ohne `foreground_window` wird `same_window` zu `never` (E4) |
| UIPI | `TokenIntegrityLevel` des Vordergrundfensters vergleichen (`0x2000` normal, `0x3000` elevated), `E_ACCESSDENIED` gilt als blockierend, im Zweifel fail-open; Ergebnis wird als Notice gemeldet, der Text bleibt in der History[^handy-uipi] | — | TCC statt UIPI: fehlende Freigabe scheitert genauso stumm | **Nicht übernehmen.** Der Text bleibt in der Zwischenablage; ein stummer Fehlschlag ist folgenlos |
| Tray | **Desired-State-Applier**: Aufrufe schreiben nur einen Wunschzustand, *ein* Applier diffed gegen den zuletzt angewandten und fasst nur Geändertes an. Grund war ein Bug mit verschwindendem Tray-Symbol bei mehreren RebUILDs aus mehreren Threads[^handy-tray] | AppIndicator/StatusNotifier — Bibliothek **vom Host**, nicht gebündelt (sonst bricht das Theming) | Menüleisten-Icon | Applier-Muster; Tray-Fehler **nie** propagieren (ein `E_FAIL` beim Icon-Update hat dort die ganze Transkriptions-Pipeline per Panic getötet) |
| Windows-11-Überlauf | Wird gar nicht behandelt — es gibt keine API, ein Symbol programmatisch sichtbar zu machen | — | — | Tray ist Komfort, nie der einzige Zugang: Hotkey funktioniert immer, Onboarding zeigt „Symbol aus dem Überlauf ziehen". Auf Linux/macOS meldet die Fähigkeitsleiter, was das Tray kann |
| Autostart | Plugin, aber die **Einstellung** ist die Wahrheit, nicht ein OS-Query; bei jedem Start wird der OS-Zustand nachgezogen, Fehler nur geloggt — ein von einem Cleaner entfernter Eintrag heilt sich selbst | `.desktop` in `~/.config/autostart`, beim AppImage bei jedem Start mit dem aktuellen `$APPIMAGE` neu geschrieben | LaunchAgent bzw. Plugin | Übernehmen; der Autostart-Start muss ohne Fenster erfolgen |
| Installer | MSI **und** NSIS, eigenes NSIS-Template nur wegen Portable-Mode; Signatur über Azure Trusted Signing; Updater mit minisign[^handy-lib] | AppImage primär, `.deb` zweitens (E7) | `.app` im `.dmg`, signiert und notarisiert | **NSIS** als primäres Windows-Ziel (Per-User ohne Elevation, updater-freundlich); MSI nur, wenn eine Enterprise-Verteilung es verlangt. Linux und macOS nach E7 bzw. Gatekeeper-Regel |
| Webview | Setzt nichts — Tauri-Default `downloadBootstrapper` | WebKitGTK aus dem System | WKWebView aus dem System | Auf Win10/11 ist WebView2 vorinstalliert, der Bootstrapper ist nur Netz-Fallback[^tauri-installer]. Der Kern läuft ohne jede Webview (Fast Path) |
| Fenster-Start | Fenster **eager im `setup` erzeugt und versteckt**, gezeigt per `show()`; kein Fallback, wenn die WebView2-Erzeugung scheitert[^handy-lib] | — | — | Eager + versteckt, aber **mit** Reparaturweg: schlägt die Erzeugung fehl, bleiben Tray und Hotkeys funktionsfähig und das Fenster wird bei Bedarf neu erzeugt |
| Audio | Eigene Crate: Gerät läuft **im OS-Format**, `rubato` resampelt auf 16 kHz mono — 16 kHz wird nicht von WASAPI verlangt. Stirbt das Mikrofon mitten in der Aufnahme, bleibt das bis dahin aufgenommene Audio erhalten[^handy-recorder] | `cpal` über ALSA/PipeWire | `cpal` über CoreAudio | `cpal` + `rubato`, Gerät pro Aufnahme öffnen, Partial-Audio behalten |
| Zwischenablage | Eigenes Win32-Modul (`windows`-Crate) für den Paste-Pfad; `arboard` kann **keine** Privatformate und kein Delayed Rendering[^arboard] | `text/html` + `text/plain;charset=utf-8`; kein wirksamer History-Ausschluss | `public.html` + `public.utf8-plain-text` + `org.nspasteboard.ConcealedType` | Eigenes Modul je Schale: Windows schreibt den `CF_HTML`-Umschlag und die drei Ausschlussformate, macOS und Linux das nackte Fragment (E5, E10) |
| Logging | Datei + Stdout, Webview nur im Debug-Build — „normal runs never broadcast log records (which can include file paths or transcribed text)" | dito | dito | Übernehmen; das ist die Datenschutzentscheidung, die zu Mandantendaten passt |

# Zwei Korrekturen an unserer eigenen Annahme

1. **Der Low-Level-Hook ist bei fokussierter Admin-App blind.** Handys README
   sagt es ausdrücklich, und das traf unsere Anforderung „Leertaste während
   der Aufnahme schlucken" direkt: das Schlucken funktioniert nur zwischen
   Prozessen gleicher Integritätsstufe. `RegisterHotKey` funktioniert dagegen
   weiter. Konsequenz war: für das Diktieren den Hotkey, für Push-to-Talk den
   Hook — und bei erhöhtem Ziel in den Toggle-Modus fallen.
   **Entfällt mit der Hook-Entscheidung (2026-11-20).** Es gibt keinen Hook und
   kein Push-to-talk; die drei globalen Hotkeys und Space/Enter im Diktierplatz
   ersetzen beides. Die Beobachtung bleibt richtig und ist der Grund für die
   Entscheidung.
2. **Die Ausschlussformate gehören an jede Lieferung, aber nicht an ein
   bewusstes Kopieren.** Unsere Lieferung ist keine Zwischenstation: der Text
   bleibt in der Zwischenablage, also Marker immer. Sobald es eine bewusste
   Aktion „Transkript kopieren" gibt, die der Nutzer behalten will, gilt das
   für sie nicht. **Präzisiert 2026-11-20:** Das gilt unverkürzt nur unter
   Windows (harte API-Marker). macOS kennt die Konvention
   `org.nspasteboard.ConcealedType`, Linux praktisch nichts — die Zusage ist
   dort entsprechend schwächer formuliert (E5).

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

Ein Hook-Crate wird **nicht** gebraucht (E3); `enigo` bleibt optional für das
Einfügen und wird unter Wayland nicht verwendet, weil es dort Erfolg meldet und
nichts tippt.

# Offene Entscheidungen

1. **Hotkey-Mechanik — entschieden 2026-11-20.** Die Frage „`RegisterHotKey`
   plus ein dritter Hotkey für Pause/Fortsetzen, damit keine blanke Taste
   geschluckt wird — oder Leertaste und Enter mit eigenem Hook plus
   dokumentierter Grenze bei Admin-Apps?" ist beantwortet: **drei globale
   Hotkeys, kein Hook**, Space und Enter nur im fokussierten Workbench-Fenster
   (E3). Push-to-talk entfällt, weil ohne Hook kein Key-up existiert.
2. **Installer:** NSIS allein oder MSI zusätzlich für Enterprise-Verteilung?
   Unter Linux ist es entschieden (AppImage primär, `.deb` zweitens, E7), unter
   macOS entscheidet die Notarisierung den Weg; offen bleibt nur die
   Windows-Frage.

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
