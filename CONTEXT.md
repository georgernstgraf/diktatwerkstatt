# CONTEXT — Vokabular

Jedes Wort bedeutet hier genau eine Sache. Wer einen Begriff neu einführt,
trägt ihn hier ein.

## Kernbegriffe

| Begriff | Bedeutung |
|---|---|
| **Diktat** | Ein Eintrag im Ring. Container aus Rohtextblöcken, einer Politur und einer Übergabespur. Ein Diktat ist entweder offen oder geschlossen. |
| **Rohtextblock** | Ein Transkriptionsergebnis, das an ein Diktat gehängt wurde. Kleinste Einheit des Rohtexts. |
| **Append** | Einen weiteren Rohtextblock an das offene Diktat hängen. Betrifft Rohtext, nie Audio. |
| **Offenes Diktat** | Das jüngste, noch nicht geschlossene Diktat. Es gibt höchstens eines. |
| **Geschlossenes Diktat** | Eingefroren. Der Rohtext wächst nicht mehr; Prompts dürfen weiter darauf laufen. |
| **Ring** | Die letzten 5–7 Diktate. Das älteste fällt heraus, sofern es nicht `pinned` ist. |
| **Prompt** | Eine benannte Anweisung mit Alias, jederzeit auf den Rohtext eines Diktats anwendbar. Beispiele: „EN-freundlich", „FR-böse", „DE-sachlich". |
| **Politur** | Das Ergebnis des letzten Play über das ganze Diktat. Genau ein Slot pro Diktat, keine Historie. |
| **Play** | Einen Prompt auf den Rohtext anwenden. Schreibt die Politur und legt sie in die Zwischenablage. |
| **Auswahl** | Der im Dropdown gewählte Prompt — das, was der *nächste* Play benutzt. |
| **Angewandter Prompt** | Der Prompt, aus dem die *aktuelle* Politur entstanden ist. Steht in der Politur, nicht im Dropdown. |
| **Delta** | Der Teil des Rohtexts, der noch nicht an die Zielanwendung übergeben wurde. |
| **Zielanwendung** | Das Programm, in das eingefügt wird: Word, Google Docs, Notepad, ein Terminal. |
| **Fast Path** | Hook → Audio → Fragmentierung → STT → Zwischenablage → Tray. Läuft ohne Webview und ohne JavaScript. |
| **Workbench** | Das Fenster: Reiter „Roh" und „gerendert", Prompt-Liste, Play. |
| **Tray-Zustand** | Einer von `IDLE`, `RECORDING`, `PAUSED`, `TRANSCRIBING`, `POLISHING`, `READY`, `POLISHED`, `FEHLER`. |

## Geerbte Begriffe

Aus `polished-recognition` übernommen, Bedeutung unverändert — sie beschreiben
die Audioseite, nicht die Textseite:

| Begriff | Bedeutung |
|---|---|
| **Fragment** | Ein während der Aufnahme geschnittenes Audiostück (~21 s). |
| **Naht** | Die Grenze zwischen zwei Fragmenten. Zentralbegriff des Verfahrens. |
| **Q** | Tiefe × Dauer eines Stillekandidaten unter der adaptiven Achse `X = 0,65·M + 100`. Dimensionslos, verstärkungsinvariant. |
| **Q-Gate** | Bedingung `Q < quality_min`; trifft sie zu, wird die Naht konditioniert. |
| **Pre-roll / Prompt-Carry-over / Echo-Trim** | Die drei Reparaturen an einer konditionierten Naht. |
| **Session-Bundle** | `session.json` plus `uploads/` als vollständige Evidenz einer Sitzung. |

## Bewusste Abgrenzungen

- **„Append" heißt hier nicht PCM anhängen.** In `polished-recognition` meint
  append das Anhängen von Audio an den Puffer. Hier heißt es: einen
  Rohtextblock anhängen.
- **„Chunk" ist kein Audiostück.** Ein Audiostück ist ein *Fragment*; ein
  Textstück ist ein *Rohtextblock*.
- **Kein Run-Verlauf.** Es gibt genau eine Politur pro Diktat. Ein neuer Play
  überschreibt sie, samt Herkunftsangabe.
- **Keine Veraltet-Markierung.** Ändert sich der Rohtext, wird die Politur
  verworfen, nicht als „veraltet" weitergeführt. Ein Zustand, der lügen kann,
  existiert nicht.

## Zustände und Übergänge

```
IDLE ──hotkey──▶ RECORDING ⇄ PAUSED
                      │
                   (stop)
                      ▼
                TRANSCRIBING ──▶ READY ──play──▶ POLISHING ──▶ POLISHED
                      │              │                            │
                   FEHLER        (append)                    (play erneut)
                                     ▼                            ▼
                                 RECORDING                    POLISHING
```
