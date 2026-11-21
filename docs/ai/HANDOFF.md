---
type: Handoff
title: Übergabe
description: Was die nächste Sitzung zuerst tut, in welcher Reihenfolge, und was schon bekannt ist.
tags: [wissensbasis, handoff]
generated: { by: opencode/general, at: 2026-11-21T08:40:06Z }
status: stable
---

# Handoff

Arbeitsverzeichnis: `~/repos/georgernstgraf/diktatwerkstatt`
Repository: https://github.com/georgernstgraf/diktatwerkstatt
Stand: **der Kern steht (#4, #5), die Schalen fehlen.** Branch `main`. Beschlossen
sind **drei Ziele** (Windows, Linux, macOS) mit einem geteilten Kern und drei
dünnen Plattformschalen, **kein globaler Tastenhook** und **keine Plattformweiche
im Kern** — die Beschlusslage steht als E1–E10 in `DECISIONS.md`. Gebaut wird
zuerst auf Linux.

## ✅ Erledigt — Schritt 1: das Epic ausgegrillt

Epic #3 ist durch. Die sechs Fragen vom 2026-11-15 sind beantwortet oder
falsifiziert (Details im Kommentar an #3), das Datenmodell ist auf zwei Tabellen
geschrumpft, `pinned`, `status`, `closed_at`, `chunk`, `seq`, `handed_over_seq`,
`quelle` und die Herkunftsspalten sind gestrichen.

## ✅ Erledigt — Schritt 2: Doku-Nachzug zur Drei-Ziele-Entscheidung (#11)

`DECISIONS`, `CONTEXT`, `DOMAIN`, `PITFALLS`, `INTEGRATION`, `README`, `AGENTS`,
`STATE`, `HANDOFF`, `index` und `log` tragen die Beschlusslage. Die Stellen, die
dadurch falsch geworden sind (Hook, geschluckte Leertaste,
Windows-Zwischenablage, `Tauri/WebView2` als Stackname), sind als „entfällt"
markiert und nicht gelöscht — die Beobachtung dahinter bleibt wertvoll.

## ✅ Erledigt — Schritt 3: Provisioning (#19), claw ist bereit

Ohne Toolchain gibt es keinen grünen `cargo`-Commit, und `AGENTS.md` verlangt ihn
vor jedem Commit. Stand nach dem 2026-11-20:

| Punkt | Stand |
|---|---|
| Rust | rustc/cargo **1.100.0** in `~/.cargo/bin`; `clippy` und `rustfmt` nachinstalliert — im Profil `minimal` **nicht** enthalten |
| Build-Abhängigkeiten | webkit2gtk-4.1 2.52.6, gtk+-3.0 3.24.52, javascriptcoregtk-4.1 2.52.6, libsoup-3.0 3.6.6, `libasound2-dev` für `cpal` |
| Paketnamen (26.04) | `libfuse2` → **`libfuse2t64`**, `libasound2` → **`libasound2t64`**; `cpal` braucht `libasound2-dev`, nicht nur die Laufzeit |
| appimagetool | `continuous` (git 11d54c9) in `~/.local/bin`, FUSE vorhanden, startet direkt |
| Verifikation | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — alle exit 0 in einem Wegwerfcrate; cpal-Smoke-Build ebenso |
| Maschine | Ubuntu 26.04.1, 2 Kerne, **3,8 GB RAM mit ~0,7 GB frei**, 6,4 GB Swap belegt, 51 GB frei |

Zwei Fallen für den nächsten Sitzenden: **`ssh claw 'cargo …'` findet `cargo`
nicht** (sshd startet eine Non-Login-Shell, `~/.profile` bleibt ungelesen) — immer
`ssh claw 'bash -lc "cargo …"'` oder der volle Pfad; und **der Speicher ist die
enge Stelle**, Workspace-Builds laufen mit `-j 2`, nicht parallel.

Provisioniert sind **claw** (Kern und Schale, headless, kein Display) und seit
2026-11-21 **elite** (Rust 1.100.0 mit `clippy`/`rustfmt`, `libasound2-dev`; ohne
Tauri-`-dev`-Pakete). **dell** bleibt der Desktop-Prüfstand für echte Hotkeys,
Tray und AppImage; **think** hat keine Rust-Toolchain.

## ✅ Erledigt — Schritt 5a: #4 Domänenkern (2026-11-21, auf think)

`Cargo.toml` und `crates/core` stehen; der Kern ist zum ersten Mal baubar und
geprüft (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`: 15 Tests grün, ohne App, ohne Netz, ohne Hardware).

| Baustein | Datei | Inhalt |
|---|---|---|
| Schema | `crates/core/src/store.rs` | `diktat` und `prompt` als `STRICT`-Tabellen, `PRAGMA user_version = 1`, WAL mit `synchronous = FULL` |
| Arbeitsverzeichnis | `crates/core/src/fragments.rs` | `NNN.wav`/`NNN.txt`, Schreiben über `.tmp` + `rename`, Verkettung ohne Trenner, 999-Grenze |
| Fehler | `crates/core/src/error.rs` | typisierte Fehler — Append auf ein offenes Diktat ist ein Wert, kein Panic |
| Invarianten | `crates/core/tests/invarianten.rs` | ein Test je Invariante plus die Fälle aus dem Issue |

Drei Entscheidungen, die man kennen muss, bevor man weiterbaut:

- Der **Abschluss ist zweistufig** (`commit_raw_text`, dann
  `remove_working_dir`); `finish_transcription` ist der Weg der App. Nur so ist
  der Absturz zwischen Text und Audio ein Test (`DECISIONS.md`, 2026-11-21).
- Der **sichtbare Text hat genau einen Leseweg**: `Diktat::view_text(View)`.
  Invariante 8 hängt daran, dass es keinen zweiten gibt.
- Das **Arbeitsverzeichnis entsteht vor dem Commit** des Diktats, die
  Verzeichnisse verdrängter Diktate verschwinden **nach** dem Commit — beides,
  damit `audio_dir` nicht lügen kann.

Gegengeprüft: Append-Sperre, Lückenprüfung der Verkettung und die Reihenfolge im
Abschluss einzeln entfernt ⇒ genau der jeweilige Test wird rot.

## ✅ Erledigt — Schritt 5b: #5 DSP-Port (2026-11-21, auf think)

`crates/core/src/audio/` steht: `pcm.rs` (Hochpass 80 Hz **mit**
`i16`-Quantisierung, Rahmenpegel, Upload-Konditionierung, WAV), `silence.rs`
(Q, Achse, Flächenzentroid, Zwangsschnitt), `seam.rs` (Q-Gate, Pre-roll,
Prompt ≤ 96 Zeichen, Echo-Trim ≤ 8 Token, Token-Überlappung) und
`fragmenter.rs` (Slice, Suchfenster, Spool, `NNN.wav`, Worker). 73 neue Tests
(26 `audio`, 28 `naht`, 19 `fragmentierung`), zusammen 88 mit den Invarianten.

| Was | Ergebnis |
|---|---|
| Exakte Kotlin-Schnittwerte | **alle reproduziert**: 7280, 5680, 7280, 400, 80, 240, 9200 Samples — inklusive der „bist"-Regression, des Frikativ-Ausschlusses und der 25-Hz-Brumm-Gegenprobe |
| Byte-exakte Rekonstruktion | Test verkettet die WAV-Nutzlasten der Dateien und vergleicht mit der Aufnahme |
| Genau eine Datei je Fragment | nach N Fragmenten genau `001.wav` … `00N.wav`, keine `.tmp`-, keine Pre-roll-Datei |
| Q-Gate-Grenze | `Q == quality_min` bleibt **kahl** (strikt `<`), ungemessen und `Q = 0` konditionieren |
| 999-Grenze | 999 Fragmente geschrieben, dann `FragmentNumber { 1000 }`, der Fehler ist sticky |
| Gegenprobe | Suchfenster ignoriert ⇒ 4 Fragmentierungstests rot; Q-Gate auf „immer" ⇒ 3 Nahttests rot |

Drei Dinge, die #6 wissen muss:

- **Der Pre-roll ist keine Datei.** `Committed` trägt `stitched`, `quality`,
  `forced` und den `Cut` als Evidenz; den Pre-roll bildet der Client mit
  `upload(previous, fragment, stitched, 16_000)` aus dem Vorfragment, das er
  ohnehin liest. Danach `prompt(previous_text, 96)`, nach der Antwort
  `trim_echo(text, previous_text, 8)` — **nur** wenn die Naht konditioniert war.
- **`Preparer::new` legt das Arbeitsverzeichnis nicht an**; `WorkingDir::create`
  ist Sache des Aufrufers. `search_ms < slice_ms` ist nur eine
  Debug-Assertion.
- **Der Zwischenraum an einer Naht ist offen.** `WorkingDir::assemble` verkettet
  die `.txt` byte-gleich ohne Trenner (Invariante 10), die Dateien enden
  absichtlich ohne Umbruch. Wer den Rohtext aus Fragmenten baut, muss den
  Trenner an der Naht setzen — heute tut das niemand; gehört zu #6/#7.

## Schritt 4 — die Spikes, Linux zuerst

| Reihenfolge | Issue | Was belegt wird |
|---|---|---|
| 1 | **#12 Linux-Spike** | Schale mit den wenigsten Fähigkeiten: GlobalShortcuts-Portal mit App-ID und `.desktop`, Tray vom Host, `insert_keystroke` unter Wayland **nein**; AppImage ohne Installation |
| 2 | **#13 macOS-Spike** | `RegisterEventHotKey`, `frontmostApplication`, TCC-Freigabe fürs Einfügen, `ConcealedType` im Pasteboard, Gatekeeper/Notarisierung |
| 3 | **#1 Windows-Spike** | `CF_HTML`-Umschlag und die drei Ausschlussmarker, Tray mit Overlay-Pille, `insert_after_dictation` in allen drei Modi — **ohne** Hook |

Windows 11 läuft in einer VM auf **think** (steht, noch ohne Gast). Geprüfte Lage:
i5-6500T (4 Kerne, `vmx`), `/dev/kvm` aktiv (Nutzer in `kvm`/`libvirt`, kein sudo
für `virsh`), qemu 10.2.1, libvirt 12.0, virt-manager 5.1, OVMF, swtpm, 227 GB
frei auf `rpool`, 24 GB RAM belegt davon 14 GB ZFS-ARC (vor dem Start
`zfs_arc_max` begrenzen).

**Audio ist der wunde Punkt.** `ich9-intel-hda` und `usb-audio` sind vorhanden,
aber #1 verlangt „30 s ohne Aussetzer" und „Gerätewechsel überlebt". Ein
emuliertes HDA-Gerät kann den Defekt **maskieren** oder einen erfinden. Für den
Audiobaustein entweder USB-Audio per Passthrough oder ein echtes Windows.

**Abnahmekriterium beantwortet** (2026-11-21, Word 365 Build 20625): `h1`–`h6`
bleiben echte Formatvorlagen „Heading 1"–„Heading 6", die Liste bleibt eine echte
Aufzählung, fett/kursiv bleiben zeichengenau; nur Fließtext wird „Normal (Web)".
Die Sorge aus der Grill-Recherche war unbegründet — Belege in `PITFALLS.md`,
Abschnitt „Windows".

Erkenntnisse danach nach `docs/ai/PITFALLS.md`.

## 🟡 In Arbeit — #12 Linux-Spike (2026-11-20, X11-Teil 2026-11-21 gemessen)

**X11 ist durch** (elite, Xubuntu 26.04.1, XFCE 4.20.7, X11 `:0`, Sitzung 8;
`cargo` 1.100.0, `cpal` 0.18.2; Wegwerfcode unter `/tmp/dw-linux-spike-elite/`
und `/tmp/dw-me/`, nichts davon im Repo). Alle Zahlen samt Fehlermeldungen stehen
in `PITFALLS.md`, Abschnitt „X11":

- **Hotkeys:** `XGrabKey` trägt (0,23–0,32 ms je Grab), Fremdbelegung meldet sich
  als `BadAccess` (10, major 33). Vier Masken je Kombination sind Pflicht, sonst
  ist der Hotkey bei CapsLock stumm. **`Ctrl+Alt+F<n>` ist tabu** — der Server
  behandelt es vor jedem Grab und ein XTEST-Anschlag schaltet die VT um.
- **Injection:** XTEST erreicht das fremde Fenster; Unicode tippt byte-identisch
  inkl. `ß`, `€`, `😀` — aber nur mit ~25 ms Serialisierung (~32 Zeichen/s), ohne
  sie fehlten 81 von 200 Bytes. Für die Lieferung zählt das nicht (sie schreibt
  die Zwischenablage und sendet ein Strg+V), aber es belegt die Fähigkeit.
  `_NET_ACTIVE_WINDOW` hat zwei Items, tote IDs kommen vor.
- **Zwischenablage:** eigener Eigentümer mit sechs Zielen, HTML und Reintext
  byte-gleich; `xfce4-clipman` ignoriert `x-kde-passwordManagerHint`.
- **Tray:** StatusNotifier (Watcher vom Host, sechs fremde Items); das Panel hat
  kein XEmbed-Systray.
- **Audio 30 s:** +0,009 % bzw. −0,024 % Frames, größte Callback-Lücke 12,3 ms,
  keine Streamfehler — dieselbe Größenordnung wie Windows.
- **Nebenbefund:** `/tmp` auf elite ist tmpfs; ein Reboot löscht den Wegwerfcode.

Gemessen auf **think** (Ubuntu 26.04.1, GNOME Shell 50.1, Wayland,
`xdg-desktop-portal` 1.21.1 mit Backend `xdg-desktop-portal-gnome` 50.0);
Wegwerfcode unter `/tmp/dw-linux-spike/`:

- **Frage 2 (GlobalShortcuts):** Portal und `host.portal.Registry` sind vorhanden
  (`version = 1`). Das App-ID-Gate ist schärfer als im Issue vermutet und steht als
  gemessener Befund in `PITFALLS.md`: der cgroup-Scope `app-<id>@…` **und** eine
  installierte `.desktop`-Datei sind beide nötig; ein `app_id`-Schlüssel in den
  Optionen wird ignoriert. Auf X11 gibt es das Interface **gar nicht**.
- **Offen und nicht ohne Mensch messbar:** die Erst-Einwilligung
  (`BindShortcuts`), ihr Bestand über einen Neustart, die Injection in echte
  Zielapps (Writer, Brave, GTK) und die Sichtprüfung des Tray-Symbols.
  think war zuletzt gesperrt (`IdleHint=yes`).
- **Host-Regel des Eigentümers (2026-11-20):** der Windows-Laptop ist für die
  **Windows**-Arbeit da; Linux-Messungen laufen auf einem Linux-Prüfstand, nicht
  per SSH vom Windows-Host aus. #19 auf claw war die Ausnahme (Provisioning).

Noch offen: Frage 2 über einen Neustart (Wayland), Frage 3 in echten Apps,
Frage 5/7 auf GNOME und die Sichtprüfung des Tray-Symbols auf elite.

## 🟡 In Arbeit — #1 Windows-Spike (2026-11-21, auf dem Windows-Laptop)

Wegwerf-Crate (`windows` 0.62.2, Rust 1.100.0), Word 365 Build 20625; alles außer
Audio ist gemessen:

| Baustein | Stand |
|---|---|
| Zwischenablage | **grün** — `CF_HTML` mit Byte-Offsets, Umlaute, Marker; in Word echte Überschriften und echte Aufzählung |
| Win+V-Ausschluss | **grün** — Text mit Markern erscheint nicht in der History, ohne Marker schon (WinRT `GetHistoryItemsAsync`, History dafür kurz ein- und wieder ausgeschaltet) |
| Hotkeys (ohne Hook) | **grün** — `RegisterHotKey`, 3 registriert, 3 ausgelöst; `Ctrl+Alt+F11/F12` sind fremdbelegt (1409) |
| Tray | **grün** — `NIM_ADD`/`GetRect`/`NIM_MODIFY`/`NIM_DELETE` |
| Textinjektion | **grün** — `SendInput` + `KEYEVENTF_UNICODE`, Emoji als Surrogatpaar, zeichengleich |
| Audio 30 s | **grün** — 48 kHz stereo → 16 kHz mono s16le, −0,035 % Frames, größte Lücke 15 ms |
| Gerätewechsel | **gemessen, negativ** — Anstecken invalidiert den Stream (`must be rebuilt`), Frames brechen nach 3,4 s ab; die Schale muss neu aufbauen |

Offen bleibt nur der Linux-Build des Crates (Windows-Abhängigkeiten) und die
Gegenprobe in der VM auf think.

Zwei Befunde betreffen die Architektur: die **Zwischenablage ist
sitzungsgetrennt** (Session 0 kann nicht in die Nutzersitzung schreiben), und
**ohne entsperrten Desktop injiziert `SendInput` nichts** (Winlogon hält dann den
Eingabe-Desktop). Beides samt Messwerten in `PITFALLS.md`, Abschnitt „Windows".
Der Spike-Code liegt unter `%TEMP%\opencode\dw-win-spike` und ist **nicht** im
Repo.

## Schritt 5 — hardwarefrei: #10 Konverter (#4 und #5 erledigt)

Alle drei (Domäne, DSP, Konverter) brauchen keine Hardware und kein Netz,
laufen also auch auf Linux — und die Toolchain steht überall, wo sie gebraucht
wird (claw, elite und seit #20 **think als Kern-Bauplatz**). Sie laufen parallel
zu den Spikes; die Reihenfolge der Spikes ändert sich dadurch nicht:
#12 → #13 → #1.

**Der nächste Bauabschnitt ist #10**, im selben Crate wie #4 und #5 (auf think,
`export PATH="$HOME/.cargo/bin:$PATH"` vor jedem `cargo`-Aufruf). #4, #5 und
#10 liegen alle in `crates/core`, also **sequenziell**, nicht parallel.

- **#4** ist erledigt (Schritt 5a oben): `diktat` und `prompt` gemäß `DOMAIN.md`,
  das Arbeitsverzeichnis, die zehn Invarianten mit je einem Test.
- **#5** ist erledigt (Schritt 5b oben): der DSP-Port, 73 portierte Tests.
- **#10** ist die reine Funktion `deliver(markdown) -> Delivery { html, text }`:
  constrained Markdown nach **semantischem Fragment** (nicht nach `CF_HTML` —
  der Umschlag ist Sache des Windows-Schreibers) und nach reinem Text. Neun
  Testfälle stehen im Issue; das Schreiben auf die Zwischenablage bleibt bei #8.

## Schritt 6 — der Rest

| Reihenfolge | Issue | Bedingung |
|---|---|---|
| 1 | #6 STT/LLM-Client | braucht #1 nur zum manuellen Test gegen die echte API |
| 2 | #7 Workbench | braucht #6 (Prompt anwenden) und #4 (FIFO-Liste) |
| 3 | #8 Fast Path | braucht die Fähigkeitsleiter (Testbarkeit) und #4; die Hardware belegt nur noch die Plattform |
| 4 | #9 Paketierung Windows | NSIS; Linux (#17, AppImage/.deb nach E7) und macOS (#18, `.dmg`/Notarisierung) folgen den Schalen; braucht #7 und #8 |

Die Plattformschalen entstehen nach den Spikes: **#14 Linux**, **#15 macOS**,
**#16 Windows**. #14 kann parallel zu #12 beginnen; #16 erst, wenn #1 belegt ist.

## Offene Entscheidungen

- **Auslöser der Lieferung beim Bearbeiten.** Die Regel ist entschieden
  (beide Reiter editierbar, Verlassen liefert bedingungslos; `DOMAIN.md`,
  Abschnitt „Ablauf: Bearbeiten und Liefern", und `DECISIONS.md` vom
  2026-11-19). Offen ist nur das **Ereignis** auf dem Desktop, der keinen
  Edit-Modus kennt: Fokusverlust, Fensterschließen oder ein Knopf. Betrifft
  #7 (Workbench) und die Abnahmekriterien von #8 (Lieferung). Mit dem Knopf
  **In die Zwischenablage** (2026-11-19) ist ein falsch geratener Auslöser nicht
  mehr blockierend: der Nutzer kann jederzeit selbst liefern.
- **Hotkey-Mechanik — entfällt.** Der alte Eintrag lautete: „Leertaste/Enter mit
  eigenem Hook (dann bleiben der Hook-Block in #1 und das Abnahmekriterium
  „Space im Pausenzustand" in #8) oder ein dritter Hotkey für Pause/Fortsetzen
  ohne Hook? Hängt an der Frage, ob Push-to-talk gebraucht wird." Beantwortet am
  2026-11-16: **drei globale Hotkeys, kein Hook**, Space und Enter nur im
  **Diktierplatz** (E3). Push-to-talk fällt weg, weil ohne Hook kein Key-up
  existiert. Die Abnahmekriterien von #1 (Hook) und #8 (Space schlucken) sind
  damit gegenstandslos; der Orchestrator zieht sie nach.
- **Einfügen** ist als Einstellung entschieden (`insert_after_dictation` mit
  `never`/`same_window`/`always`); die **Plattform-Defaults** folgen jetzt der
  Fähigkeitsleiter — `same_window` ohne `foreground_window` wird `never`, und in
  das eigene Fenster wird nie eingefügt (E2, E3, E4). Offen bleibt **der Name**
  des Schlüssels (gewählt, nicht abgestimmt) — eine Zeile in `config.json` bzw.
  ein Bezeichner im Code.
- **#9 Installer und Update-Kanal** — offen nur noch für Windows: NSIS allein
  oder MSI zusätzlich für Enterprise-Verteilung; unsigniert mit
  SmartScreen-Warnung oder mit Zertifikat. Linux ist mit AppImage primär und
  `.deb` zweitens entschieden (E7), macOS folgt aus der Notarisierung.
- **App-ID endgültig** — vorläufig `at.priv.graf.Diktatwerkstatt` (E8). Endgültig
  mit der Namens-/Domainfrage; die Tür schließt sich mit dem ersten öffentlichen
  Build, weil die Portal-Bindungen an der ID hängen.
- **§, Aktenzeichen und Beträge** — bewusst vertagt. Befund: Whisper hat keine
  Inverse Text Normalization, also wird „Paragraph" zum Wort und nicht zum
  Zeichen; für Aktenzeichen gibt es weltweit keine Messung. In v0.1 ist das eine
  Prompt-Aufgabe. Ob es je eigene Eingabefelder braucht, entscheidet sich nach
  dem ersten echten Protokoll.
- **Wiederaufnahme mit Rückfrage** statt automatisch (heute: automatisch).
- **Markenrecherche** EUIPO/AT für den Namen, vor der ersten Veröffentlichung.

Der Umsetzungsplan für die Einfüge-Entscheidung und den Konverter liegt
außerhalb des Repos unter
`~/.opencode/plan/diktatwerkstatt-einfuegen-und-cf-html.md` (Stand
2026-11-16); der Inhalt ist in `DOMAIN.md`, `DECISIONS.md` und #10 abgebildet,
der Plan ist nur die Lesefassung.

## Bekannte Fallen

Die Fallstricke aus Audio, STT, Textübergabe, Bedienung, **Linux/Wayland** und
**macOS** stehen in [PITFALLS.md](PITFALLS.md) — dort mit Belegen. Lies sie,
bevor du einen der Bausteine anfasst; sie sind die Kurzfassung von drei
Recherchen, zwei Vorgängerprojekten und dem Provisioning-Befund von claw.

## Umgebung

- Entwicklung auf Linux (`cargo`, Tauri-CLI, WebKitGTK); Windows-Test in der VM
  auf think, macOS-Test braucht echte Apple-Hardware.
- Kein Code-Signing-Zertifikat vorhanden — weder für Windows noch für macOS.
- WebView2 ist auf Windows 10/11 vorinstalliert; im Installer prüfen, nicht
  mitliefern.
- Auf **claw** ist die Toolchain eingerichtet (#19, 2026-11-20: Rust 1.100.0 mit
  `clippy`/`rustfmt`, Tauri-Abhängigkeiten, `appimagetool`). **think** ist seit
  2026-11-21 der **Kern-Bauplatz** (#20: Rust 1.100.0 mit `clippy`/`rustfmt`,
  `libasound2-dev`, 4 Kerne, 30 GB RAM — ohne Tauri-`-dev`-Pakete). **elite** hat
  seit 2026-11-21 ebenfalls Rust 1.100.0 mit `clippy`/`rustfmt` und
  `libasound2-dev` (#12, ohne Tauri-`-dev`-Pakete); **dell** bleibt der
  Desktop-Prüfstand für echte Hotkeys, Tray und AppImage.
- **elite erreicht man über `ssh elite`** — der Alias zeigt seit 2026-11-21 auf
  `10.8.0.3` (VPN-Pool-Adresse, war vorher fälschlich `10.8.0.24`). Sein `/tmp`
  ist tmpfs: nach einem Reboot ist der Spike-Code weg.
- Der **Windows-Laptop** hat Rust 1.100.0 und MSVC 14.44 und ist damit für die
  Windows-Arbeit tauglich. Achtung: der Agent läuft dort in **Session 0** — jede
  Messung, die Zwischenablage, Hotkeys, Tray oder Injektion betrifft, muss in die
  Nutzersitzung gestartet werden (Dienst oder SYSTEM-Task mit
  `SeTcbPrivilege`), sonst misst sie nichts.
