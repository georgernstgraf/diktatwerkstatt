# Persistenz-Log

## 2026-11-21 — Windows-Spike: Word, Hotkeys, Tray, Sitzungsgrenzen gemessen (#1)

* **Messung**: Wegwerf-Crate auf dem Windows-11-Laptop (`windows` 0.62.2),
  Word 365 Build 20625. Zwischenablage (`CF_HTML` mit Byte-Offsets, Umlaute,
  drei Marker) **grün**; Word macht aus `<h1>`/`<h2>` echte Formatvorlagen
  „Heading 1/2" und aus `<ul>` eine echte Aufzählung — die Grill-Sorge war
  unbegründet. `RegisterHotKey` global ohne Fenster: 3 von 3 ausgelöst
  (`gesendet=3 empfangen=3`), `Ctrl+Alt+F11/F12` fremdbelegt (1409). Tray über
  `Shell_NotifyIcon` inkl. `GetRect`-Beweis grün. `SendInput` mit
  `KEYEVENTF_UNICODE` tippt 32 Zeichen samt `äöüß` und Emoji zeichengleich.
  Zwischenablage-History war aus: Win+V-Ausschluss **nicht** gemessen.
* **Umgebungsbefunde**: die Zwischenablage ist **sitzungsgetrennt** (Session 0
  schreibt nicht in die Nutzersitzung); ohne **entsperrten** Desktop injiziert
  `SendInput` nichts (Winlogon hält den Eingabe-Desktop). Beides erklärt, warum
  die ersten Läufe 0 Events lieferten.
* **Update**: [PITFALLS.md](PITFALLS.md) — neuer Abschnitt **Windows**; die
  Aussage „Überschriften verflachen unter Merge Formatting" ist widerlegt und
  ersetzt. [HANDOFF.md](HANDOFF.md) — Abnahmekriterium beantwortet, #1-Stand als
  Tabelle, Umgebung um den Windows-Laptop und die Session-0-Regel ergänzt.
  [index.md](index.md) — Beschreibung nachgezogen.
* **Offen**: Audio (`cpal` 30 s, Gerätewechsel) und der Win+V-Ausschluss; beides
  braucht die VM auf think oder eine eingeschaltete History. #1 bleibt offen.

## 2026-11-20 — Provisioning auf claw erledigt (#19)

* **Erledigt**: claw ist provisioniert und verifiziert — rustc/cargo **1.100.0**,
  `clippy` und `rustfmt` nachgezogen (im Profil `minimal` nicht enthalten),
  Tauri-Abhängigkeiten und `libasound2-dev` per apt, `appimagetool` in
  `~/.local/bin`; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`
  und `cargo test` laufen dort grün, der cpal-Smoke-Build ebenso.
* **Update**: [HANDOFF.md](HANDOFF.md) — Schritt 3 auf „erledigt" mit
  Prüfstand-Tabelle und der Host-Regel, dass Linux-Messungen nicht vom
  Windows-Laptop aus laufen; neuer Abschnitt „In Arbeit — #12" mit dem ersten
  gemessenen Befund. [STATE.md](STATE.md) — claw provisioniert, #12 als nächster
  Schritt, Issue-Landschaft ohne #19. [PITFALLS.md](PITFALLS.md) — App-ID-Gate aus
  dem cgroup-Scope **gemessen** (statt „stumm"), `Registry.Register`, Laufzeit- vs.
  Entwicklungs-Paket bei ALSA, `--profile minimal`, Non-Login-PATH, 3-GB-RAM.
  [../../AGENTS.md](../../AGENTS.md) — Verifikationsabschnitt auf den Stand von #19.
* **Verworfen**: `--profile minimal` ohne `clippy`/`rustfmt`; `libfuse2` und
  `libasound2` unter den alten Namen suchen; Provisioning auch auf think/dell.

## 2026-11-20 — Reihenfolge bestätigt: Provisioning zuerst, dann Spikes und Kerne parallel (#19)

* **Update**: [HANDOFF.md](HANDOFF.md) — Schritt 5 beginnt nicht mehr „neben dem
  Provisioning", sondern **nach** #19 und läuft dann parallel zu den Spikes; die
  Reihenfolge selbst bleibt #19 → #12 → #13 → #1. Der Grund: ohne `cargo` gibt es
  keinen grünen Commit. [STATE.md](STATE.md) — der Abschnitt „Verifikation" sagte
  die Reihenfolge zweimal verschieden (einmal Kerne vor den Spikes, einmal
  parallel); jetzt einmal: #19 zuerst, danach #4/#5/#10 parallel zu
  #12 → #13 → #1. [../../AGENTS.md](../../AGENTS.md) — „hardwarefrei parallel
  #4/#5/#10" an beiden Stellen um „nach dem Provisioning" ergänzt.
* **Verworfen**: #4/#5/#10 schon während des Provisionings anfangen (kein
  `cargo`, also kein grüner Commit, den `AGENTS.md` verlangt); die Spikes hinter
  die Kerne schieben (dann bremsen die Hardware-Befunde den ganzen Strang).

## 2026-11-20 — Drei Ziele, Hook-freie Bedienung, Fähigkeitsleiter (#11)

* **Creation**: Beschlusslage E1–E10 des Eigentümers (2026-11-16) in die Doku
  eingearbeitet. Drei Ziele (Windows, Linux, macOS), ein Kern, drei dünne
  **Plattformschalen**; **kein globaler Tastenhook** — drei globale Hotkeys
  (diktieren roh, diktieren poliert, pausieren/fortsetzen), Space und Enter nur
  im **Diktierplatz**; **Fähigkeitsleiter** statt Plattformweichen im Kern;
  Zwischenablage-Ausschluss dreistufig; `Delivery { html, text }` trennt Fragment
  und `CF_HTML`-Umschlag; App-IDs `at.priv.graf.Diktatwerkstatt` (vorläufig);
  Name bleibt `diktatwerkstatt`.
* **Update**: [DECISIONS.md](DECISIONS.md) — zehn Einträge E1–E10 obenauf; die
  Einträge „Rust-Kern + Tauri/WebView2", „Einfügen ist eine Einstellung",
  „Semantisches HTML auf der Zwischenablage" und „Politur pro Diktat, über zwei
  Hotkeys" mit Amendierungsabsatz. [CONTEXT.md](../../CONTEXT.md) —
  **Plattformschale**, **Fähigkeitsleiter**, **Diktierplatz**, **Diktier-Hotkey**,
  **Pause-Hotkey** neu; **Zwischenablage** = Systemzwischenablage; Übergangstabelle
  `RECORDING→PAUSED` mit zwei Auslösern. [DOMAIN.md](DOMAIN.md) — „Lieferung"
  dreiteilig (gemeinsam / Windows / macOS und Linux) mit Ausschluss-Tabelle,
  „Ablauf: roh" Schritt 5 neu, neue Sektion **Fähigkeitsleiter**, Einfüge-Regel
  „nie ins eigene Fenster"; die zehn Invarianten unverändert.
  [PITFALLS.md](PITFALLS.md) — Hook-Passagen als „entfällt" markiert, neue
  Abschnitte **Linux und Wayland** und **macOS**. [INTEGRATION.md](INTEGRATION.md)
  — Linux- und macOS-Spalte, Korrektur Nr. 1 als „entfällt", `handy-keys` als
  Abgrenzung. [../../README.md](../../README.md) — „Windows-Hilfsprogramm" →
  drei Ziele, ein Kern. [../../AGENTS.md](../../AGENTS.md) — Stack „Tauri mit drei
  Webviews", Ausschlussregel mit Linux-Ausnahme, Status Doku-Phase, Issues
  #1/#4–#19. [HANDOFF.md](HANDOFF.md) und [STATE.md](STATE.md) — Reihenfolge
  Provisioning (#19) → Linux-Spike (#12) → macOS-Spike (#13) → Windows-Spike (#1).
  [index.md](index.md) — Beschreibungen nachgezogen.
* **Entfällt (markiert, nicht gelöscht)**: „Taste schlucken" via `WH_KEYBOARD_LL`
  und das eigene Modul (~300 Zeilen); „der Low-Level-Hook ist bei fokussierter
  Admin-App blind" samt „für Push-to-talk den Hook"; „Space darf nur im Zustand
  `RECORDING` geschluckt werden"; „Zwischenablage — die Windows-Zwischenablage";
  „Leertaste … wird nur in `RECORDING` geschluckt"; die offene Hotkey-Mechanik
  und Push-to-talk in `HANDOFF.md`; „WebView2" als Stackname.
* **Verworfen**: Windows-only; Windows+Linux ohne macOS; Electron; getrennte Apps
  je Plattform; Plattformweichen im Kern (`cfg(target_os)`, `enum Platform`,
  Feature-Flags); `WH_KEYBOARD_LL`; `evdev`/`/dev/input`; Hook nur unter Windows;
  Push-to-talk; `always` als Ersatz für fehlendes `foreground_window`; Ausschluss
  als „überall gelöst"; alles in einem `cf_html()`; nur `.deb` oder nur AppImage;
  Flatpak.
* **Offen**: Reihenfolge der Auslieferung (Architektur ja, Termin nein);
  App-ID endgültig mit der Namensfrage; Desktop-Auslöser von „verlassen" für die
  Bearbeiten-Lieferung; Windows-Installer (NSIS allein oder MSI zusätzlich).
* **Drift gebogen**: `AGENTS.md` verlangte `cargo fmt/clippy/test` vor jedem
  Commit, während auf claw weder `cargo` noch `rustup` existiert
  (Provisioning-Befund 2026-11-16). Die Toolchain-Lücke steht jetzt in
  `AGENTS.md` und `STATE.md` und ist #19 zugeordnet.
* **Verifikation**: zehn Prüfungen aus #11 — Doku-Diff, neue Begriffe in
  `CONTEXT.md`, „Windows-Zwischenablage", „geschluckt", `WH_KEYBOARD_LL`,
  `cfg(target_os)`, E1–E10, „Windows-only"-Reste, OKF-Envelopes, Log-Eintrag —
  alle grün.
* **Korrektur**: Issue-Landschaft in `AGENTS.md`, `STATE.md` und `HANDOFF.md`
  auf #1 und #4–#19 vervollständigt — die Plattformschalen #14–#16 und die
  Paketierung #17/#18 fehlten. Der Belegstatus der Linux-/macOS-Fallstricke ist
  in `index.md` und `STATE.md` präzisiert.
* **Abschluss**: Issue #11 geschlossen; #19 als Sub-Issue des Epics verlinkt,
  #3-Body auf die Drei-Ziele-Lage gezogen.
* **Korrektur**: #19 (Provisioning) gilt nur für **claw**; dell bleibt der
  Desktop-Prüfstand für echte Hotkeys und Tray, ohne Provisioning.

## 2026-11-19 — Kopier-Knopf als zweiter Weg

* **Creation**: Knopf **„In die Zwischenablage"** in der Workbench — liefert den
  sichtbaren Reiter jederzeit (ohne Bearbeitung, ohne Verlassen, ohne Einfügen),
  mit History-Ausschluss wie jede andere Lieferung. Dieselbe Regel im
  Polished-Editor (#166).
* **Update**: [DECISIONS.md](DECISIONS.md) — neuer Eintrag (24, „Kopier-Knopf als
  zweiter Weg"), 23 aus dem Eintrag darunter bleibt bestehen. [DOMAIN.md](DOMAIN.md)
  — Invariante 8 nennt jetzt vier Auslöser; „Ablauf: Bearbeiten und Liefern" um
  den Knopf erweitert. [CONTEXT.md](../../CONTEXT.md) — Begriff **In die
  Zwischenablage** neu, **Lieferung** und **Workbench** nachgezogen.
  [AGENTS.md](../../AGENTS.md) — Projektabsatz und Ausschlussregel: der Knopf
  ist kein bewusstes „Transkript kopieren", also bleibt die History aus.
* **Drift gebogen**: Der Eintrag von heute früh („Bearbeiten liefert") führte
  „Kein Kopier-Knopf" als Abgrenzung und „Lieferung nur auf Knopfdruck" als
  verworfen. Richtig ist: verworfen bleibt der Knopf als **einziger** Weg; der
  Knopf **als zweiter** Weg ist jetzt entschieden. Die Abgrenzung in
  `CONTEXT.md` ist ersetzt, der alte Log-Eintrag bleibt als Chronik stehen.
* **Verifikation**: `okf-check --root docs/ai` grün.

## 2026-11-19 — Bearbeiten liefert in die Zwischenablage

* **Creation**: Regel „Bearbeiten liefert" — beide Workbench-Reiter sind
  editierbar; wer eine bearbeitete Ansicht verlässt, schreibt ihren Text
  bedingungslos in die Zwischenablage (kein Dialog, kein Zurücklegen des alten
  Inhalts); maßgeblich ist die zuletzt bearbeitete Ansicht; kein Einfügen.
* **Update**: [DECISIONS.md](DECISIONS.md) — neuer Eintrag vom 2026-11-19 (23),
  „Zwischenablage genau einmal, atomar" um den Erweiterungssatz ergänzt.
  [DOMAIN.md](DOMAIN.md) — Invariante 8 von „genau einmal pro Diktat" auf „hält
  immer den Text, der im Fenster steht" umgestellt; neuer Abschnitt „Ablauf:
  Bearbeiten und Liefern". [CONTEXT.md](../../CONTEXT.md) — **Bearbeiten** neu,
  **Lieferung** und **Workbench** nachgezogen, Abgrenzung „Kein Kopier-Knopf".
  [HANDOFF.md](HANDOFF.md) — offener Desktop-Auslöser. [STATE.md](STATE.md) —
  Entscheidungszahl 23.
* **Verworfen**: Lieferung nur auf Knopfdruck, Rückfrage vor dem Überschreiben,
  alten Inhalt aufheben, Bearbeiten nur im polierten Reiter, Poliert-Reiter
  einfrieren.
* **Offen**: das Ereignis „verlassen" auf dem Desktop (Fokusverlust,
  Fensterschließen oder Knopf) — auf Android ist es die geschlossene Tastatur.
* **Grenze**: Android bleibt bei `polished-recognition` (#166), mit getrenntem
  Code (Entscheidung 2026-11-15); die Lieferregel gilt dort sinngemäß.
* **Verifikation**: Dokumentenkonsistenz geprüft (kein Widerspruch in
  `CONTEXT.md`/`DOMAIN.md`/`DECISIONS.md`); `okf-check --root docs/ai` grün
  (6 Konzepte, 0 Warnungen).

## 2026-11-16 — Sitzungsschluss: offene Fragen der Einfüge-Entscheidung

* **Update**: [HANDOFF.md](HANDOFF.md) — Schritt 3 nennt jetzt #10 (Konverter)
  als dritten Linux-Baustein; die offenen Entscheidungen sind aufgeteilt in
  **Hotkey-Mechanik** (Hook vs. dritter Hotkey, hängt an Push-to-talk),
  **Einfügen** (entschieden als Einstellung; offen sind Default `same_window`
  und der Schlüsselname `insert_after_dictation`) und **#9 Installer/Signatur**;
  Zeiger auf die Lesefassung des Umsetzungsplans.
* **Update**: [STATE.md](STATE.md) — Nächster Schritt und Verifikationsabsatz
  nennen #4 und #10 als erste `cargo`-Commits.
* **Verworfen**: nichts; die Superseded-Einträge dieses Tages stehen im
  vorherigen Eintrag.
* **Verifikation**: `okf-check --root docs/ai` grün.

## 2026-11-16 — Einfügen als Einstellung, Lieferung als CF_HTML

* **Creation**: [DOMAIN.md](DOMAIN.md) um `### CF_HTML` (Kopfzeile,
  Byte-Offsets, Fragment-Marker) erweitert; Invariante 8 um das optionale
  Einfügen und den Ausschluss bei **Prompt anwenden**.
* **Update**: [DECISIONS.md](DECISIONS.md) um „Einfügen ist eine Einstellung"
  (drei Werte, Default `same_window`, keine UIPI-Prüfung, keine
  Zielfenster-Reaktivierung); [CONTEXT.md](../../CONTEXT.md) um **Einfügen** und
  **Lieferung**; [INTEGRATION.md](INTEGRATION.md) von der Einfüge-Maschinerie
  befreit; [PITFALLS.md](PITFALLS.md) korrigiert; `README.md`-Abgrenzung
  angepasst. Neues Sub-Issue #10 (Markdown → `CF_HTML`) an #3.
* **Verworfen**: Delayed Render, Format-Snapshot, Read-Receipt, UIPI-Meldung,
  Zielfenster-Reaktivierung — sie setzen ein Zurücklegen der Zwischenablage
  voraus, das es nicht gibt.
* **Drift gebogen**: [STATE.md](STATE.md) behauptete, `PITFALLS.md` existiere
  nicht; [HANDOFF.md](HANDOFF.md) trug „Datei anlegen" und vier offene Fragen,
  von denen zwei entschieden sind.
* **Verifikation**: `okf-check --root docs/ai` grün.

## 2026-11-16 — Projektregeln aus opencode-helpers, Fallen nach PITFALLS.md

* **Creation**: [PITFALLS.md](PITFALLS.md) — die Fallstricke aus Audio/STT, Text
  und Einfügen sowie Bedienung, jetzt mit `sources` und Fußnoten statt als
  Prosa-Block im Handoff.
* **Update**: [HANDOFF.md](HANDOFF.md) auf das reduziert, was sein Name sagt
  (wo man anfängt); [DECISIONS.md](DECISIONS.md) um „Korrigieren heißt Hören
  reparieren" ergänzt (Prinzip aus dem `protokoll`-Skill).
  `AGENTS.md` um Knowledge Bootstrap, Skill-Trigger, gh-Konventionen,
  Repository/Kontakt und Projektidentität erweitert.
* **Verworfen**: `CONVENTIONS.md` und `ARCHITECTURE.md` vorerst anzulegen (kein
  Abnehmer — `ARCHITECTURE.md` erst mit dem ersten Code); sichtbare Flag-Marker
  im polierten Transkript (im Brief unbrauchbar).
* **Verifikation**: `okf-check --root docs/ai` grün.

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
