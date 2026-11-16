---
type: Glossary
title: Vokabular
description: Was die Wörter in diesem Repository genau bedeuten.
tags: [wissensbasis, vokabular]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T06:08:40Z }
status: stable
---

# CONTEXT — Vokabular

Jedes Wort bedeutet hier genau eine Sache. Wer einen Begriff neu einführt,
trägt ihn hier ein.

## Kernbegriffe

| Begriff | Bedeutung |
|---|---|
| **Diktat** | Ein Eintrag im FIFO. Ein in sich geschlossener Vorgang: ein Brief, eine Notiz, ein Besprechungsprotokoll. Besteht aus einem rohen und höchstens einem polierten Transkript. |
| **Rohes Transkript** | Der aus den transkribierten Fragmenten zusammengesetzte Text eines Diktats. Es gibt genau eines pro Diktat; es wächst durch **Append**. |
| **Poliertes Transkript** | Das Ergebnis des letzten **Prompt anwenden** über das ganze rohe Transkript. Genau ein Slot pro Diktat, keine Historie. |
| **Polieren** | Die Tätigkeit: einen Prompt auf das rohe Transkript anwenden. |
| **Append** | Weiteren Rohtext an ein Diktat hängen. Betrifft das rohe Transkript, nie Audio. Nur möglich, wenn dessen Transkription abgeschlossen ist. |
| **Fragment** | Ein während der Aufnahme geschnittenes Audiostück (~21 s). Kleinste Einheit der Audioseite. |
| **Arbeitsverzeichnis** | Das Verzeichnis eines Diktats, solange dessen Transkription läuft. Es hält je Fragment entweder `NNN.wav` (noch nicht transkribiert) oder `NNN.txt` (transkribiert). Die Mischung ist der Zustand. |
| **Prompt** | Eine Anweisung an das LLM — **nie** der Text. Jeder Prompt hat einen Alias und ist entweder System-Prompt (Rolle, Ausgabeformat) oder User-Prompt (die eigentliche Aufgabe). |
| **Auswahl** | Das Paar aus gewähltem System- und gewähltem User-Prompt, mit dem der nächste Aufruf arbeitet. |
| **Prompt anwenden** | Die Auswahl auf das rohe Transkript anwenden. Schreibt das polierte Transkript und legt es in die Zwischenablage. |
| **FIFO** | Die letzten N Diktate, N aus der Konfiguration. Kommt eines hinzu, fällt das älteste samt Arbeitsverzeichnis heraus. |
| **Zielanwendung** | Das Programm, in das der Nutzer den Text einfügt: Word, Google Docs, Notepad, ein Terminal. |
| **Einfügen** | Das einmalige Senden von Strg+V nach dem Abschluss eines Diktats. Nie bei **Prompt anwenden**. Steht in `insert_after_dictation` (`never`, `same_window`, `always`). |
| **Lieferung** | Das Schreiben des Ergebnisses in die Zwischenablage: `CF_HTML` und `CF_UNICODETEXT` plus Ausschlussformate. Immer, unabhängig vom Einfügen; die Zwischenablage behält den Text. |
| **Zwischenablage** | Die Windows-Zwischenablage. Sie ist die einzige Schnittstelle zur Zielanwendung. |
| **Fast Path** | Hotkey → Audio → Fragmente → STT → rohes Transkript. Läuft ohne Webview und ohne JavaScript. |
| **Workbench** | Das Fenster: Reiter „rohes Transkript" und „poliertes Transkript", Prompt-Auswahl, **Prompt anwenden**. |
| **Tray-Zustand** | Einer von `IDLE`, `RECORDING`, `PAUSED`, `TRANSCRIBING`, `POLISHING`, `READY`, `POLISHED`, `FEHLER`. |

## Geerbte Begriffe

Aus `polished-recognition` übernommen, Bedeutung unverändert — sie beschreiben
die Audioseite, nicht die Textseite:

| Begriff | Bedeutung |
|---|---|
| **Naht** | Die Grenze zwischen zwei Fragmenten. Zentralbegriff des Verfahrens. |
| **Q** | Tiefe × Dauer eines Stillekandidaten unter der adaptiven Achse `X = 0,65·M + 100`. Dimensionslos, verstärkungsinvariant. |
| **Q-Gate** | Bedingung `Q < quality_min`; trifft sie zu, wird die Naht konditioniert. |
| **Pre-roll / Prompt-Carry-over / Echo-Trim** | Die drei Reparaturen an einer konditionierten Naht. |
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

## Zustände und Übergänge

```
IDLE ──hotkey──▶ RECORDING ⇄ PAUSED          (Arbeitsverzeichnis entsteht)
                     │
                  (Enter)
                     ▼
               TRANSCRIBING ──────────────▶ READY ──"Prompt anwenden"──▶ POLISHING
                     │                        │                            │
                  (STT-Fehler)                │                       (Antwort)
                     ▼                        │                            ▼
                  FEHLER ◀────────────────────┴────────────────────── FEHLER
                                                                            │
                                                                       POLISHED
```

| Von | Nach | Auslöser |
|---|---|---|
| `IDLE` | `RECORDING` | Hotkey „diktieren roh" oder „diktieren poliert"; neues Diktat, Arbeitsverzeichnis entsteht |
| `RECORDING` | `PAUSED` | Leertaste. Die Leertaste wird **nur** in `RECORDING` geschluckt |
| `PAUSED` | `RECORDING` | Leertaste |
| `RECORDING` | `TRANSCRIBING` | Enter. Es wird gewartet, bis das letzte Fragment transkribiert ist |
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
