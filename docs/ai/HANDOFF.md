---
type: Handoff
title: Übergabe
description: Was die nächste Sitzung zuerst tut, in welcher Reihenfolge, und was schon bekannt ist.
tags: [wissensbasis, handoff]
generated: { by: opencode/general, at: 2026-11-20T16:23:16Z }
status: stable
---

# Handoff

Arbeitsverzeichnis: `~/repos/georgernstgraf/diktatwerkstatt`
Repository: https://github.com/georgernstgraf/diktatwerkstatt
Stand: **Doku-Phase, kein Code.** Branch `main`, Arbeitskopie sauber. Beschlossen
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

## 🚦 Schritt 3 — Provisioning (#19), der erste Schritt

Ohne Toolchain gibt es keinen grünen `cargo`-Commit, und `AGENTS.md` verlangt ihn
vor jedem Commit. Befund auf **claw** (2026-11-16):

| Punkt | Befund |
|---|---|
| Rust | `~/.cargo` und `~/.rustup` fehlen, `cargo` nicht im PATH ⇒ rustup installieren |
| Build-Abhängigkeiten | `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `patchelf`, `libfuse2`, `appimagetool` |
| sudo | passwortlos verfügbar |
| Maschine | Ubuntu 26.04.1, 2 Kerne, 3 GB RAM, 54 GB frei |

Arbeitsteilung: **claw** baut den Kern (headless, `cargo test`, kein Display),
**dell** (Desktop) baut Schale, Hotkeys, Tray und AppImage — dort gibt es echte
Hotkeys und eine echte Tray-Probe.

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

**Neues Abnahmekriterium** (aus der Grill-Recherche): Word 365 mit Build ≥ 2405,
Einfügen aus einem anderen Programm, Standard „Merge Formatting" — bleiben
`h1`–`h6` echte Überschriften-Formatvorlagen oder werden sie zu Fließtext?

Erkenntnisse danach nach `docs/ai/PITFALLS.md`.

## Schritt 5 — hardwarefrei parallel: #4 Domäne, #5 DSP-Port, #10 Konverter

Alle drei brauchen keine Hardware und kein Netz, laufen also auch auf Linux und
können neben dem Provisioning beginnen.

- **#4** hat jetzt ein sehr kleines Ziel: `diktat` und `prompt` gemäß
  `DOMAIN.md`, das Arbeitsverzeichnis mit `NNN.wav`/`NNN.txt`, die zehn
  Invarianten — jede mit einem Test, der sie bricht, wenn man sie entfernt.
- **#5** hat einen fertigen Prüfstein: die Tests aus
  `polished-recognition/:core` und `aitranscribe/tests/test_fragmenter.py`. Sie
  werden portiert, nicht neu erfunden.
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
| 4 | #9 Paketierung | Windows NSIS, Linux AppImage/.deb (E7), macOS `.dmg`; braucht #7 und #8 |

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
- Auf claw fehlen Toolchain und Build-Abhängigkeiten; #19 (Provisioning) ist
  deshalb der erste Schritt.
