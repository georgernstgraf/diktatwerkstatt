---
type: Decisions
title: Entscheidungen
description: Getroffene Architektur- und Produktentscheidungen mit Begründung.
tags: [entscheidungen, architektur]
generated: { by: opencode/general, at: 2026-11-21T08:40:06Z }
status: stable
---

# Entscheidungen

Neueste oben. Jeder Eintrag nennt Entscheidung, Grund und verworfene Alternative.

## 2026-11-21 — Fragmentierung: Slice-Modell, Q-Gate, Naht ohne Seitendatei

**Entscheidung:** Der Kern schneidet nach dem **Slice-Modell** von
`polished-recognition`: Die Slice (Default 21 s) ist die **Obergrenze** eines
Fragments, geschnitten wird nur in den letzten 30 % (Suchfenster, 6,3 s), und
entschieden wird erst, wenn die ganze Slice aufgenommen ist. Der Schnitt liegt
im **Flächenzentroid** des besten Stillekandidaten `Q` (Achse `X = 0,65·M + 100`,
Rand 150 ms geklemmt); ohne Kandidaten am leisesten 10-ms-Rahmen (`Q = 0`). Die
Naht wird nach dem **Q-Gate** konditioniert (`Q < 10` oder ungemessen), und die
drei Reparaturen — Pre-roll (1 s), Prompt-Carry-over (≤ 96 Zeichen,
wortgrenzenbewusst), Echo-Trim (≤ 8 Token) — liegen im Kern. Der Pre-roll ist
**keine Datei**, sondern der letzte Teil des Vorfragments im Speicher; ein
Fragment ist genau eine `NNN.wav`, lückenlos dreistellig, ab 1000 bricht die
Transkription ab.
**Grund:** Die Obergrenze folgt der Messung (20–30 s ist das stabile
Whisper-Fenster: 30 s → 4,76 % WER, 45 s → 34,59 %, 60 s → 51,47 %). Die
Untergrenze 14,7 s schadet nicht, weil ein Fragment nur dann kürzer als 21 s
ist, wenn im Suchfenster wirklich eine Pause liegt. Das Q-Gate ist die Lehre
aus dem Unfall „bist. bist.", bei dem ein stilleausgerichteter Schnitt mitten
im Wort lag: Der Schnitt bleibt best-effort, die Naht ist die Garantie — und
beide hängen an derselben Entscheidung (`Committed::stitched`). Der Pre-roll im
Speicher hält das Arbeitsverzeichnis ehrlich („eine Datei je Fragment") und ist
ohnehin vorhanden: er ist der letzte Teil des Vorfragments, das der STT-Client
zum Transkribieren liest.
**Verworfen:** das Vorwärtsfenster aus `aitranscribe` (Fragment mindestens
Fragmenttakt, bis +5 s ⇒ Obergrenze 26 s und mehr, zu nah an der Klippe);
Pre-roll als `preroll_NNN.wav` (verdoppelt die Dateien, bricht „nur `.wav` heißt
offen" und wäre nach dem Löschen des Vorfragment-Audios ohnehin verloren);
Konditionieren nur bei erzwungenen Schnitten (`FORCED_ONLY`; der Unfall beweist,
dass ein stilleausgerichteter Schnitt genauso schief liegen kann); PCM im Kern
als Little-Endian-Bytes (Byte-Offsets sind eine Frage der WAV-Datei, die Analyse
rechnet auf `i16`); eine Filterbibliothek (der Einpoler ist vier Zeilen, und die
Quantisierung auf `i16` ist Teil der Rechnung, nicht ein Detail).

## 2026-11-21 — Der Abschluss ist im Code zweistufig, das Schema ist versioniert

**Entscheidung:** `Store::finish_transcription` ist der Weg, den die App geht;
darunter stehen zwei öffentliche Schritte: `commit_raw_text` (Invariante 3,
Schritt 1) und `remove_working_dir` (Schritt 2), ersterer in einer eigenen
Transaktion. Das Schema ist `STRICT`, trägt `PRAGMA user_version = 1`, und ein
neueres Schema wird mit einem typisierten Fehler abgelehnt; WAL läuft mit
`synchronous = FULL`. Das Diktatverzeichnis entsteht **vor** dem Commit des
Diktats; die Verzeichnisse verdrängter Diktate verschwinden **nach** dem Commit.
**Grund:** Text-vor-Audio (Invarianten 2 und 3) ist nur dann eine Zusage, wenn
der Commit die Platte erreicht und wenn ein Absturz zwischen den beiden Schritten
nachstellbar ist. Mit zwei Schritten ist der Abbruch ein Test, der genau den
Zustand herstellt, den die Wiederaufnahme vorfindet — ohne Testhaken im
Produktivpfad. Die Reihenfolge beim Anlegen und Verdrängen hält den einen
Zustandsmarker ehrlich: ein Verzeichnis ohne Diktat ist Müll, ein Diktat mit
gesetztem `audio_dir` ohne Verzeichnis wäre eine Lüge.
**Verworfen:** eine einzige Abschlussfunktion mit `#[cfg(test)]`-Haken (die
Integrationstests sehen ihn nicht, also gäbe es keinen Test für Invariante 3);
`synchronous = NORMAL` (schneller, aber „committet" wäre eine Behauptung);
Verzeichnisse vor dem Commit zu löschen (ein Rollback hinterließe Diktate, deren
Audio fehlt, obwohl sie offen aussehen).

## 2026-11-20 — E1 Drei Ziele, ein Kern

**Entscheidung:** Ein geteilter Rust-Kern, drei dünne Plattformschalen: Windows,
Linux, macOS. Die **Reihenfolge der Auslieferung bleibt offen** — die Architektur
ist festgelegt, der Termin nicht. „macOS in v0.1 oder danach" ist damit eine
Aufwandsfrage, kein Umbau.
**Grund:** Der größte Teil des Codes ist plattformfrei — Audio, Fragmente, Naht,
STT-/LLM-Client, FIFO, Markdown → Fragment, `Delivery`. Wer ihn an Windows
bindet, lässt die VM Torwächter für alles spielen, obwohl nur drei Bausteine sie
brauchen. Es ist dasselbe Muster, mit dem TMOG aus einer Spezifikation entstanden
ist: ein geteilter kompilierter Kern plus dünne Schalen.
**Verworfen:** Windows-only (die VM bleibt Torwächter für alles, obwohl der
größte Teil des Codes plattformfrei ist); Windows+Linux ohne macOS (die dritte
Schale wäre ein Umbau); Electron (Chromium im Binary, gegen den
Footprint-Grund der Stack-Entscheidung); getrennte Apps je Plattform (drei
Doku-Sätze, drei Datenmodelle).

## 2026-11-20 — E2 Der Kern fragt Fähigkeiten ab, nicht Plattformen

**Entscheidung:** Kein `cfg(target_os)`, kein `enum Platform` in der Domäne. Die
Schale meldet beim Start einen **Fähigkeitssatz**, der Kern leitet sein Verhalten
daraus ab:

```
global_hotkey        ja / nein
foreground_window    ja / nein        ← entscheidet über same_window
insert_keystroke     ja / nein / ja-mit-freigabe
clipboard_envelope   cf_html | html_fragment
clipboard_exclusion  api | convention | keine
tray                 ja / mit-erweiterung
autostart, secret_storage, audio_host
```

**Degradationsregel:** was die Plattform nicht kann, fällt auf die **sichere
Seite** — `same_window` ohne `foreground_window` wird `never`, nie `always`. Nie
einzufügen ist immer richtig, der Text liegt in der Zwischenablage.
**Grund:** Die Schichtung bleibt erzwingbar: `crates/core` kompiliert und testet
auf jeder Plattform, weil es keine kennt. Nebeneffekt: #8 (Fast Path) wird ohne
Hardware testbar — eine Fake-Plattform mit
`{foreground_window: nein, insert_keystroke: nein}` ist Wayland, eine mit
`{ja, ja}` ist X11 oder Windows. Die Fähigkeitsleiter wird zur Testsuite statt
zur Behauptung.
**Verworfen:** Plattformweichen im Kern (zerstört die Schichtung, die
`cargo test` auf Linux erst möglich macht); `enum Platform` (jedes vierte Ziel
ist ein Umbau); Feature-Flags pro Betriebssystem (dieselbe Weiche mit anderem
Namen).

## 2026-11-20 — E3 Kein globaler Tastenhook — zwei Scopes statt einer geschluckten Taste

**Entscheidung:** Nicht fokussiert steuern **drei globale Hotkeys**: `diktieren
roh`, `diktieren poliert`, `pausieren/fortsetzen`. Fokussiert (Workbench im
Vordergrund) gelten zusätzlich Space (Pause) und Enter (Ende) — im eigenen
Fenster abgefangen, nicht systemweit.

- Der Diktier-Hotkey ist ein **Umschalter**: erster Druck startet, zweiter
  beendet. Enter gibt es nur im Fenster. Ohne Hook gibt es kein Key-up, also kein
  Push-to-talk — auf keiner Plattform.
- Während einer Aufnahme **beenden beide** Diktier-Hotkeys. Das Moduspaar
  (roh/poliert) wird beim Start entschieden, nicht mitten im Diktat gewechselt.
- **Nie ins eigene Fenster einfügen:** ist die Workbench das Vordergrundfenster,
  unterbleibt das Einfügen, auch bei `always`.

**Grund:** Der riskanteste Windows-Baustein fällt weg (`WH_KEYBOARD_LL`, ~300
Zeilen, Re-Install nach Session-/Power-Events, blind bei fokussierter Admin-App)
und mit ihm der einzige unter Wayland nicht baubare Baustein. Zwei dokumentierte
Fallstricke werden zu Konstruktionsunmöglichkeiten: „Space darf nur in
`RECORDING` geschluckt werden" und „der Hook ist bei Admin-Apps blind" können
nicht mehr falsch gemacht werden, weil die App keine fremde Taste mehr anfasst.
**Verworfen:** `WH_KEYBOARD_LL` (Aufwand, Störungsanfälligkeit, Blindheit, unter
Wayland ohnehin unmöglich — drei Bedienmodelle); `evdev`/`/dev/input` (sähe alle
Tastatureingaben eines Mandantenwerkzeugs); Hook nur unter Windows (zwei
Bedienmodelle); Push-to-talk (kein Key-up bei `RegisterHotKey`).

## 2026-11-20 — E4 `same_window` nur mit der Fähigkeit

**Entscheidung:** `insert_after_dictation = same_window` gilt nur, wo die
Plattform das Vordergrundfenster kennt: Windows, macOS (`frontmostApplication`),
Linux/X11. Unter Wayland gibt es keine API dafür ⇒ `never`.
**Grund:** Die Fähigkeitsleiter (E2) entscheidet über den Modus, nicht der
Wunsch. Ein falsch geratenes Fenster ist teurer als ein nicht gesendetes Strg+V:
der Text bleibt in der Zwischenablage.
**Verworfen:** Fensteridentität über Compositor-IPC raten (KWin-Skript,
GNOME-Erweiterung); `always` als Ersatz (schickt Text ins falsche Fenster).

## 2026-11-20 — E5 Zwischenablage-Ausschluss ist dreistufig

**Entscheidung:** Der Ausschluss aus History und Cloud ist je Plattform
verschieden stark — und die Doku sagt das, statt „überall gelöst" zu behaupten:

| | Formate | Ausschluss aus History/Cloud |
|---|---|---|
| Windows | `CF_HTML` + `CF_UNICODETEXT` | harte API: `CanIncludeInClipboardHistory=0`, `CanUploadToCloudClipboard=0`, `ExcludeClipboardContentFromMonitorProcessing` |
| macOS | `public.html` + `public.utf8-plain-text` | De-facto-Konvention `org.nspasteboard.ConcealedType` (+ `TransientType`), von Maccy/PastePal u. a. honoriert, vom OS nicht erzwungen |
| Linux | `text/html` + `text/plain;charset=utf-8` | praktisch nichts; KDE kennt `x-kde-passwordManagerHint`, GNOME hat keine eingebaute History |

Die Zusage in `AGENTS.md` („immer die Ausschlussformate setzen") gilt nur für
Windows unverkürzt und ist dort plattformabhängig formuliert. Tauris
`writeSecret()` markiert plattformgerecht, kann aber kein HTML mit Markern — die
eigenen Plattformmodule bleiben.
**Grund:** Der Marker ist der einzige Hebel gegen `Win+V` und die Cloud-History,
und unter Windows ist er erzwungen. Auf macOS und Linux ist weniger erreichbar —
das zu sagen ist billiger als ein Versprechen, das die Plattform nicht einlöst.
**Verworfen:** Ausschluss als „überall gelöst" behaupten; Marker weglassen
(verschenkt die macOS-Konvention); nur Text ausliefern (verschenkt die Struktur).

## 2026-11-20 — E6 Linux ist die erste Schale, Windows bleibt das Produkt

**Entscheidung:** Gebaut wird auf Linux, deshalb zuerst die Linux-Schale;
Reihenfolge der Auslieferung bleibt offen (E1).
**Grund:** Markt ≠ erste Implementierung. Die Linux-Schale meldet die wenigsten
Fähigkeiten (unter Wayland weder Vordergrundfenster noch Einfügen), jede spätere
kann nur hinzufügen. Die schwächste zuerst zu bauen ist ein Vorteil: der
Entwickler lebt täglich mit dem degradierten Pfad, statt ihn am Ende zu
entdecken.
**Verworfen:** Linux als Primärziel (kippt Zielgruppe und das
Word-Merge-Formatting-Kriterium von #1); Windows zuerst (die VM ist der Blocker,
nicht der Kern).

## 2026-11-20 — E7 Verpackung Linux: AppImage primär, `.deb` zweitens

**Entscheidung:** AppImage für Entwicklung und Ausprobieren, `.deb` als der
„richtige" Weg (dort sind `.desktop`, Autostart und Icon nativ dabei).

Das AppImage kollidiert mit dem Hotkey-Weg: **das GlobalShortcuts-Portal
verlangt unter GNOME eine reverse-DNS-App-ID, die durch eine installierte
`.desktop`-Datei gedeckt ist.** Eine AppImage aus `~/Downloads` hat keine, und
GNOME lehnt dann stumm jede Bindung ab. Drei Pflichten der Schale:

1. beim ersten Start `~/.local/share/applications/<app-id>.desktop` + Icon
   schreiben (ohne root),
2. `org.freedesktop.host.portal.Registry.Register(app_id)` vor `CreateSession`
   aufrufen,
3. Autostart bei jedem Start neu schreiben, mit dem aktuellen
   `$APPIMAGE`-Pfad (sonst bricht der Eintrag beim Umbenennen/Verschieben).

Dazu: `libfuse2` fehlt auf Ubuntu ≥ 22.04 (auf claw bestätigt) ⇒
`--appimage-extract-and-run` oder einmalig nachinstallieren; Tray-Bibliothek vom
Host, nicht gebündelt (gebündeltes AppIndicator bricht das Theming); die Datei
wird groß, weil WebKit mitgeht.
**Grund:** Ein AppImage ist ohne Installation ausprobierbar, ein `.deb` ist der
Weg, den ein Linux-Nutzer erwartet. Beides zu haben kostet wenig; eines zu
wählen kostet entweder Ausprobieren oder Installation.
**Verworfen:** nur `.deb` (kein Ausprobieren ohne Installation); nur AppImage
(Portal-Gate, Autostart-Fußangel); Flatpak (eigener Verteiler, eigener
Testaufwand).

## 2026-11-20 — E8 App-ID: PascalCase, endgültige Entscheidung zum ersten öffentlichen Build

**Entscheidung:** Vorläufig `at.priv.graf.Diktatwerkstatt` — reverse-DNS der
Domain `graf.priv.at`, letztes Segment in PascalCase; die `.desktop`-Datei heißt
zeichengenau `at.priv.graf.Diktatwerkstatt.desktop`. Endgültig zusammen mit der
Namens-/Domainfrage (Markenrecherche steht offen); im Rennen ist dann auch
`at.diktatwerkstatt.Diktatwerkstatt`. Die Tür schließt sich mit dem ersten
öffentlichen Build, weil die Portal-Bindungen an der ID hängen und einen Neustart
überleben. Spike-IDs sind Wegwerf.
**Grund:** GNOME akzeptiert Hotkey-Bindungen nur für eine ID, die eine
installierte `.desktop`-Datei deckt (E7); reverse-DNS ist die Konvention, und das
letzte Segment in PascalCase hält Dateiname und ID deckungsgleich.
**Verworfen:** durchgehend klein (zulässig, aber unüblich und fehleranfällig beim
Dateinamen); `at.graf.…` (falsches reverse-DNS); `io.github.georgernstgraf.…`
(sperrt die Zukunft an einen Hosting-Namen).

## 2026-11-20 — E9 Der Name bleibt `diktatwerkstatt`

**Entscheidung:** Der Name bleibt `diktatwerkstatt`, bestätigt gegen die Erwägung
„Diktierwerkstatt". Technischer Name bleibt `diktatwerkstatt` (crate,
Datenverzeichnis, DB-Datei).
**Grund:** „Diktier-" ist im Deutschen ein überfülltes Wortfeld (`Diktiergerät`,
`Diktiersoftware`, `Diktierfunktion`) — schlechter suchbar und markenrechtlich
enger. Ein bestehender Name, der trägt, ist billiger als ein Wechsel vor dem
ersten Build.
**Verworfen:** „Diktierwerkstatt" (überfülltes Wortfeld); ein neuer Name vor der
Markenrecherche (zwei Umbenennungen statt einer).

## 2026-11-20 — E10 `Delivery` trennt Fragment und Umschlag

**Entscheidung:** `deliver(markdown) -> Delivery { html, text }`, wobei `html`
ein **semantisches Fragment** ist (Absatz, Überschrift, ungeordnete Liste, fett).
Der `CF_HTML`-Umschlag (Kopfzeile, Byte-Offsets, Zehnstellig-Nullfüllung) ist
eine Funktion des **Windows-Schreibers**.
**Grund:** Zwei von drei Zielen wollen das nackte Fragment (`text/html`,
`public.html`); ein Umschlag im Kern wäre Plattformwissen in der Domäne und
schließt Linux und macOS aus. Der Kern bleibt mit `cargo test` prüfbar, der
Umschlag ist eine Plattformfunktion mit eigenen Tests.
**Verworfen:** alles in einem `cf_html()` (schließt Linux und macOS aus);
Umschlag in der Domäne (Plattformwissen im Kern).

> Die zehn Einträge E1–E10 halten die **Beschlusslage des Eigentümers vom
> 2026-11-16** fest; sie stehen hier in der Reihenfolge des Beschlusses und sind
> am 2026-11-20 in die Dokumentation eingearbeitet (#11).

## 2026-11-19 — Kopier-Knopf als zweiter Weg zur Zwischenablage

**Entscheidung:** Die Workbench hat einen Knopf **„In die Zwischenablage"**. Er
liefert den **sichtbaren** Reiter jederzeit — ohne vorherige Bearbeitung, ohne
Verlassen des Fensters und ohne Einfügen, in beiden Formaten und mit denselben
Ausschlussformaten wie jede andere Lieferung. Er ist ein **zweiter** Weg, nicht
der einzige: die Lieferung beim Abschluss, bei **Prompt anwenden** und beim
Verlassen einer bearbeiteten Ansicht bleibt unverändert. Auf Android gilt
dasselbe im Editor (`polished-recognition`, #166).
**Grund:** Drei Fälle deckt die Automatik nicht ab: die Zwischenablage wurde
inzwischen von einer anderen Anwendung überschrieben; ein älteres Diktat aus dem
FIFO wird nur gelesen und darum nie geliefert; derselbe Text wird nach dem
Verlassen noch einmal gebraucht. Im Markor-Ablauf ist „alles markieren,
kopieren" genau der Schritt, der hier ein Klick wird. Der Knopf als **einziger**
Weg bleibt verworfen — siehe den Eintrag darunter.
**Verworfen:** der Knopf als einziger Weg (ein Pflichtklick, wo ein Ereignis
genügt); der Knopf liefert die „zuletzt bearbeitete" statt der sichtbaren
Ansicht (bei einem nur gelesenen Diktat gäbe es nichts Bearbeitetes); der
History-Ausschluss nur bei der Automatik (die Quelle des Textes ist dieselbe —
ein Mandantendiktat gehört nicht in `Win+V`); ein zweiter Knopf „Kopieren ohne
Ausschluss" (zwei Knöpfe für einen Vorgang, und der Unterschied ist im
Arbeitsalltag nicht erklärbar).

## 2026-11-19 — Bearbeiten ist erlaubt, und wer bearbeitet, liefert

**Entscheidung:** Beide Reiter der Workbench sind editierbar — das rohe **und**
das polierte Transkript. Wer eine Ansicht bearbeitet hat und sie wieder
verlässt, schreibt ihren Text **bedingungslos** in die Zwischenablage: kein
Dialog, keine Rückfrage, keine Rücksicht auf den vorhandenen Inhalt. Maßgeblich
ist die **zuletzt bearbeitete** Ansicht. Es folgt **kein** Einfügen; geliefert
wird die Zwischenablage. Wurde nichts bearbeitet, wird auch nichts geliefert.
Ein Rohtext-Edit verwirft das Polierte; wer danach erneut poliert, bekommt das
Ergebnis in die Zwischenablage, sobald es vom LLM-Dienst kommt.
**Grund:** Die Praxis mit Markor zeigt den Bedarf: die Zwischenablage ist die
Schnittstelle, aber „alles markieren, kopieren" ist ein manuelles Ritual — und
genau der Schritt fehlt dort, den nur die App tun kann. Die Workbench ist damit
kein Lesefenster mehr, sondern ein **Text-Supplier**: was im Fenster steht, ist
das, was eingefügt wird. Das ist zugleich die einzige Form, in der Invariante 8
ohne Wettlauf hält — geschrieben wird nie ein Text, der nicht auf dem Schirm
steht.
**Verworfen:** Lieferung nur auf Knopfdruck (ein Klick mehr als nötig, und
genau der Klick, den Markor verlangt); Rückfrage vor dem Überschreiben (die
Zwischenablage ist ein Zwischenspeicher, kein Archiv); den alten Inhalt
aufheben und nach dem Einfügen zurücklegen (Delayed Render und Read-Receipt,
siehe 2026-11-16); Bearbeiten nur im polierten Reiter (der Rohtext ist die
Reparaturfläche, wenn die Politur den Fehler nicht kennt); den polierten Reiter
einfrieren (macht jede Korrektur nach dem Polieren unmöglich).
**Offen:** Auf dem Desktop gibt es keinen „Edit-Modus", dessen Ende das Signal
gibt. Der Auslöser — Fokusverlust, Fensterschließen oder ein Knopf — ist nicht
entschieden (siehe `HANDOFF.md`). Auf Android ist er eindeutig: die geschlossene
Tastatur. Dieselbe Lieferregel gilt dort in `polished-recognition` (#166), mit
getrenntem Code (Entscheidung 2026-11-15).

## 2026-11-16 — Einfügen ist eine Einstellung, die Zwischenablage bleibt das Ergebnis

**Entscheidung:** Nach dem Abschluss eines Diktats darf die App **einmal**
Strg+V senden. Ob sie es tut, steht in `config.json` als
`insert_after_dictation` mit `never`, `same_window` (Default) und `always`.
Bei `same_window` wird nur eingefügt, wenn das Vordergrundfenster beim
Abschluss dasselbe ist wie beim Einfügen; es wird nie ein Fenster reaktiviert.
**Prompt anwenden** fügt nie ein. Die Zwischenablage wird in jedem Fall
geschrieben und behält den Text.
**Grund:** Handys Bedienung spart genau einen Tastendruck; der Preis ist die
Verantwortung für den Fokus. Ein 90-Minuten-Protokoll darf nicht in das Fenster
fallen, das zufällig vorn ist. Weil die Zwischenablage das Ergebnis ist und den
Text behält, ist ein gescheitertes Einfügen folgenlos — der Nutzer drückt
Strg+V. Deshalb braucht es weder UIPI-Prüfung noch Meldung noch ein
Zurückholen des Zielfensters.
**Verworfen:** nie einfügen (kostet einen Tastendruck ohne Gegenwert; die
Atomarität der Zwischenablage verlangt es nicht); immer einfügen (falsches
Fenster bei langen Diktaten); UIPI-Erkennung mit Meldung (löst ein Problem, das
nicht existiert, solange der Text in der Zwischenablage liegt); Zielfenster
reaktivieren (verschiebt den Fokus des Nutzers); Delayed Render,
Format-Snapshot und Read-Receipt (setzen voraus, dass die alte Zwischenablage
zurückgelegt wird — tun wir nicht).
**Amendiert 2026-11-20:** Die Einstellung gilt auf allen drei Zielen. Die
**Plattform-Defaults** folgen der Fähigkeitsleiter: Windows und Linux senden
Strg+V, macOS Cmd+V; kennt die Plattform das Vordergrundfenster nicht (Wayland
ohne `foreground_window`), wird `same_window` zu `never` (E2, E4). Neu ist die
Regel **„nie ins eigene Fenster"**: ist die Workbench das Vordergrundfenster,
unterbleibt das Einfügen auch bei `always` (E3).

## 2026-11-16 — Korrigieren heißt Hören reparieren

**Entscheidung:** Das polierende Modell repariert die Transkription, nicht die
Aussage. Dialekt, Umgangssprache und Satzbau des Sprechers bleiben; korrigiert
werden Erkennungsfehler. Namen, Zahlen, Daten und Beträge werden repariert, wo
die Lesart sicher ist, und sonst gelassen — nie erfunden, nie stillschweigend
„verbessert". Sichtbare Marker wie `[? Grund]` gibt es in v0.1 **nicht**.
**Grund:** Übernommen aus dem `protokoll`-Skill der `opencode-helpers`
(„Repair the hearing, not the speech", „Flag, don't fix" für die tragenden
Fakten). Ein falscher Nachname ist ein Wort von hundert — 1 % WER und trotzdem
ein Kanzleifehler; der WER misst hier das Falsche. Marker im Text scheiden aus,
weil das polierte Transkript in einen Brief wandert und `[? unklar]` dort eine
Peinlichkeit ist.
**Verworfen:** das Modell als Lektor, der glättet und umformuliert (verändert
Aussagen, die niemand ändern wollte); sichtbare Flag-Marker im polierten
Transkript (im Brief unbrauchbar); Flag-Marker als **spätere** Prompt-Option für
Protokolle (bleibt möglich, gehört aber nicht in v0.1).

## 2026-11-16 — Lizenz: dual MIT OR Apache-2.0

**Entscheidung:** Das Projekt steht unter `MIT OR Apache-2.0`. Zwei
Lizenzdateien (`LICENSE-MIT`, `LICENSE-APACHE`), in `Cargo.toml` später
`license = "MIT OR Apache-2.0"`.
**Grund:** Das ist die Konvention des Rust-Ökosystems und löst den Konflikt, der
den Eintrag vom 2026-11-15 offen hielt: MIT für maximale Anschlussfähigkeit,
Apache-2.0 für den Patentabsatz, den Firmen erwarten — der Empfänger wählt.
**Verworfen:** nur MIT (kein Patentabsatz; unattraktiv für eine mitentwickelnde
Firma), nur Apache-2.0 (NOTICE-Pflicht, eine Hürde mehr für private
Beitragende), AGPL (schließt genau die Wiederverwendung aus, die gewünscht ist).

## 2026-11-16 — FIFO statt Ring

**Entscheidung:** Die letzten N Diktate bilden eine Warteschlange. N steht in
`config.json`. Kommt eines hinzu, fällt das älteste heraus, samt
Arbeitsverzeichnis. Es gibt kein `pinned`.
**Grund:** „Ring" behauptete eine kreisförmige Struktur, die es nie gab, und
`pinned` machte aus einer Warteschlange eine Regel mit Ausnahme: „ältestes
*nicht angeheftetes*". Diese Ausnahme hat ein Loch — sind alle N angeheftet,
wächst der Bestand unbegrenzt, und die Invariante fällt lautlos.
**Verworfen:** Ring mit `pinned` (Ausnahme ohne Obergrenze), feste Größe ohne
Einstellung (der Bedarf schwankt zwischen Briefserie und Protokollwoche).

## 2026-11-16 — Kein Zustandsfeld: das Arbeitsverzeichnis ist der Zustand

**Entscheidung:** Ein Diktat hat kein `status`, kein `closed_at`, keine
Unveränderlichkeit. Ob es offen ist, sagt allein `audio_dir`: existiert das
Verzeichnis, ist die Transkription nicht abgeschlossen. Enter beendet die
Aufnahme, nicht das Diktat — angehängt werden darf jederzeit, auch Tage später.
**Grund:** „Geschlossen" bedeutete „eingefroren" — und das wurde verneint, weil
`aitranscribe` das Anhängen später erlaubt und weil ein Vormittag mehrere
Diktate hat. Damit hätte ein Statusfeld zwei Namen für einen Zustand geführt,
der kein Verhalten ändert. Ein Feld, das man setzen kann, ohne dass sich etwas
ändert, ist genau das, was der Eintrag „Politur wird verworfen, nicht markiert"
ausschließt.
**Verworfen:** `status` + `closed_at` (Zustand ohne Verhalten);
Unveränderlichkeit geschlossener Diktate (verhindert das Nachtragen eines
Satzes); Schließen per Zeitablauf (ein Timer, der schließt, während man
nachdenkt); mehrere gleichzeitig offene Diktate (braucht „wieder öffnen").

## 2026-11-16 — Rohes Transkript als ein Text, keine Blöcke

**Entscheidung:** `diktat.raw_text` ist ein einziger Text. Es gibt keine Tabelle
`chunk`, kein `seq`, keine Blockgrenzen in der Datenbank.
**Grund:** Der Rohtext wurde bisher als Verkettung von Blöcken definiert und
„nicht redundant" gespeichert — was bedeutete, dass es die Spalte gar nicht
gab. Mit einem Text ist Invariante 4 trivial wahr statt dauernd zu beweisen.
`seq` war ein zweiter Sortierschlüssel neben dem ohnehin monotonen `id`, und
„lückenlos 1-basiert" kostete beim Löschen eines Blocks ein Umnummerieren.
**Verworfen:** Tabelle `chunk` mit `seq` (zweite Wahrheit, Umnummerierung,
Blockgrenzen als Anzeige, die niemand angefordert hat); `raw_text` zusätzlich
zu `chunk` (Redundanz, die auseinanderlaufen kann).

## 2026-11-16 — Text vor Audio

**Entscheidung:** Ein transkribiertes Fragment steht auf der Platte, **bevor**
seine Audiodatei gelöscht wird. Auf Diktatebene gilt dasselbe: `raw_text` ist
committet, bevor das Arbeitsverzeichnis verschwindet.
**Grund:** Zwei Dateisystem-Operationen sind nie atomar. Löscht man zuerst das
Audio und stürzt dazwischen ab, sind Audio und Text weg — im Protokoll fehlt
still ein Satz, und niemand merkt es. In der anderen Reihenfolge bleiben beide
liegen, und die Wiederaufnahme räumt die überflüssige `.wav` weg. Geschrieben
wird über `.tmp` + `rename`, damit eine halb geschriebene `.txt` nicht als
erledigt gilt.
**Verworfen:** „Audio zuerst" (stiller Datenverlust); Zustand in der Datenbank
statt im Verzeichnis (hätte bei jedem Absturz eine Reparatur gebraucht);
direktes Schreiben ohne `rename` (abgeschnittene Dateien sehen fertig aus).

## 2026-11-16 — Zwei Prompt-Auswahlen aus einer Tabelle

**Entscheidung:** Die Tabelle `prompt` hält Zeilen mit `is_system`. Der Nutzer
wählt genau einen System-Prompt (Rolle, Ausgabeformat) und genau einen
User-Prompt (die Aufgabe); beide gehen in den Aufruf. Keine Verknüpfung, keine
Fremdschlüsselspalte. Die Zielsprachen-Klausel steht im `user`-Text, nicht in
einem eigenen Feld.
**Grund:** Die Chat-API will genau eine System- und eine User-Nachricht. Zwei
unabhängige Auswahlen bilden das ohne Zwischentabelle ab. `zielsprache` und
`ton` waren von Anfang an „optional, der Alias genügt" — eine Klausel im Text
leistet dasselbe ohne Spalte.
**Verworfen:** `prompt` mit `system` und `user` als zwei Spalten einer Zeile
(hätte den Nutzer gezwungen, Paare zu pflegen, die er nicht will);
Fremdschlüssel User-Prompt → System-Prompt (Verknüpfung ohne Anwendungsfall);
`zielsprache`/`ton`/`default_model`/`archiviert` (Spekulation bzw. Zustand, den
bei acht Prompts niemand braucht).

## 2026-11-16 — Politur pro Diktat, über zwei Hotkeys

**Entscheidung:** Es gibt „diktieren roh" und „diktieren poliert". Der zweite
wendet die aktuelle Auswahl automatisch an. **Prompt anwenden** bleibt für die
Wiederholung mit einer anderen Auswahl.
**Grund:** In `polished` ist das Keyboard immer im Blick; dort genügt eine
sichtbare Auswahl. Auf dem Desktop ist das Fenster im Normalfall zu — ein
Schalter, den man erst öffnen muss, ist kein Schalter. Zwei Hotkeys sind das
Gegenstück zur Send-Taste: eine Handlung, kein Zustand, an den man sich erinnern
muss.
**Verworfen:** Politur-Schalter im Fenster (nicht erreichbar, wenn das Fenster
zu ist); globaler Schalter (man muss vor dem Sprechen an etwas denken); dritter
Hotkey zum Umschalten (zwei Schalter für dieselbe Sache).
**Amendiert 2026-11-20:** Beide Diktier-Hotkeys sind **Umschalter** ohne Key-up —
erster Druck startet, zweiter beendet; während einer Aufnahme beenden beide
(E3). Pause/Fortsetzen ist der dritte globale Hotkey, nicht ein zweiter
Diktier-Hotkey.

## 2026-11-16 — Ein Aufruf, kein Abschneiden

**Entscheidung:** Das ganze rohe Transkript geht in **einem** Aufruf an das
LLM. Passt es nicht in das Modell (Kontext oder Ausgabelänge), erscheint ein
Fehler; es wird nie gekürzt und nie in Abschnitte zerlegt.
**Grund:** Ein 90-Minuten-Protokoll hat ~20.000 Token. Modelle wie Luna oder
Flash tragen das; ein kleines Modell trägt es nicht — und dann ist eine
Fehlermeldung billiger und ehrlicher als eine abschnittsweise Politur, deren
Nähte man nachträglich wieder zusammenkleben müsste. Das Nahtproblem der
Audioseite ein zweites Mal auf der Textseite zu lösen, wäre der teuerste Teil
des Projekts.
**Verworfen:** Chunked Politur mit unsichtbaren Nähten (neues Nahtproblem,
Overlap, Dedupe, Stilbruch zwischen Abschnitten); stilles Kürzen (der Nutzer
merkt es erst am fehlenden Absatz).

## 2026-11-16 — Zwischenablage genau einmal, atomar

**Entscheidung:** Die Zwischenablage wird pro Diktat **einmal** geschrieben, im
selben Zug, in dem der Text im Fenster erscheint. Bei „diktieren roh" ist das der
Rohtext, bei „diktieren poliert" das polierte Transkript — vorher bleibt die
Zwischenablage unberührt. Scheitert die Politur, geht der Rohtext als Rückfall
hinein.
**Grund:** Würde der Rohtext sofort in die Zwischenablage gehen und die Politur
ihn später überschreiben, gäbe es einen Wettlauf: wer früh einfügt, hat roh,
wer spät einfügt, poliert — und im Dokument steht etwas anderes als in der
Zwischenablage. Atomar gibt es diesen Zustand nicht. Der Rückfall im Fehlerfall
ist die einzige Stelle, an der die App sonst fertigen Text vernichten würde.
**Verworfen:** Rohtext sofort, Politur überschreibt später (Wettlauf, zwei
Wahrheiten); nur im Fenster, Zwischenablage auf Knopfdruck (der Nutzer hat den
Text, aber nicht dort, wo er ihn braucht); Rohtext im Fehlerfall verwerfen
(vernichtet bezahlte Arbeit).
**Erweitert 2026-11-19:** Die Zwischenablage wird zusätzlich bei jeder Lieferung
aus einer bearbeiteten Ansicht geschrieben („Bearbeiten ist erlaubt, und wer
bearbeitet, liefert"); die Atomarität gilt weiter für den Abschluss, und es wird
weiterhin nie ein Text geschrieben, der nicht im Fenster steht.

## 2026-11-16 — Semantisches HTML auf der Zwischenablage, constrained Markdown als Format

**Entscheidung:** Auf die Zwischenablage gehen `CF_HTML` (semantische Tags,
kein CSS, keine Klassen) und `CF_UNICODETEXT`. Das polierte Transkript darf nur
Absatz, Überschrift, ungeordnete Liste und fett enthalten; geordnete Listen
werden als Text mit ihren Ziffern ausgegeben. Vor dem Schreiben wird validiert.
**Grund:** Reiner Text transportiert keine Struktur, und Words nativer
„Paste from Markdown" ist undokumentiert und je nach Variante nicht vorhanden
(Mac: gar nicht, Windows: Insider). HTML versteht Word seit jeher und bildet
`h1`–`h6` auf seine Formatvorlagen ab — damit trägt der Prompt die Struktur und
die Vorlage das Aussehen, ohne Word-Kopplung. Die Einschränkung auf vier
Konstrukte, weil Markdown gegen juristischen Text nicht neutral ist:
`1)` wird zur Liste (CommonMark erlaubt `.` und `)`), `5*6*78` wird kursiv,
`\§` bleibt als Backslash stehen, und Words alte AutoFormat-Option macht aus
`*fett*` **fett** statt kursiv.
**Verworfen:** Markdown roh in die Zwischenablage (Word nimmt das HTML und sieht
es nie; textonly-Ziele zeigen Sternchen); volles CommonMark (Kollisionen mit
Rubrum-Nummerierung); Word-COM (nur installiertes Word, nicht Google Docs oder
Outlook Web); docx-Datei (Dateidialog im Alltag).
**Offenes Risiko:** Seit Word-Build 2405 ist „Merge Formatting" der Default für
Einfügen aus anderen Programmen; Microsofts Beschreibung nennt Fett und Listen,
aber **nicht** Überschriften. Ob Überschriften verflachen, ist nur auf echtem
Windows messbar und gehört zu den Abnahmekriterien von #1.
**Amendiert 2026-11-20:** `CF_HTML` ist nicht mehr das Format „der
Zwischenablage", sondern der **Umschlag des Windows-Schreibers**; macOS und
Linux nehmen das nackte semantische Fragment. Die Formatregeln — vier
Konstrukte, Validierung, geordnete Listen als Text — bleiben und gelten für das
Fragment selbst (E10, E5).

## 2026-11-16 — Kein Audio-Import; Diktat aus der Zwischenablage

**Entscheidung:** Die App nimmt Audio nur über das Mikrofon auf. Ein langes
Transkript (Besprechungsprotokoll) kommt über ein neues Diktat aus der
Zwischenablage herein. Woher der Text stammt, wird nicht gespeichert.
**Grund:** Ein Datei-Eingang ist ein zweiter Ablauf ohne Hotkey, ohne
Tray-Zustand und ohne Fragmentierung — und er zieht die Frage nach der
Abschnittspolitur sofort in v0.1. `aitranscribe` deckt den Fall heute ab; ein
Kopieren ist billiger als ein zweiter Eingang.
**Verworfen:** Audio-Import in v0.1 (verdoppelt den Eingang); Spalte `quelle`
(der Nutzer braucht sie nicht — sie wurde eingeführt und wieder gestrichen).

## 2026-11-16 — Keine Herkunftsangabe am polierten Transkript

**Entscheidung:** Es wird nicht gespeichert, welcher Prompt und welches Modell
das polierte Transkript erzeugt haben — weder als Fremdschlüssel noch als
Schnappschuss.
**Grund:** Die Anzeige „erzeugt von Prompt X, Modell Y" war eine Anforderung
ohne Nutzer. Ein Fremdschlüssel hätte zusätzlich gelogen, sobald ein Prompt
umbenannt wird: die Herkunft hätte sich rückwirkend geändert.
**Verworfen:** `polished_prompt_id` (Lüge nach Umbenennung, Fremdschlüssel auf
löschbare Zeilen); Alias-Schnappschuss (wahr, aber ohne Abnehmer);
`polished_model` (dito); `polished_at` (eine Spalte, die niemand liest).

## 2026-11-16 — Einstellungen in `config.json`, nicht in der Datenbank

**Entscheidung:** FIFO-Größe, Hotkeys, Endpunkte, Modellnamen und die
Prompt-Auswahl stehen in `config.json` neben der Datenbank.
**Grund:** Die Datenbank enthält Mandantendaten. „Daten löschen" darf die
API-Konfiguration nicht mitnehmen, und eine Einstellungsdatei soll man von Hand
lesen und sichern können. Eine `setting`-Tabelle wäre außerdem eine dritte
Tabelle für Daten, die nicht in die Datenbank gehören.
**Verworfen:** `setting`-Tabelle (vermischt Konfiguration mit Mandantendaten);
Einstellungen im Windows-Register (nicht portierbar, nicht lesbar).

## 2026-11-15 — Name: diktatwerkstatt

**Entscheidung:** technischer Name `diktatwerkstatt`.
**Grund:** frei auf GitHub, crates.io und PyPI; ein Token; beschreibend; keine
Kollision mit dem übersättigten `whisper-*`-Namensfeld (und „Whisper" ist
zugleich Modellmarke von OpenAI).
**Verworfen:** `zumdiktat` (GitHub-User belegt, und „zum Diktat" ist im Deutschen
eine Allerweltswendung — nicht suchbar), `polished-dictation` (generisch,
englisch), `briefsteller` (bleibt als Markenidee in Reserve).

## 2026-11-15 — Rust-Kern + Tauri/WebView2

**Entscheidung:** Domänen- und Plattformlogik in Rust; UI als Tauri-App mit
WebView2; der Fast Path läuft vollständig ohne Webview.
**Grund:** Kompiliert, kleiner Footprint, Start unter 200 ms, und der einzige
realistische Kooperationspartner im Feld (Handy, 38 k Sterne) fährt genau diese
Kombination. TMOG, der neue Task Manager seines Originalautors, zeigt dasselbe
Muster: ein geteilter kompilierter Kern plus dünne Plattformschalen.
**Verworfen:** Kotlin/JVM (Bytecode + JIT — nicht „nativ" im hier gemeinten Sinn);
Python (interpretiert, dazu PortAudio-/PyInstaller-Reibung); C# (nur sinnvoll,
wenn Word-Automation zentral wird — dann später ein eigener Helfer).
**Amendiert 2026-11-20:** „Tauri/WebView2" wird zu **Tauri mit drei Webviews** —
WebView2 ist eine von drei, daneben WebKitGTK (Linux) und WKWebView (macOS). Als
Windows-Sonderfall bleibt der Satz richtig; als Stackname ist er es nicht mehr,
und der Kern ist seither plattformfrei (E1, E2).

## 2026-11-15 — Poliertes Transkript wird verworfen, nicht markiert

**Entscheidung:** Ändert sich das rohe Transkript, wird `polished_text` auf
`NULL` gesetzt. Es folgt keine automatische Neupolitur.
**Grund:** Ein als „veraltet" markiertes Ergebnis bleibt sichtbar und wird
irgendwann kopiert. Bei Sekretariatsarbeit ist ein Zustand, in dem die App etwas
Falsches als richtig anzeigt, schlimmer als ein verlorenes Ergebnis. Die
Neupolitur bleibt eine bewusste Handlung, weil sie Geld kostet — bei einem
90-Minuten-Protokoll viel Geld.
**Verworfen:** `raw_fingerprint` plus Veraltet-Kennzeichnung (mehr Zustand, mehr
Fehlerquellen); automatische Neupolitur nach jedem Append (ein API-Aufruf ohne
Zutun, ausgelöst durch eine Tipp-Korrektur).

## 2026-11-15 — Kein Run-Verlauf

**Entscheidung:** keine Tabelle `run`, keine Prompt-Versionierung, kein
Vergleich zweier Ergebnisse.
**Grund:** Der Nutzer braucht 3–5 Diktate und jeweils das letzte Ergebnis. Der
Verlauf ist Ballast, und bei einem FIFO von 7 Einträgen wäre er ohnehin kurz.
**Verworfen:** `runs`-Tabelle mit Snapshots (war der erste Entwurf; verworfen,
weil sie eine Funktion ohne Nutzerbedarf trägt).

## 2026-11-15 — Prompt anwenden fügt nicht ein

**Entscheidung:** **Prompt anwenden** schreibt das polierte Transkript in die
Workbench und in die Zwischenablage. Es tippt nichts in die Zielanwendung.
**Grund:** Ein zweiter Lauf bei bereits eingefügtem Rohtext erzeugt sonst
doppelten Text in einem fremden Dokument. Die Übergabe an die Zielanwendung
bleibt eine bewusste Handlung.
**Verworfen:** automatisches Einfügen nach dem Polieren.

## 2026-11-15 — Tray als Zustandsanzeige

**Entscheidung:** Zustandsanzeige im Tray (`IDLE`, `RECORDING`, `PAUSED`,
`TRANSCRIBING`, `POLISHING`, `READY`, `POLISHED`, `FEHLER`), mit Blinken während
der Aufnahme und **einem** Fehlerzustand. Zusätzlich eine kleine Overlay-Pille,
weil Windows 11 neue Tray-Symbole standardmäßig im Überlauf versteckt.
**Grund:** Der Nutzer soll diktieren können, ohne die App anzusehen; das Tray
allein ist dafür nicht verlässlich sichtbar genug. Zwei verschiedene Rottöne für
STT- und LLM-Fehler unterscheidet niemand im Augenwinkel — die Ursache steht im
Fenster.
**Verworfen:** Tray als Diagnosewerkzeug (zeigt an, was die App tut, statt was
der Nutzer tun soll); Fortschrittsanzeige im Tray (dafür ist das Fenster da).

## 2026-11-15 — Keine geteilte Codebasis mit polished-recognition

**Entscheidung:** Der Rust-Kern ist eine Neuimplementierung; geteilt werden
Spezifikation, Tests als Prüfstein, das Dateiformat — und seit 2026-11-16 auch
der Prompt-Text (ergänzt).
**Grund:** `polished-recognition` bleibt die Android-Tastatur in Kotlin. Ein
gemeinsamer Kern über Android und Desktop hieße JNI oder Rust in der App — dafür
gibt es keinen Bedarf.

Was konkret geborgt wird und was nicht:

| Was | Woher | Wie |
|---|---|---|
| Vokabular (Fragment, Naht, Q, Q-Gate, Pre-roll, Echo-Trim) | `polished-recognition` | übernommen, hier weitergeführt |
| Algorithmen (Fragmentierung, Naht, Stillemaß) | `polished-recognition` (Kotlin), `aitranscribe` (Python, 33 Tests) | nach Rust **portiert**, Tests als Prüfstein (#5) |
| Prompt-Text (`prompts.json`) | `polished-recognition` | wörtlich als Standard-Prompt |
| Bedienmuster (Send-Taste, Prompt-Auswahl, mitlaufende Fragmente) | `polished-recognition` (IME) | auf den Desktop übertragen: zwei Hotkeys, zwei Auswahlfelder |
| Datenmodell „ein Record in SQLite, später appendable" | `aitranscribe` | übernommen, ohne Blöcke |
| Evidenzformat `session.json` | `polished-recognition` | bleibt lesbar für den dortigen Kotlin-`analyzer` |
| Code | — | keiner |

**Verworfen:** auf `polished` aufbauen (Android-Tastatur ohne Tray, Hook, Fenster
und FIFO); einen gemeinsamen Rust-Kern in die Android-App ziehen (JNI ohne
Nutzen).
