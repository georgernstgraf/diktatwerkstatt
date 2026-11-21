---
type: Domain
title: Domänenmodell
description: Tabellen, Invarianten, Abläufe und Arbeitsverzeichnis der Diktatwerkstatt.
tags: [domain, datenmodell]
generated: { by: opencode/general, at: 2026-11-21T08:40:06Z }
status: stable
---

# Domänenmodell

Zwei Tabellen. Eine SQLite-Datei. Dazu eine Konfigurationsdatei und ein
Arbeitsverzeichnis pro laufendem Diktat. Dasselbe Modell gilt auf allen drei
Zielen — Windows, Linux, macOS; verschieden ist nur die **Plattformschale**.

## Speicherorte

| Was | Wo |
|---|---|
| Datenbank | `%LOCALAPPDATA%\diktatwerkstatt\diktatwerkstatt.db` (Windows), `~/.local/share/diktatwerkstatt/diktatwerkstatt.db` (Linux), `~/Library/Application Support/diktatwerkstatt/diktatwerkstatt.db` (macOS) |
| Konfiguration | `config.json` **neben** der Datenbank, nicht darin |
| Arbeitsverzeichnisse | `<Datenverzeichnis>/audio/<diktat-id>/` — temporär, verschwindet nach dem Abschluss |
| API-Schlüssel | Windows Credential Manager bzw. Secret Service (Linux) bzw. Schlüsselbund (macOS) — **nie** in der Datenbank oder in `config.json` |

Die Trennung von Datenbank und Konfiguration ist Absicht: „Daten löschen" darf
die API-Konfiguration nicht mitnehmen, und eine Einstellungsdatei soll man von
Hand lesen können.

## `diktat` — das Diktat

| Spalte | Typ | Bedeutung |
|---|---|---|
| `id` | INTEGER PK | |
| `created_at` | TEXT | ISO-8601, lokal |
| `audio_dir` | TEXT NULL | Arbeitsverzeichnis. **`NULL` heißt: Transkription abgeschlossen** |
| `raw_text` | TEXT NOT NULL DEFAULT `''` | das rohe Transkript |
| `polished_text` | TEXT NULL | das polierte Transkript — genau ein Slot |

`audio_dir` ist das einzige Zustandsfeld. Es gibt kein `status`, kein
`closed_at`, kein `pinned`, kein `handed_over_seq`. Ein Feld, das man setzen
kann, ohne dass sich Verhalten ändert, existiert nicht.

## `prompt` — die Bibliothek

| Spalte | Typ | Bedeutung |
|---|---|---|
| `id` | INTEGER PK | |
| `alias` | TEXT UNIQUE | Anzeigename, z. B. „DE-sachlich" |
| `is_system` | INTEGER | 1 = System-Prompt (Rolle, Ausgabeformat), 0 = User-Prompt (die Aufgabe) |
| `prompt_text` | TEXT | die Anweisung; im User-Prompt ist `{{text}}` das rohe Transkript |

Die Auswahl besteht aus **zwei** Zeilen: einem System-Prompt und einem
User-Prompt. Sie werden unabhängig gewählt; es gibt keine Verknüpfung und
damit keine Fremdschlüsselspalte.

## Das Arbeitsverzeichnis

Solange ein Diktat transkribiert wird, hält sein Arbeitsverzeichnis je Fragment
**eine** Datei:

```
audio/42/
  041.wav     noch nicht transkribiert
  042.txt     transkribiert, Audio bereits gelöscht
  043.wav
```

- Drei Ziffern, nullgefüllt, damit lexikografische und numerische Ordnung
  dasselbe sind. Ab 1000 Fragmenten wird die Transkription mit einem Fehler
  abgebrochen, statt falsch zu sortieren (999 Fragmente ≈ 5,8 h bei 21 s).
- Die Mischung ist kein Fehler, sondern die Anzeige: nur `.wav` heißt offen,
  `.txt` heißt erledigt.
- Ein **leeres** `.txt` ist gültig und bedeutet „dieses Fragment war Stille".
  Es muss geschrieben werden, sonst wird die zugehörige `.wav` endlos neu
  transkribiert.
- Geschrieben wird über `NNN.txt.tmp` und `rename`; ein abgebrochener
  Schreibvorgang darf nicht als erledigt gelten.
- Am Ende wird in Fragmentreihenfolge verkettet. Die `.txt`-Dateien enden
  **ohne** abschließenden Zeilenumbruch — der Zwischenraum an einer Naht gehört
  der Naht-Reparatur, nicht dem Dateischreiber.
- Die Nummerierung beginnt bei `001` und ist lückenlos; `999` ist die höchste
  Nummer. Das Arbeitsverzeichnis heißt `<Datenverzeichnis>/audio/<diktat-id>/`
  und entsteht mit dem Diktat.
- Nach einem Absturz kann **beides** liegen (`NNN.txt` und `NNN.wav`): die `.txt`
  gewinnt, das Fragment gilt als erledigt, die `.wav` ist nur noch nicht
  abgeräumt. Eine halb geschriebene `NNN.txt.tmp` zählt nicht als Fortschritt.
- Fehlt das Verzeichnis, obwohl `audio_dir` gesetzt ist, gilt der committete
  `raw_text` als fertiger Text — Invariante 3 hat ihn vorher geschrieben. Der
  Abschluss wird dann nur noch zu Ende geführt.

## Die zehn Invarianten

1. **`audio_dir` gesetzt ⇔ Transkription nicht abgeschlossen.** Es gibt kein
   zweites Zustandsfeld.
2. **Text vor Audio, auf Fragmentebene:** `NNN.txt` ist geschrieben und
   umbenannt, **bevor** `NNN.wav` gelöscht wird.
3. **Text vor Audio, auf Diktatebene:** `raw_text` ist committet, **bevor** das
   Verzeichnis entfernt und `audio_dir` auf `NULL` gesetzt wird.
4. **`raw_text` ist die einzige Speicherung des rohen Transkripts.** Keine
   Blöcke, keine `seq`, keine Redundanz.
5. **Append nur bei abgeschlossener Transkription.** Solange `audio_dir`
   gesetzt ist, wird Append verweigert (typisierter Fehler, kein Panic).
6. **Jede Änderung am rohen Transkript verwirft das polierte:** Append und
   Korrektur setzen `polished_text` auf `NULL`. Es folgt **keine**
   automatische Neupolitur.
7. **Ein poliertes Transkript pro Diktat.** **Prompt anwenden** überschreibt es
   vollständig.
8. **Die Zwischenablage hält immer genau den Text, der im Fenster steht.** Sie
   wird beim Abschluss atomar mit dem Erscheinen des Textes geschrieben und
   danach bei jeder **Lieferung** neu — nie ein Text, den der Nutzer nicht
   sieht. Vier Auslöser: Abschluss, **Prompt anwenden**, Verlassen einer
   bearbeiteten Ansicht, Knopf **In die Zwischenablage**. Ein optionales
   Einfügen folgt nur dem Abschluss unmittelbar und schreibt sie nicht neu;
   **Prompt anwenden**, das Verlassen einer bearbeiteten Ansicht und der Knopf
   fügen nie ein.
9. **Der FIFO hält höchstens N Diktate** (N aus `config.json`). Kommt eines
   hinzu, fällt das älteste samt Arbeitsverzeichnis heraus. Es gibt kein
   Archiv und keine Pagination.
10. **Das rohe Transkript wird nie umformatiert gespeichert** — kein Wrapping,
    keine Normalisierung, keine stille Korrektur.

## Fähigkeitsleiter

Der Kern kennt **keine** Plattform: kein `cfg(target_os)`, kein
`enum Platform`. Jede **Plattformschale** meldet beim Start, was sie kann; der
Kern leitet sein Verhalten daraus ab.

| Fähigkeit | Werte | Was sie entscheidet |
|---|---|---|
| `global_hotkey` | ja / nein | ob die drei Hotkeys (diktieren roh, diktieren poliert, pausieren/fortsetzen) registriert werden können |
| `foreground_window` | ja / nein | `same_window` — ohne diese Fähigkeit wird der Modus `never`, nie `always` |
| `insert_keystroke` | ja / nein / ja-mit-freigabe | ob überhaupt eingefügt werden kann; `ja-mit-freigabe` heißt: erst nach einer Freigabe durch den Nutzer |
| `clipboard_envelope` | `cf_html` / `html_fragment` | ob die Schale den `CF_HTML`-Umschlag baut (Windows) oder das nackte Fragment schreibt (macOS, Linux) |
| `clipboard_exclusion` | `api` / `convention` / `keine` | was als History-/Cloud-Ausschluss zugesagt werden darf (siehe „Lieferung") |
| `tray` | ja / mit-erweiterung | Tray-Zustand; `mit-erweiterung` heißt: Overlay-Pille zusätzlich (Windows 11) |
| `autostart`, `secret_storage`, `audio_host` | plattformabhängig | Autostart-Weg, Ablage der API-Schlüssel, Audiogerät |

**Degradationsregel:** Was die Plattform nicht kann, fällt auf die **sichere
Seite**. `same_window` ohne `foreground_window` wird `never`, nie `always` — nie
einzufügen ist immer richtig, der Text liegt in der Zwischenablage. Es gibt
keinen Weg, auf dem eine fehlende Fähigkeit zu `always` führt.

Die Fähigkeitsleiter ist die Testsuite: eine Fake-Plattform mit
`{foreground_window: nein, insert_keystroke: nein}` ist Wayland, eine mit
`{ja, ja}` ist X11 oder Windows. Derselbe Kern, zwei Sätze, zwei erwartete
Verhalten — ohne Hardware.

### Gemessen je Kompositor (Linux, #12)

| Fähigkeit | Wayland/GNOME (think, 2026-11-20) | X11/XFCE (elite, Xubuntu, 2026-11-21) |
|---|---|---|
| `global_hotkey` | ja, über `GlobalShortcuts` — aber nur mit gedeckter App-ID und installierter `.desktop`; ohne Backend (GNOME 45–47) **nein** | ja, über `XGrabKey`; das Portal **existiert dort nicht** |
| `foreground_window` | **nein**, keine API | ja, `_NET_ACTIVE_WINDOW` in unter 3 ms |
| `insert_keystroke` | **nein** — `enigo`/`xdotool` melden Erfolg und tippen nichts | ja, XTEST erreicht das fremde Fenster (gemessen); für die Lieferung genügt **ein** Strg+V |
| `clipboard_envelope` | `html_fragment` | `html_fragment` |
| `clipboard_exclusion` | `keine` | `keine` — der Hinweis wird vom Clipboard-Manager ignoriert |
| `tray` | ja, AppIndicator/SNI (Erweiterung nötig) | ja, StatusNotifierItem; XEmbed-Systray fehlt im Panel |

Vier Regeln folgen daraus für jede X11-Schale:

- **`Ctrl+Alt+F<n>` wird nie angeboten.** Der X-Server behandelt die Kombination
  vor jedem Client-Grab; ein synthetischer Anschlag schaltet den Bildschirm auf
  eine Textkonsole um (`PITFALLS.md`, „X11").
- **Ein Hotkey braucht vier Grabs**: `plain`, `+Lock`, `+Mod2`, `+Lock+Mod2`.
  Sonst ist er bei aktivem CapsLock stumm.
- **Eingefügt wird über die Zwischenablage, nicht getippt.** Die Lieferung
  schreibt das Fragment und sendet **ein** Strg+V; XTEST kann auch beliebigen
  Unicode tippen, das kostet aber ~25 ms pro Zeichen (siehe `PITFALLS.md`) und
  ist kein Weg für ein Transkript.
- **Vordergrundfenster**: erstes Item von `_NET_ACTIVE_WINDOW` lesen, Fenster
  validieren, Toplevel und Eingabefokus nicht verwechseln; `same_window`
  vergleicht das validierte Toplevel.

## Ablauf: roh

1. Diktier-Hotkey „diktieren roh" → Aufnahme beginnt; ein `diktat` entsteht mit
   `audio_dir`, `raw_text` ist leer. Der Hotkey ist ein **Umschalter**: der
   erste Druck startet, der zweite beendet.
2. Alle ~21 s wird ein Fragment geschnitten und als `NNN.wav` abgelegt.
3. Das Fragment wird an den STT-Dienst geschickt, mit exponentiellem Backoff
   bis es angenommen ist. Das Transkript erscheint sofort im Reiter „rohes
   Transkript".
4. Antwort da → `NNN.txt.tmp` schreiben, `rename`, **dann** `NNN.wav` löschen.
5. Space (Pause/Fortsetzen) und Enter (Ende) gelten **nur im Diktierplatz**,
   also im fokussierten Workbench-Fenster. Ohne Fensterfokus pausiert der
   Pause-Hotkey, und ein Diktier-Hotkey beendet die Aufnahme — während einer
   Aufnahme beenden **beide**. Es wird keine fremde Taste geschluckt: kein
   Tastenhook, kein Push-to-talk.
6. Nach dem Ende: warten, bis das letzte Fragment als `.txt` vorliegt. Dann
   verketten, `raw_text` schreiben, **dann** die Zwischenablage füllen, **dann**
   das Verzeichnis löschen und `audio_dir` auf `NULL`. Tray: `READY`. Danach
   folgt das optionale Einfügen, wenn `insert_after_dictation` es erlaubt, die
   Fähigkeiten es zulassen und das Fenster dasselbe ist.

## Ablauf: poliert

Gleich bis Schritt 6, mit zwei Unterschieden:

- Die Zwischenablage bleibt **leer**, bis das Ergebnis vorliegt.
- Der vollständige `raw_text` geht als **ein** Aufruf an das LLM (gewählter
  System- und gewählter User-Prompt). Passt der Text nicht in das Modell, gibt
  es einen Fehler — **nie** eine Kürzung.

Kommt die Antwort, werden `polished_text`, der zweite Reiter und die
Zwischenablage in einem Zug gesetzt; danach folgt das optionale Einfügen. Tray:
`POLISHED`. **Prompt anwenden** fügt nie ein.

Wird poliert, obwohl die Auswahl unvollständig ist (kein User-Prompt gewählt),
verweigert die App den Aufruf mit einem Hinweis, statt still roh zu liefern.

## Ablauf: Diktat aus der Zwischenablage

Ein neues Diktat kann statt einer Aufnahme einen Text aus der Zwischenablage
aufnehmen — der Weg für lange Protokolle, die `aitranscribe` bereits
transkribiert hat:

`diktat` anlegen, `audio_dir` bleibt `NULL`, der eingefügte Text wird
`raw_text`. Danach ist das Diktat sofort append- und polierfähig. Woher der
Text kam, wird **nicht** gespeichert.

## Ablauf: Bearbeiten und Liefern

Beide Reiter sind editierbar. Das Bearbeiten folgt diesen Regeln:

- Ein Edit im **rohen** Transkript verwirft das polierte (Invariante 6) —
  sichtbar, ohne Rückfrage und ohne automatische Neupolitur.
- Ein Edit im **polierten** Transkript überschreibt es; es bleibt bei genau
  einem Slot.
- Verlässt der Nutzer eine bearbeitete Ansicht, geht deren Text **sofort und
  bedingungslos** in die Zwischenablage: ohne Rückfrage, ohne Rücksicht auf den
  vorhandenen Inhalt, in beiden Formaten und mit den Ausschlussformaten aus
  „Lieferung". Maßgeblich ist die **zuletzt bearbeitete** Ansicht.
- Es folgt **kein** Einfügen. Einfügen ist an den Abschluss gebunden.
- Der Knopf **In die Zwischenablage** liefert den **sichtbaren** Reiter
  jederzeit: ohne Bearbeitung, ohne Verlassen, ohne Einfügen. Er ist der Weg für
  ein nur gelesenes Diktat aus dem FIFO und für den Fall, dass inzwischen eine
  andere Anwendung die Zwischenablage überschrieben hat. Er liefert immer die
  sichtbare Ansicht — „zuletzt bearbeitet" gilt nur für die Automatik.
- Wurde nichts bearbeitet, wird auch nichts geliefert; die Zwischenablage
  behält, was sie hat.
- Nach einem Rohtext-Edit setzt **Polieren** die Zwischenablage, den zweiten
  Reiter und `polished_text` in einem Zug, sobald die Antwort da ist — wie beim
  Abschluss.

Der Auslöser von „verlassen" ist auf dem Desktop offen (Fokusverlust,
Fensterschließen oder ein Knopf, siehe `HANDOFF.md`); auf Android ist es die
geschlossene Tastatur. Die Lieferregel gilt unabhängig davon, nur ihr Ereignis
fehlt noch.

Scheitert das Schreiben, weil eine andere Anwendung die Zwischenablage hält,
bleibt der Text im Fenster und der Fehler wird dort gemeldet — kein stiller
Fehlschlag.

## Lieferung

Die Lieferung ist **eine** Funktion des Kerns — und **drei** Schreibvorgänge in
den Schalen:

`deliver(markdown) -> Delivery { html, text }`. `html` ist ein **semantisches
Fragment** — Absatz, Überschrift, ungeordnete Liste, fett. `text` ist derselbe
Inhalt ohne Markdown, geordnete Listen als `1. …`. Wie das Fragment auf die
**Systemzwischenablage** kommt, entscheidet die Plattformschale
(`clipboard_envelope`, `clipboard_exclusion`); der Kern kennt weder `CF_HTML`
noch ein Pasteboard.

Für alle drei Ziele gleich:

- Beide Formate gehen **zusammen** in die Zwischenablage: das HTML-Fragment und
  der reine Text. Nie nur Text — reiner Text transportiert keine Struktur.
- Das polierte Transkript darf nur Absatz, Überschrift, ungeordnete Liste und
  fett enthalten. **Geordnete Listen werden als Text mit ihren Ziffern
  ausgegeben**, weil deutsche Rubrum-Nummerierung Text ist und keine Struktur.
  Vor dem Schreiben wird validiert; ein Verstoß wird gemeldet und nicht
  stillschweigend repariert.
- Nach dem Schreiben darf die App **einmal** Strg+V senden (macOS: Cmd+V),
  gesteuert durch `insert_after_dictation` (`never`, `same_window`, `always`;
  Default `same_window`). Bei `same_window` wird nur eingefügt, wenn das
  Vordergrundfenster noch dasselbe ist wie beim Abschluss — das setzt die
  Fähigkeit `foreground_window` voraus, sonst wird der Modus `never`; kein
  Fenster wird reaktiviert. **Nie ins eigene Fenster:** ist die Workbench das
  Vordergrundfenster, unterbleibt das Einfügen, auch bei `always`. Die
  Zwischenablage behält den Text in jedem Fall.

### Formate und Ausschluss je Plattform

| Plattform | Formate | Ausschluss aus History/Cloud | Belastbarkeit auf diesem Ziel |
|---|---|---|---|
| Windows | `CF_HTML` + `CF_UNICODETEXT` | harte API: `CanIncludeInClipboardHistory=0`, `CanUploadToCloudClipboard=0`, `ExcludeClipboardContentFromMonitorProcessing` | **erzwungen** — der Marker gilt für den gesamten Inhalt |
| macOS | `public.html` + `public.utf8-plain-text` | De-facto-Konvention `org.nspasteboard.ConcealedType` (+ `TransientType`) | **Konvention** — von Maccy, PastePal u. a. honoriert, vom OS nicht erzwungen |
| Linux | `text/html` + `text/plain;charset=utf-8` | praktisch nichts; `x-kde-passwordManagerHint=secret` wurde gemessen mitgeliefert und von `xfce4-clipman` **ignoriert** | **keine Zusage** — die Doku verspricht dort keinen Ausschluss |

- Die Ausschlussformate wirken **global** für den gesamten Inhalt, nicht je
  Datenformat — `CF_HTML` daneben zu legen hebt den Ausschluss nicht auf.
- Tauris `writeSecret()` markiert plattformgerecht, kann aber kein HTML mit
  Markern; deshalb bleiben eigene Plattformmodule für die Lieferung.

### Windows: der `CF_HTML`-Umschlag

Der Umschlag ist eine Funktion des **Windows-Schreibers**, nicht der Domäne:
Linux (`text/html`) und macOS (`public.html`) nehmen das nackte Fragment.

- Format über `RegisterClipboardFormat("HTML Format")`: ASCII-Kopfzeile, dann
  das HTML.
- Kopfzeile: `Version:1.0`, `StartHTML`, `EndHTML`, `StartFragment`,
  `EndFragment`. Alle Werte sind **Byte-Offsets** ab Anfang der Nutzlast und
  werden zehnstellig nullgefüllt geschrieben, damit die Kopflänge beim Rechnen
  konstant bleibt.
- Das Fragment steht zwischen `<!--StartFragment-->` und `<!--EndFragment-->`;
  `StartFragment`/`EndFragment` zeigen auf den Inhalt dazwischen, nicht auf die
  Marker.
- UTF-8 mit `<meta charset="utf-8">`; Offsets zählen Bytes, nicht Zeichen — bei
  Umlauten ist das der Unterschied.
- `CF_UNICODETEXT` trägt denselben Inhalt ohne Markdown; geordnete Listen als
  `1. …`.

### macOS und Linux: das nackte Fragment

- macOS: `public.html` und `public.utf8-plain-text`, dazu
  `org.nspasteboard.ConcealedType` (und `TransientType`) setzen. Das OS erzwingt
  den Marker nicht; Clipboard-Manager halten sich daran.
- Linux: `text/html` und `text/plain;charset=utf-8`, dazu
  `x-kde-passwordManagerHint=secret` (KF5/KF6 und einige Manager kennen ihn).
  Einen wirksamen History-Ausschluss gibt es praktisch nicht: auf Xubuntu legte
  `xfce4-clipman` den markierten Inhalt trotz Hint in seine History (gemessen
  2026-11-21, `PITFALLS.md` „X11"). Die Zusage lautet hier deshalb **nicht**
  „nichts landet in der History".

## Fehlerfälle

| Fall | Verhalten |
|---|---|
| STT antwortet nicht | `.wav` bleibt liegen, Backoff, Tray `FEHLER`; das Verzeichnis bleibt für die Wiederaufnahme |
| Modell kann den Text nicht | kein Abschneiden; der Rohtext geht als Rückfall in die Zwischenablage; Tray `FEHLER` |
| Schlüssel abgelehnt | wie oben, plus Handlungsanweisung im Fenster |
| nur Stille aufgenommen | leere `.txt` je Fragment, `raw_text` leer, nichts in die Zwischenablage, Meldung |
| Plattform kann nicht einfügen (Fähigkeit fehlt) | kein Einfügen und **kein** Fehler — der Text bleibt in der Zwischenablage; `same_window` fällt auf `never` |
| Hotkey schon belegt (`BadAccess`) | die Registrierung scheitert erkennbar (0,3 ms), nicht still: die Schale meldet die Kombination als belegt, die anderen beiden bleiben nutzbar |
| Absturz mitten in der Aufnahme | das Verzeichnis bleibt; beim Start werden die restlichen `.wav` transkribiert **und** der Abschluss läuft |
| Diktat fällt aus dem FIFO | das Arbeitsverzeichnis wird mitgelöscht |

## Wiederaufnahme beim Start

Beim Start werden alle Diktate mit gesetztem `audio_dir` gefunden. Für jedes
werden die verbliebenen `.wav` transkribiert; danach läuft derselbe Abschluss
wie nach dem Ende der Aufnahme (verketten, `raw_text` schreiben, Verzeichnis
entfernen). Ohne diesen zweiten Schritt bliebe das Diktat in einer Sackgasse
hängen: alle Fragmente erledigt, aber kein Text und kein Ende.

## Audio

Audio wird nicht aufbewahrt. Jede Fragmentdatei verschwindet, sobald ihr
Transkript auf der Platte steht; das Verzeichnis verschwindet mit dem Abschluss.
Es gibt keine Medienbibliothek und keine Audio-Historie. Wird Evidenz gewünscht,
gilt das Session-Bundle-Format aus `polished-recognition`, damit der dortige
Kotlin-`:analyzer` die Desktop-Sitzungen mitlesen kann.

## Nicht Teil des Modells

- Run-Verlauf, Prompt-Versionierung, Fingerprints.
- Herkunftsangabe am polierten Transkript (Prompt, Modell, Zeitpunkt).
- Blockgrenzen, `seq`, Übergabezähler.
- Konten, Synchronisation, Cloud.
- Einfüge-Protokoll pro Zielanwendung.
- Plattformweichen im Kern: kein `cfg(target_os)`, kein `enum Platform`, keine Feature-Flags pro Betriebssystem. Der Kern kennt nur die **Fähigkeitsleiter**.
