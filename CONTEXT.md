---
type: Glossary
title: Vokabular
description: Was die Wörter in diesem Repository genau bedeuten.
tags: [wissensbasis, vokabular]
generated: { by: opencode/general, at: 2026-11-20T16:23:16Z }
status: stable
---

# CONTEXT — Vokabular

Jedes Wort bedeutet hier genau eine Sache. Wer einen Begriff neu einführt,
trägt ihn hier ein.

## Kernbegriffe

| Begriff | Bedeutung |
|---|---|
| **Plattformschale** | Die dünne, plattformspezifische Schicht um den Kern — je eine für Windows, Linux und macOS. Sie meldet beim Start den Fähigkeitssatz und setzt Kernaufträge in Plattformaufrufe um (Hotkeys, Zwischenablage, Einfügen, Tray, Autostart, Geheimnisse). Sie enthält **keine** Fachlogik. |
| **Fähigkeitsleiter** | Der Satz von Fähigkeiten, den eine Plattformschale beim Start meldet — `global_hotkey`, `foreground_window`, `insert_keystroke`, `clipboard_envelope`, `clipboard_exclusion`, `tray`, `autostart`, `secret_storage`, `audio_host`. Der Kern leitet sein Verhalten daraus ab, **nie** aus dem Betriebssystem. Was fehlt, fällt auf die sichere Seite. |
| **Diktierplatz** | Das **fokussierte** Workbench-Fenster, in dem die blanken Tasten gelten: Space = Pause/Fortsetzen, Enter = Ende der Aufnahme. Außerhalb des Diktierplatzes gelten nur die globalen Hotkeys. |
| **Diktat** | Ein Eintrag im FIFO. Ein in sich geschlossener Vorgang: ein Brief, eine Notiz, ein Besprechungsprotokoll. Besteht aus einem rohen und höchstens einem polierten Transkript. |
| **Rohes Transkript** | Der aus den transkribierten Fragmenten zusammengesetzte Text eines Diktats. Es gibt genau eines pro Diktat; es wächst durch **Append**. |
| **Poliertes Transkript** | Das Ergebnis des letzten **Prompt anwenden** über das ganze rohe Transkript. Genau ein Slot pro Diktat, keine Historie. |
| **Polieren** | Die Tätigkeit: einen Prompt auf das rohe Transkript anwenden. |
| **Append** | Weiteren Rohtext an ein Diktat hängen. Betrifft das rohe Transkript, nie Audio. Nur möglich, wenn dessen Transkription abgeschlossen ist. |
| **Fragment** | Ein während der Aufnahme geschnittenes Audiostück (14,7–21 s). Kleinste Einheit der Audioseite. |
| **Arbeitsverzeichnis** | Das Verzeichnis eines Diktats, solange dessen Transkription läuft. Es hält je Fragment entweder `NNN.wav` (noch nicht transkribiert) oder `NNN.txt` (transkribiert). Die Mischung ist der Zustand. |
| **Prompt** | Eine Anweisung an das LLM — **nie** der Text. Jeder Prompt hat einen Alias und ist entweder System-Prompt (Rolle, Ausgabeformat) oder User-Prompt (die eigentliche Aufgabe). |
| **Auswahl** | Das Paar aus gewähltem System- und gewähltem User-Prompt, mit dem der nächste Aufruf arbeitet. |
| **Prompt anwenden** | Die Auswahl auf das rohe Transkript anwenden. Schreibt das polierte Transkript und legt es in die Zwischenablage. |
| **FIFO** | Die letzten N Diktate, N aus der Konfiguration. Kommt eines hinzu, fällt das älteste samt Arbeitsverzeichnis heraus. |
| **Zielanwendung** | Das Programm, in das der Nutzer den Text einfügt: Word, Google Docs, Notepad, ein Terminal. |
| **Diktier-Hotkey** | „diktieren roh" oder „diktieren poliert" — ein **Umschalter**: erster Druck startet die Aufnahme, zweiter beendet sie. Während einer Aufnahme beenden **beide**. Kein Push-to-talk: ohne Tastenhook gibt es kein Key-up. |
| **Pause-Hotkey** | Der dritte globale Hotkey: pausiert eine laufende Aufnahme und setzt sie fort. Er wirkt auch ohne Fensterfokus. |
| **Einfügen** | Das einmalige Senden von Strg+V (macOS: Cmd+V) nach dem Abschluss eines Diktats. Nie bei **Prompt anwenden**. Steht in `insert_after_dictation` (`never`, `same_window`, `always`); die Plattform-Defaults folgen der **Fähigkeitsleiter**, und in das eigene Fenster wird **nie** eingefügt. |
| **Lieferung** | Das Schreiben des Ergebnisses in die **Systemzwischenablage**: das semantische `html`-Fragment und der reine Text, dazu der Ausschluss aus History/Cloud, den die Plattform kennt. Immer, unabhängig vom Einfügen: beim Abschluss, bei **Prompt anwenden**, beim Verlassen einer bearbeiteten Ansicht und auf Knopfdruck. Die Zwischenablage behält den Text. |
| **In die Zwischenablage** | Der Knopf in der Workbench: liefert den **sichtbaren** Reiter jederzeit — ohne Bearbeitung, ohne Verlassen, ohne Einfügen. Der Weg für ein nur gelesenes Diktat und für eine inzwischen überschriebene Zwischenablage. |
| **Zwischenablage** | Die **Systemzwischenablage** des jeweiligen Ziels — unter Windows, Linux und macOS die eigene. Sie ist die einzige Schnittstelle zur Zielanwendung. |
| **Fast Path** | Hotkey → Audio → Fragmente → STT → rohes Transkript. Läuft ohne Webview und ohne JavaScript. |
| **Bearbeiten** | Manuelle Änderung eines Transkripts in der Workbench — roh oder poliert. Ein Rohtext-Edit verwirft das Polierte. Wer eine Ansicht bearbeitet hat, **liefert** sie beim Verlassen; maßgeblich ist die zuletzt bearbeitete Ansicht. |
| **Workbench** | Das Fenster: Reiter „rohes Transkript" und „poliertes Transkript" — **beide editierbar** —, Prompt-Auswahl, **Prompt anwenden**, Knopf **In die Zwischenablage**. Ist es fokussiert, ist es der **Diktierplatz**. |
| **Tray-Zustand** | Einer von `IDLE`, `RECORDING`, `PAUSED`, `TRANSCRIBING`, `POLISHING`, `READY`, `POLISHED`, `FEHLER`. |

## Geerbte Begriffe

Aus `polished-recognition` übernommen, Bedeutung unverändert — sie beschreiben
die Audioseite, nicht die Textseite:

| Begriff | Bedeutung |
|---|---|
| **Naht** | Die Grenze zwischen zwei Fragmenten. Zentralbegriff des Verfahrens. |
| **Slice** | Die **Obergrenze** eines Fragments (Default 21 s). Sie bestimmt, wann entschieden wird: erst wenn sie vollständig aufgenommen ist. |
| **Suchfenster** | Die letzten 30 % der Slice (Default 6,3 s) — der einzige Bereich, in dem ein Schnitt liegen darf. |
| **Q** | Tiefe × Dauer eines Stillekandidaten unter der adaptiven Achse `X = 0,65·M + 100`. Dimensionslos, verstärkungsinvariant. |
| **Q-Gate** | Bedingung `Q < quality_min`; trifft sie zu, wird die Naht konditioniert. Ungemessenes `Q` konditioniert ebenfalls. |
| **Zwangsschnitt** | Kein Rahmen unter der Achse ⇒ Schnitt am leisesten 10-ms-Rahmen, `Q = 0`, und die Naht wird konditioniert. |
| **Pre-roll / Prompt-Carry-over / Echo-Trim** | Die drei Reparaturen an einer konditionierten Naht: die letzte Sekunde des Vorfragments — **im Speicher, nie eine Datei** —, sein Textende als Prompt (≤ 96 Zeichen), das Entfernen des wiederholten Kopfs (≤ 8 Token). |
| **Session-Bundle** | `session.json` plus `uploads/` als vollständige Evidenz einer Sitzung. |

## Bewusste Abgrenzungen

- **„Fragment" ist immer Audio, nie Text.** Ein Textstück heißt Satz, Absatz
  oder — als Ganzes — rohes Transkript. Es gibt keinen „Rohtextblock" und
  keinen „Chunk" mehr.
- **„Prompt" ist nie der Text.** Wer vom Ergebnis spricht, meint das polierte
  Transkript.
- **FIFO ist kein Ring.** Eine Warteschlange hat ein Ende; einen Kreis gibt es
  hier nicht, und „Ring" hat lange eine Struktur behauptet, die es nie gab.
- **Das rohe Transkript ist ein Text, keine Liste.** Es gibt keine Blöcke,
  keine `seq`, keine Blockgrenzen in der Datenbank. Wo ein Fragment anfing,
  weiß nur das Arbeitsverzeichnis, und nur solange es existiert.
- **Es gibt kein Zustandsfeld.** Ob ein Diktat offen ist, sagt allein das
  Arbeitsverzeichnis: existiert es, ist die Transkription nicht fertig.
- **Kein Run-Verlauf.** Es gibt genau ein poliertes Transkript pro Diktat. Ein
  neues Prompt anwenden überschreibt es.
- **Keine Veraltet-Markierung.** Ändert sich das rohe Transkript, wird das
  polierte Transkript verworfen, nicht als „veraltet" weitergeführt. Ein
  Zustand, der lügen kann, existiert nicht.
- **Keine Herkunftsangabe am polierten Transkript.** Welcher Prompt und
  welches Modell es erzeugt haben, wird nicht gespeichert.
- **Der Knopf ist nicht der einzige Weg.** Die Lieferung passiert auch ohne ihn:
  beim Abschluss, bei **Prompt anwenden** und beim Verlassen einer bearbeiteten
  Ansicht. Umgekehrt ist er der einzige Weg zu einem Text, den die Automatik
  nicht kennt — ein nur gelesenes Diktat, eine inzwischen überschriebene
  Zwischenablage.
- **„Zwischenablage" heißt Systemzwischenablage, nicht Windows-Zwischenablage.**
  Die frühere Definition „**Zwischenablage** — die Windows-Zwischenablage.
  Sie ist die einzige Schnittstelle zur Zielanwendung" **entfällt** mit der
  Entscheidung „Drei Ziele, ein Kern" (2026-11-20): gemeint war immer die
  Schnittstelle zur Zielanwendung, und die ist auf jedem Ziel eine andere.
- **Es wird keine fremde Taste geschluckt.** Die frühere Regel „Leertaste
  pausiert und setzt fort; sie wird **nur** in `RECORDING` geschluckt"
  **entfällt** mit der Hook-Entscheidung (2026-11-20): Space und Enter gelten
  nur noch im **Diktierplatz**, also im fokussierten eigenen Fenster. Kein
  Tastenhook, kein Push-to-talk, kein Schlucken.
- **Der Kern kennt keine Plattform.** Kein `cfg(target_os)`, kein
  `enum Platform`: der Kern fragt die **Fähigkeitsleiter** der Schale ab.
  „Windows-only", „Plattformweiche im Kern" und „Feature-Flag pro Betriebssystem"
  sind verworfen.
- **Drei Ziele, ein Kern, drei dünne Schalen.** Windows, Linux und macOS teilen
  sich Domäne, Datenmodell und Abläufe; verschieden ist nur die Schale. Die
  Reihenfolge der Auslieferung ist **offen**; gebaut wird zuerst auf Linux
  (erste Schale), Windows bleibt das Produkt.

## Zustände und Übergänge

```
IDLE ──Hotkey──▶ RECORDING ⇄ PAUSED          (Arbeitsverzeichnis entsteht)
                     │
                  (Enter im Fenster)
                     ▼
               TRANSCRIBING ──────────────▶ READY ──"Prompt anwenden"──▶ POLISHING
                     │                        │                            │
                  (STT-Fehler)                │                       (Antwort)
                     ▼                        │                            ▼
                  FEHLER ◀────────────────────┴────────────────────── FEHLER
                                                                            │
                                                                       POLISHED
```

Zwei Scopes: **global** wirken die drei Hotkeys (diktieren roh, diktieren
poliert, pausieren/fortsetzen); **fokussiert** im Diktierplatz zusätzlich Space
(Pause) und Enter (Ende). Beides ist kein Tastenhook — die App fängt nur im
eigenen Fenster ab.

| Von | Nach | Auslöser |
|---|---|---|
| `IDLE` | `RECORDING` | Diktier-Hotkey „diktieren roh" oder „diktieren poliert" (Umschalter, erster Druck); neues Diktat, Arbeitsverzeichnis entsteht |
| `RECORDING` | `PAUSED` | **zwei Auslöser:** Space im **Diktierplatz** oder der **Pause-Hotkey** (global, auch ohne Fensterfokus) |
| `PAUSED` | `RECORDING` | derselbe Auslöser: Space im **Diktierplatz** oder der **Pause-Hotkey** |
| `RECORDING` | `TRANSCRIBING` | Enter im **Diktierplatz** oder ein Diktier-Hotkey (Umschalter, zweiter Druck — während einer Aufnahme beenden **beide**). Es wird gewartet, bis das letzte Fragment transkribiert ist |
| `TRANSCRIBING` | `READY` | alle Fragmente transkribiert, roh gewählt: Zwischenablage = rohes Transkript |
| `TRANSCRIBING` | `POLISHING` | alle Fragmente transkribiert, poliert gewählt: ein Aufruf an das LLM |
| `POLISHING` | `POLISHED` | Antwort da: poliertes Transkript in Fenster **und** Zwischenablage |
| `POLISHING` | `FEHLER` | LLM-Fehler; der Rohtext geht als Rückfall in die Zwischenablage |
| `TRANSCRIBING` | `FEHLER` | STT-Fehler; das Arbeitsverzeichnis bleibt und wird beim nächsten Start fortgesetzt |
| `READY`, `POLISHED`, `FEHLER` | `POLISHING` | **Prompt anwenden** |
| `READY`, `POLISHED`, `FEHLER` | `RECORDING` | nächste Aufnahme; das vorige Diktat bleibt im FIFO |
| `READY` | `POLISHED` | **Prompt anwenden** über einen bereits polierten Stand |

Während `RECORDING` transkribiert die App im Hintergrund. Das Tray zeigt das
nicht — es bleibt eine Ampel, kein Dashboard. `TRANSCRIBING` erscheint nur,
wenn nichts mehr aufgenommen wird.
