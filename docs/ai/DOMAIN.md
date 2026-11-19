---
type: Domain
title: Domänenmodell
description: Tabellen, Invarianten, Abläufe und Arbeitsverzeichnis der Diktatwerkstatt.
tags: [domain, datenmodell]
generated: { by: opencode/knowledge-persistence, at: 2026-11-19T04:22:34Z }
status: stable
---

# Domänenmodell

Zwei Tabellen. Eine SQLite-Datei. Dazu eine Konfigurationsdatei und ein
Arbeitsverzeichnis pro laufendem Diktat.

## Speicherorte

| Was | Wo |
|---|---|
| Datenbank | `%LOCALAPPDATA%\diktatwerkstatt\diktatwerkstatt.db` (Windows), `~/.local/share/diktatwerkstatt/diktatwerkstatt.db` (Linux) |
| Konfiguration | `config.json` **neben** der Datenbank, nicht darin |
| Arbeitsverzeichnisse | `<Datenverzeichnis>/audio/<diktat-id>/` — temporär, verschwindet nach dem Abschluss |
| API-Schlüssel | Windows Credential Manager bzw. Secret Service — **nie** in der Datenbank oder in `config.json` |

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
   sieht. Ein optionales Einfügen folgt nur dem Abschluss unmittelbar und
   schreibt sie nicht neu; **Prompt anwenden** und das Verlassen einer
   bearbeiteten Ansicht fügen nie ein.
9. **Der FIFO hält höchstens N Diktate** (N aus `config.json`). Kommt eines
   hinzu, fällt das älteste samt Arbeitsverzeichnis heraus. Es gibt kein
   Archiv und keine Pagination.
10. **Das rohe Transkript wird nie umformatiert gespeichert** — kein Wrapping,
    keine Normalisierung, keine stille Korrektur.

## Ablauf: roh

1. Hotkey „diktieren roh" → Aufnahme beginnt; ein `diktat` entsteht mit
   `audio_dir`, `raw_text` ist leer.
2. Alle ~21 s wird ein Fragment geschnitten und als `NNN.wav` abgelegt.
3. Das Fragment wird an den STT-Dienst geschickt, mit exponentiellem Backoff
   bis es angenommen ist. Das Transkript erscheint sofort im Reiter „rohes
   Transkript".
4. Antwort da → `NNN.txt.tmp` schreiben, `rename`, **dann** `NNN.wav` löschen.
5. Leertaste pausiert und setzt fort. Enter beendet die Aufnahme.
6. Nach Enter: warten, bis das letzte Fragment als `.txt` vorliegt. Dann
   verketten, `raw_text` schreiben, **dann** die Zwischenablage füllen, **dann**
   das Verzeichnis löschen und `audio_dir` auf `NULL`. Tray: `READY`. Danach
   folgt das optionale Einfügen, wenn `insert_after_dictation` es erlaubt und
   das Fenster dasselbe ist.

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

- In die Zwischenablage gehen **beide** Formate: `CF_HTML` (semantisches HTML,
  damit die Zielanwendung ihre eigenen Formatvorlagen anwendet) und
  `CF_UNICODETEXT`.
- Dazu werden immer `CanIncludeInClipboardHistory=0`,
  `CanUploadToCloudClipboard=0` und `ExcludeClipboardContentFromMonitorProcessing`
  gesetzt. Diese Formate wirken **global** für den gesamten Inhalt, nicht je
  Datenformat — `CF_HTML` daneben zu legen hebt den Ausschluss nicht auf.
- Das polierte Transkript darf nur Absatz, Überschrift, ungeordnete Liste und
  fett enthalten. **Geordnete Listen werden als Text mit ihren Ziffern
  ausgegeben**, weil deutsche Rubrum-Nummerierung Text ist und keine Struktur.
  Vor dem Schreiben wird validiert; ein Verstoß wird gemeldet und nicht
  stillschweigend repariert.
- Nach dem Schreiben darf die App **einmal** Strg+V senden, gesteuert durch
  `insert_after_dictation` (`never`, `same_window`, `always`; Default
  `same_window`). Bei `same_window` wird nur eingefügt, wenn das
  Vordergrundfenster noch dasselbe ist wie beim Abschluss; kein Fenster wird
  reaktiviert. Die Zwischenablage behält den Text in jedem Fall.

### `CF_HTML`

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

## Fehlerfälle

| Fall | Verhalten |
|---|---|
| STT antwortet nicht | `.wav` bleibt liegen, Backoff, Tray `FEHLER`; das Verzeichnis bleibt für die Wiederaufnahme |
| Modell kann den Text nicht | kein Abschneiden; der Rohtext geht als Rückfall in die Zwischenablage; Tray `FEHLER` |
| Schlüssel abgelehnt | wie oben, plus Handlungsanweisung im Fenster |
| nur Stille aufgenommen | leere `.txt` je Fragment, `raw_text` leer, nichts in die Zwischenablage, Meldung |
| Absturz mitten in der Aufnahme | das Verzeichnis bleibt; beim Start werden die restlichen `.wav` transkribiert **und** der Abschluss läuft |
| Diktat fällt aus dem FIFO | das Arbeitsverzeichnis wird mitgelöscht |

## Wiederaufnahme beim Start

Beim Start werden alle Diktate mit gesetztem `audio_dir` gefunden. Für jedes
werden die verbliebenen `.wav` transkribiert; danach läuft derselbe Abschluss
wie nach Enter (verketten, `raw_text` schreiben, Verzeichnis entfernen). Ohne
diesen zweiten Schritt bliebe das Diktat in einer Sackgasse hängen: alle
Fragmente erledigt, aber kein Text und kein Ende.

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
