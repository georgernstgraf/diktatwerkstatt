# Diktatwerkstatt

Ein Diktier-Werkzeug für Menschen, die professionelle Texte schreiben, ohne zu
tippen: Sekretariate, Kanzleien, Assistenz.

**Status: Spezifikation.** Es gibt noch keinen Code. Dieses Repository hält
Vokabular, Domänenmodell und Entscheidungen fest, bevor die erste Zeile
geschrieben wird — nach dem Muster, mit dem TMOG (der neue Task Manager seines
Originalautors) aus einer 107-seitigen Spezifikation entstanden ist.

Das Epic ist **ausgegrillt**: die teuren Annahmen sind geprüft, zwei davon
gekippt (Word-Formatierung, Whisper-Qualität auf Spontansprache). Ergebnis im
Kommentar an [#3](https://github.com/georgernstgraf/diktatwerkstatt/issues/3).

## Was es ist

- Ein **Windows-Hilfsprogramm im Tray**, das Diktate in jede Anwendung bringt
  (Word, Google Docs, Notepad) — über die Zwischenablage, ohne Fenster im Weg.
- Eine **Workbench** mit zwei Reitern: rohes Transkript links, poliertes
  rechts, Prompt-Auswahl darunter, „Prompt anwenden".
- Ein **FIFO der letzten Diktate** — kein Archiv, keine Datenbankpflege.

## Was es nicht ist

- Keine weitere generische Diktier-App (die gibt es: Handy, OpenWhispr).
- Kein Cloud-Dienst. Rohtext und Politur bleiben lokal; nur Audio geht an die
  konfigurierte STT-API.
- Kein Verlauf. Das polierte Transkript ist ein Slot, kein Protokoll.
- Keine Word-Automation. Ein optionales Strg+V nach dem Diktat ist eine
  Einstellung (`never`/`same_window`/`always`); die Zwischenablage ist in jedem
  Fall das Ergebnis und behält den Text.

## Architektur in einem Bild

```
Fast Path (Rust, ohne Webview)          Workbench (Tauri-Webview)
Hook → Audio → Fragmente → STT              rohes Transkript
     → rohes Transkript                     poliertes Transkript
     → [ein LLM-Aufruf]                     Prompt-Auswahl (System + User)
     → Zwischenablage                       Prompt anwenden
              ↕                                     ↕
     eine SQLite-Datei (diktat, prompt), config.json,
     ein Arbeitsverzeichnis pro laufendem Diktat
```

Der Kern (`crates/core`) hat keine UI- und keine Windows-Abhängigkeit und ist
ohne laufende Anwendung testbar.

## Dokumente

| Datei | Inhalt |
|---|---|
| [CONTEXT.md](CONTEXT.md) | Vokabular — was die Wörter hier genau bedeuten |
| [docs/ai/DOMAIN.md](docs/ai/DOMAIN.md) | Tabellen, Invarianten, Abläufe, Arbeitsverzeichnis |
| [docs/ai/DECISIONS.md](docs/ai/DECISIONS.md) | getroffene Entscheidungen mit Begründung |
| [docs/ai/HANDOFF.md](docs/ai/HANDOFF.md) | **wo die nächste Sitzung anfängt** |
| [docs/ai/STATE.md](docs/ai/STATE.md) | was existiert und was nicht |
| [AGENTS.md](AGENTS.md) | Arbeitsregeln für dieses Repository |

Der Einstieg für die nächste Sitzung ist
[docs/ai/HANDOFF.md](docs/ai/HANDOFF.md): erst der Spike (#1) in der Windows-VM,
parallel Domäne (#4) und DSP-Port (#5).

## Verwandte Projekte

- [polished-recognition](https://github.com/georgernstgraf/polished-recognition)
  — Android-Tastatur; liefert Fragmentierung, Naht-Konditionierung (Q-Gate), das
  Evidenzformat und den Standard-Prompt.
- [aitranscribe](https://github.com/georgernstgraf/aitranscribe) — Terminal-App;
  liefert das Modell „ein Record, später appendable" und transkribiert die
  langen Protokolle, die hier als Text hereinkommen.

Beide werden **nicht als Code** geteilt — Spezifikation, Tests als Prüfstein und
Dateiformate schon. Die vollständige Liste steht in
[docs/ai/DECISIONS.md](docs/ai/DECISIONS.md).

## Lizenz

Dual lizenziert unter [MIT](LICENSE-MIT) **oder**
[Apache-2.0](LICENSE-APACHE), nach Wahl des Empfängers. In `Cargo.toml` wird das
später `license = "MIT OR Apache-2.0"` sein.
