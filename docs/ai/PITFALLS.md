---
type: Reference
title: Fallstricke
description: Hart erarbeitete Fallen aus Audio, STT, Textübergabe, Bedienung und den drei Plattformen — nicht neu entdecken.
tags: [referenz, fallstricke]
generated: { by: opencode/general, at: 2026-11-21T03:22:15Z }
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
  **Gemessen am 2026-11-21 (Word 365, Build 20625) ist das für `CF_HTML` falsch:**
  `<h1>`/`<h2>` kommen als echte Formatvorlagen „Heading 1"/„Heading 2" an, `<ul>`
  als echte Aufzählung (`listType=4`), `strong`/`em` zeichengenau; nur Fließtext
  landet in „Normal (Web)". Die Einstellung `PasteFormattingOtherApp` (0/1/2) hat
  daran nichts geändert — weder bei HTML noch bei reinem Text. Ob sie nur in der
  Oberfläche wirkt, ist offen.
- **Bei reinem Text übernimmt Word die Absatzvorlage an der Einfügestelle auf
  alle eingefügten Absätze**, nicht nur auf den ersten (gemessen 2026-11-21: drei
  Absätze in einem „Block Text"-Absatz, alle drei wurden „Block Text").
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
  Drittanbieter-Clipboard-Manager halten sich aber nicht zwingend daran. Das
  gilt für **Windows**; macOS hat nur eine Konvention und Linux praktisch nichts
  (siehe [DOMAIN.md](DOMAIN.md), „Formate und Ausschluss je Plattform").
- **UIPI scheitert stumm.** Erhöht laufende Ziele erreichen weder Hook noch
  Injection, und `SendInput` meldet den Fehlschlag **nicht** — der Text
  verschwindet.[^handy-uipi] Für uns folgenlos: der Text bleibt in der
  Zwischenablage. Relevant blieb es nur für das Schlucken der Leertaste — dort
  war der Hook blind. **Entfällt mit der Hook-Entscheidung (2026-11-20)**: es
  wird keine fremde Taste mehr geschluckt, und das stumme Scheitern eines
  Einfügens bleibt folgenlos, weil die Zwischenablage den Text behält.
- **Google Docs ist ein Canvas.** UI Automation ist wertlos; nur echte
  Tastatur-Events oder ein Paste kommen an.

## Bedienung

- **Space darf nur im Zustand `RECORDING` geschluckt werden**, sonst tippt
  niemand mehr Leerzeichen. **Entfällt: wir hooken nicht mehr** (Entscheidung
  „Kein globaler Tastenhook", 2026-11-20). Space gilt nur noch im
  **Diktierplatz** — dem fokussierten Workbench-Fenster — und wird dort vom
  eigenen Fenster abgefangen, nicht systemweit. Der Satz bleibt stehen, weil er
  die Grenze benennt, an der ein systemweiter Hook scheitern muss.
- **Doppelt einfügen droht nur bei der Wiederholung.** Das erste Einfügen
  nach einem Diktat ist erlaubt (Einstellung `insert_after_dictation`);
  **Prompt anwenden** fügt nie ein, sonst steht der zweite Lauf im fremden
  Dokument. In das **eigene** Fenster wird nie eingefügt.
- **Rohtext nie gewrappt speichern** (Fehler aus `aitranscribe`: `PromptManager`
  wrappt beim Schreiben).
- **Windows 11 versteckt neue Tray-Symbole im Überlauf** ⇒ Overlay-Pille
  zusätzlich, und das Tray nie als einzigen Zugang behandeln.
- **Der Low-Level-Hook ist bei fokussierter Admin-App blind.** Das Schlucken
  funktioniert nur zwischen Prozessen gleicher Integritätsstufe;
  `RegisterHotKey` funktioniert weiter. Siehe [INTEGRATION.md](INTEGRATION.md).
  **Entfällt mit der Hook-Entscheidung (2026-11-20)** — genau deshalb sind die
  drei globalen Hotkeys der Weg und nicht ein Hook.

## Windows

Gemessen am 2026-11-21 auf Windows 11 Pro (10.0.26200), Word 365 Build 20625 und
einem Wegwerf-Crate (`windows` 0.62.2, Rust 1.100.0).

- **Die Zwischenablage ist sitzungsgetrennt.** Ein Prozess in Session 0 (Dienste)
  schreibt in eine andere Zwischenablage als die Nutzersitzung: nach einem
  Schreibversuch aus Session 0 las Session 1 weiterhin den alten Text. Der Kern
  samt Tray muss deshalb **in der Nutzersitzung** laufen — aus einem Dienst heraus
  ist auch der Win+V-Ausschluss wirkungslos, weil der Text dort nie ankommt.
- **Ohne entsperrten Desktop gibt es keine Eingabe.** In Session 0 und in einer
  gesperrten Sitzung liefern `OpenInputDesktop` „Access denied",
  `GetForegroundWindow` 0 und `SendInput` 0 von n Events (der Eingabe-Desktop
  gehört dann Winlogon). Entsperrt funktioniert derselbe Aufruf: 64 Events für
  31 Zeichen, Unicode samt Emoji (Surrogatpaar `U+D83C U+DFA4`) kommt zeichengleich
  an. Für jede Automatisierung heißt das: sie muss in die entsperrte
  Nutzersitzung starten, sonst misst sie nichts.
- **`RegisterHotKey` ist der Hook-Ersatz und wirkt global ohne eigenes Fenster**:
  drei Bindungen registriert, drei ausgelöst, `gesendet=3 empfangen=3`. Aber
  **Kombinationen können fremdbelegt sein** — `Ctrl+Alt+F11` und `Ctrl+Alt+F12`
  antworten `ERROR_HOTKEY_ALREADY_REGISTERED` (1409); auf diesem Host hält
  `igfxEMN.exe` (Intel Graphics Event Monitor) sie. Die Schale muss den Fehlschlag
  behandeln und eine Ausweichkombination anbieten, sonst startet sie stumm ohne
  Hotkey.
- **Die Ausschlussmarker bleiben nach `SetClipboardData` als Formate stehen**
  (`CanIncludeInClipboardHistory`, `CanUploadToCloudClipboard`,
  `ExcludeClipboardContentFromMonitorProcessing`, je DWORD 0). Windows entfernt
  sie nicht selbst. Ob Win+V sie beachtet, ist **nicht** gemessen: die History war
  aus (`EnableClipboardHistory` nicht gesetzt, kein `…\Windows\Clipboard`-Ordner).
- **`Shell_NotifyIcon` trägt auch bei gesperrter und getrennter Sitzung**, solange
  `explorer.exe` läuft: `NIM_ADD` → 1, `Shell_NotifyIconGetRect` liefert ein
  Rechteck im Infobereich, `NIM_MODIFY` wechselt das Icon, `NIM_DELETE` → 1 und
  danach kein Rechteck mehr. `GetRect` ist der programmatische Beweis, dass das
  Symbol wirklich im Infobereich liegt — ein Screenshot wäre keiner.
- **`cpal` über WASAPI hält 30 s ohne Aussetzer.** 48 000 Hz stereo F32, auf
  16 kHz mono s16le gebracht: 1 453 920 Frames gegen 1 454 426 erwartete
  (−0,035 %, also 10 ms auf 30 s), 3 029 Callbacks, größte Lücke **15,41 ms**,
  Spitze −27,3 dBFS bei Sprache. Die Abweichung liegt unter einer
  Callback-Periode; das ist Puffer-Rest, kein Aussetzer.
- **Ein Gerätewechsel invalidiert den offenen Stream.** USB-Headset anstecken
  während der Aufnahme → `The stream configuration is no longer valid and must be
  rebuilt.`, der Fehler-Callback feuert, die Frames brechen ab (3,4 s von 30 s,
  größte Lücke 252 ms), und das Standardgerät flattert zwischen Realtek und dem
  Headset. Die Aufnahme **überlebt nicht** — sie endet mit einem meldbaren Fehler.
  Die Schale muss den Fehler abfangen und den Stream neu aufbauen; ohne das endet
  die Aufnahme still, weil der Aufrufer weiterläuft und keine Frames mehr bekommt.
- **Die drei Marker wirken gegen Win+V** (gemessen mit eingeschalteter History und
  der WinRT-API `GetHistoryItemsAsync`): Text **ohne** Marker erscheint in der
  History, Text **mit** Markern nicht. Die Marker bleiben dabei als Formate in der
  Zwischenablage stehen; Windows wertet sie beim Aufnehmen in die History aus.

## Linux und Wayland

Befunde aus dem Provisioning (#19, claw, 2026-11-16), aus den ersten Messungen des
Linux-Spikes (#12, think, GNOME 50.1, 2026-11-20) und aus dem X11-Prüfstand
(#12, elite, 2026-11-21). Der Wayland-Teil stützt sich weiter auf Warnungen aus
dem Bau, soweit keine Messung daneben steht; der X11-Teil ist gemessen.

- **Das GlobalShortcuts-Portal leitet die App-ID aus dem Prozess ab, nicht aus den
  Optionen** (gemessen auf think, GNOME Shell 50.1, 2026-11-20). Ein `app_id`-
  Schlüssel in den `CreateSession`-Optionen wird **ignoriert**. Nötig sind
  **beides**: ein cgroup-Scope `app-<id>@….service` **und** eine installierte
  `.desktop`-Datei, die die ID deckt. Fehlt eines, antwortet `CreateSession` mit
  `org.freedesktop.portal.Error.NotAllowed / An app id is required` — ein Fehler,
  aber kein stummes Nichts. Ein aus einer SSH-Sitzung gestarteter Prozess passiert
  das Gate nie, auch nicht über `gio launch` (landet im `session-*.scope`);
  `systemd-run --user --unit=app-<id>@…` mit installierter `.desktop` passiert es.
  Pflichten der Schale bleiben: `.desktop` + Icon beim ersten Start nach
  `~/.local/share/applications/` schreiben, Autostart mit aktuellem `$APPIMAGE`
  neu schreiben.
- **`Registry.Register` ersetzt den Scope nicht.** Von einer Verbindung ohne
  App-Scope antwortet das Portal
  `Error.Failed / Could not register app ID: Connection already associated with an
  application ID` (think, 2026-11-20).
- **GNOME 45–47 liefern kein GlobalShortcuts-Backend.** Ohne Backend gibt es
  keine globalen Hotkeys — das ist die Plattform, nicht ein Fehler. Die App muss
  dann über Fenster und Tray bedienbar bleiben; eine Fehlermeldung beim Start ist
  ehrlicher als ein Hotkey, der nichts tut.
- **`wtype` funktioniert nur unter wlroots.** Sway und Hyprland ja, GNOME und
  KDE nein.
- **`enigo` und `xdotool` melden Erfolg und tippen nichts.** Unter Wayland gibt
  es keinen Injektionsweg in fremde Fenster; die Aufrufe kehren erfolgreich
  zurück, und der Text fehlt. Deshalb meldet die Linux-Schale unter Wayland
  `insert_keystroke: nein` — und nicht „hat geklappt".
- **KDE verliert die RemoteDesktop-Erlaubnis beim Reboot.** Die Freigabe aus dem
  Portal-Dialog gilt für die Sitzung; nach dem Neustart muss sie neu erteilt
  werden. Kein verlässlicher Einfügeweg, also nicht darauf bauen.
- **AppImage ohne `.desktop` ist keine App.** Ohne den Eintrag gibt es weder
  Portal-Bindung noch Autostart; auf Ubuntu ≥ 24.04 heißt `libfuse2` außerdem
  `libfuse2t64` (auf claw 2026-11-20 bestätigt) ⇒ einmalig nachinstallieren oder
  `--appimage-extract-and-run`.
- **`rustup --profile minimal` bringt weder `clippy` noch `rustfmt`.** Die
  Verifikation aus `AGENTS.md` scheitert dann mit „component not installed";
  `rustup component add clippy rustfmt` gehört zum Provisioning (claw,
  2026-11-20).
- **`ssh host 'cargo …'` findet `cargo` nicht.** sshd startet eine Non-Login-
  Shell, `~/.profile` bleibt ungelesen, `~/.bashrc` hilft dort ebenfalls nicht.
  Remote-Aufrufe brauchen `ssh host 'bash -lc "cargo …"'` oder den vollen Pfad
  `/home/<user>/.cargo/bin/cargo` (claw, 2026-11-20).
- **3 GB RAM sind die Grenze, nicht die Software.** Auf claw (3,8 GB, davon
  ~0,7 GB frei, Swap fast voll) bauen Workspace-Aufrufe mit `-j 2`; parallele
  Builds laufen ins OOM. `cpal` braucht `libasound2-dev`, nicht nur
  `libasound2t64`.

### X11: gemessen auf elite (Xubuntu 26.04.1, XFCE 4.20.7, 2026-11-21)

Prüfstand: Laptop `elite`, Sitzung 8, X11 auf `:0`, `xdg-desktop-portal`
1.21.1+ds-1ubuntu3.1 mit Backend `xdg-desktop-portal-gtk` 1.15.3, `cpal` 0.18.2,
Rust 1.100.0. Der Wegwerfcode liegt unter `/tmp/dw-linux-spike-elite/`, nicht im
Repo; **`/tmp` ist dort tmpfs** und überlebt keinen Neustart.

- **Auf X11 gibt es das GlobalShortcuts-Portal nicht.** `gdbus introspect` auf
  `org.freedesktop.portal.Desktop` listet 21 Interfaces — Account, Camera,
  FileChooser, Inhibit, Notification, Secret, Settings … —, aber **kein**
  `GlobalShortcuts`; das Backend ist `-gtk`, nicht `-gnome`. Auf X11 ist
  `XGrabKey` der Weg, das Portal ist dort keine Option (und nach dem Paket-Update
  auf 1.21.1+ds-1ubuntu3.1 unverändert).
- **`XGrabKey` braucht vier Grabs pro Kombination.** Einmal greifen genügt nicht:
  gemessen feuerte `Ctrl+Shift+F10` als `plain`, aber **nicht** als `+Lock`,
  `+Mod2` oder `+Lock+Mod2` — bei aktivem CapsLock ist der Hotkey sonst stumm. Mit
  CapsLock an war erwartungsgemäß `plain` stumm und `+lock` lebendig. Ein Grab
  dauert 0,23–0,32 ms; eine **Fremdbelegung meldet sich als
  `BadAccess (error_code=10, major=33, X_GrabKey)`** in 0,29 ms — die Schale kann
  „schon belegt" also erkennen und melden.
- **`Ctrl+Alt+F<n>` ist für die Schale tabu.** Der X-Server behandelt diese
  Kombination **vor** jedem Client-Grab: der Grab gelingt, der Hotkey feuert aber
  nie, und ein per XTEST gesendeter Anschlag **schaltet wirklich auf eine
  Textkonsole um** — der Bildschirm des Menschen wurde schwarz mit blinkendem
  Cursor, während die Sitzung weiterlief (`Active=no`, zurück mit
  `sudo -n chvt 7`). Keine dieser Kombinationen anbieten, in keinem Test.
- **`_NET_ACTIVE_WINDOW` hat zwei Items.** Gemessen `[0x1e00003, 0x0]` mit
  declared length 2 — **immer nur das erste** lesen. Tote IDs kommen vor: nach
  dem Abschießen des aktiven Testfensters blieb dessen ID in der Property *und*
  in `_NET_CLIENT_LIST` stehen, das Lesen endete in `BadWindow`; `PointerRoot`
  (`0x1`) heißt „kein Fenster". Der Eingabefokus ist oft ein **Kindfenster**
  (`0x1e00004`) und nicht das Toplevel (`0x1e00003`) — Fenster-ID und Fokus nie
  gleichsetzen.
- **Unicode tippt nur mit Serialisierung.** XTEST kennt keine Zeichen; wer Text
  *tippt* (die Lieferung tut das nicht — sie schreibt die Zwischenablage und sendet
  ein Strg+V), mappt einen freien Keycode (248–255) auf das Keysym
  (`0x01000000 + Codepoint` für alles außerhalb Latin-1) und restauriert danach.
  Ohne Wartezeit zwischen Tastenereignis und nächstem Remap gingen im Test
  **81 von 200 Bytes still verloren** (einzeln, nicht am Ende); mit **25 ms pro
  Zeichen** kam `Grüße, Müller€ 1.234,50— „Zitat“😀` dreimal byte-identisch an,
  also ~32 Zeichen/s. Zeichen, die schon auf einem Level-1-Keycode liegen,
  schaffen über den normalen Keycode ~820 Zeichen/s. Der Befund belegt nebenbei,
  dass XTEST das fremde Fenster wirklich erreicht.
- **`xterm` nimmt `XSendEvent` an.** Die Annahme „viele Apps ignorieren
  SendEvent" gilt nicht allgemein — gemessen landeten 15 Bytes aus einem
  synthetischen `SendEvent` im Ziel. Der sichere Weg bleibt XTEST; die Gegenprobe
  mit einem GTK-/Qt-Fenster fehlt noch.
- **Die Zwischenablage braucht einen eigenen Eigentümer.** Sechs Ziele
  gleichzeitig (`TARGETS`, `UTF8_STRING`, `text/plain;charset=utf-8`,
  `text/html`, `STRING`, `x-kde-passwordManagerHint`) sind möglich, HTML-Fragment
  und Reintext kamen **byte-gleich** zurück (115 bzw. 64 Bytes, Umlaute, `§`,
  `€`, Gedankenstrich, `„“` intakt). `xclip` kann das nicht — es dient ein Ziel
  pro Aufruf; die Schale muss die Selection selbst halten.
- **`x-kde-passwordManagerHint` wirkt auf diesem Host nicht.** `xfce4-clipman`
  legte den markierten Inhalt (eindeutiger Marker, `textsrc`) in seine History,
  obwohl `x-kde-passwordManagerHint=secret` mitgeliefert wurde; die Zeichenkette
  `passwordManagerHint` kommt im Binary nicht vor. Der Hinweis ist eine Konvention
  ohne Wirkung — die Zusage bleibt „keine". Auf Xubuntu läuft der Clipman
  standardmäßig **nicht** (Autostart-Eintrag existiert, das Panel-Plugin fehlt).
- **Tray ist StatusNotifier, nicht XEmbed.** Das XFCE-Panel hier hat **kein**
  `systray`-Plugin, sondern `indicator`; `org.kde.StatusNotifierWatcher` gehört
  `ayatana-indicator-application`, `IsStatusNotifierHostRegistered = true`, sechs
  fremde Items (nm_applet, blueman, jetbrains-toolbox …). Wer XEmbed schreibt,
  erscheint auf diesem Desktop gar nicht.
- **Audio 30 s ist unauffällig.** `cpal` über pipewire-alsa, 48 kHz stereo f32:
  1.440.256 Frames gegen 1.440.122 erwartet (**+0,009 %**), größte
  Callback-Lücke 12,3 ms, 0 Streamfehler, Rohdatei 30,01 s; ein zweiter Lauf
  −0,024 % bei 12,6 ms. Das ist die Größenordnung des Windows-Befunds (−0,035 %,
  15 ms) — PipeWire liefert nicht schlechter als WASAPI. 1,03 % der Samples lagen
  bei Eingangsverstärkung 100 % am Vollausschlag: Pegel, kein Aussetzer.
- **`Data::len()` ist bei `cpal` 0.18 nicht die Puffergröße in Bytes.**
  `Data::bytes()` liefert die Bytes; mit `len()` zählte die erste Probe 128 statt
  512 Frames pro Callback und meldete −87,5 %. Genau der Messfehler, der einen
  Plattformbefund erfindet — jede Frame-Zahl gegen die Rohdatei prüfen.

## macOS

- **TCC `PostEvent` scheitert stumm wie UIPI.** Ohne die Freigabe in den
  Bedienungshilfen werden synthetische Tastatur-Events verworfen, und die API
  meldet keinen Fehler. Dasselbe Muster wie unter Windows: der Text bleibt in der
  Zwischenablage, das Einfügen unterbleibt stillschweigend. Fähigkeit
  `insert_keystroke: ja-mit-freigabe`.
- **Gatekeeper ohne Notarisierung.** Signatur und Notarisierung sind keine
  Kür: ohne sie startet die App nur über „Rechtsklick → Öffnen" oder nach
  Entfernen des Quarantäne-Attributs.
- **`org.nspasteboard.ConcealedType` ist nur eine Konvention.** Das OS erzwingt
  nichts; Clipboard-Manager, die den Typ nicht kennen, speichern den Inhalt
  trotzdem. Die Zusage lautet darum „Konvention", nicht „garantiert".

[^whisper-silence]: Whisper Discussion #928 — https://github.com/openai/whisper/discussions/928
[^essv-chunks]: ESSV 2026, S. 232–239 — https://www.essv.de/pdf/2026_232_239.pdf
[^openai-stt]: OpenAI Speech to Text — https://developers.openai.com/api/docs/guides/speech-to-text
[^prompt-echo]: Prompt-Echo-Messung — https://zenodo.org/records/22657843
[^word-merge]: Microsoft 365 Insider — https://insider.microsoft365.com/en-us/blog/updated-default-paste-option-in-word-for-windows
[^ms-clipboard]: Microsoft Clipboard Formats — https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-formats
[^commonmark]: CommonMark 0.31.2 — https://spec.commonmark.org/0.31.2/
[^office-watch]: Office Watch, 2026-11-03 — https://office-watch.com/2026/microsoft-word-markdown-support/
[^handy-uipi]: Handy PR #2502 — https://github.com/cjpais/Handy/pull/2502
