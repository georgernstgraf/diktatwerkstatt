# Diktatwerkstatt

Ein Diktier-Werkzeug für Menschen, die professionelle Texte schreiben, ohne zu
tippen: Sekretariate, Kanzleien, Assistenz.

**Status: Spezifikation.** Es gibt noch keinen Code. Dieses Repository hält
Vokabular, Domänenmodell und Entscheidungen fest, bevor die erste Zeile
geschrieben wird — nach dem Muster, mit dem TMOG (der neue Task Manager seines
Originalautors) aus einer 107-seitigen Spezifikation entstanden ist.

## Was es ist

- Ein **Windows-Hilfsprogramm im Tray**, das Diktate in jede Anwendung bringt
  (Word, Google Docs, Notepad) — über die Zwischenablage, ohne Fenster im Weg.
- Eine **Workbench** mit zwei Reitern: Rohtext links, poliertes Ergebnis
  rechts, Prompt-Bibliothek darunter, Play-Taste zum Anwenden.
- Ein **Ring der letzten 5–7 Diktate** — kein Archiv, keine Datenbankpflege.

## Was es nicht ist

- Keine weitere generische Diktier-App (die gibt es: Handy, OpenWhispr).
- Kein Cloud-Dienst. Rohtext und Politur bleiben lokal; nur Audio geht an die
  konfigurierte STT-API.
- Kein Verlauf. Die Politur ist ein Slot, kein Protokoll.

## Architektur in einem Bild

```
Fast Path (Rust, ohne Webview)              Workbench (Tauri-Webview)
Hook → Audio → Fragmentierung → STT            Roh | gerendert
     → Clipboard → Tray-Zustand                Prompt-Liste, Play
              ↕                                     ↕
        eine SQLite-Datei, drei Tabellen
```

Der Kern (`crates/core`) hat keine UI- und keine Windows-Abhängigkeit und ist
ohne laufende Anwendung testbar.

## Dokumente

| Datei | Inhalt |
|---|---|
| [CONTEXT.md](CONTEXT.md) | Vokabular — was die Wörter hier genau bedeuten |
| [docs/ai/DOMAIN.md](docs/ai/DOMAIN.md) | Tabellen, Invarianten, Zustandsmaschine |
| [docs/ai/DECISIONS.md](docs/ai/DECISIONS.md) | getroffene Entscheidungen mit Begründung |
| [docs/ai/HANDOFF.md](docs/ai/HANDOFF.md) | **wo die nächste Sitzung anfängt** |
| [docs/ai/STATE.md](docs/ai/STATE.md) | was existiert und was nicht |
| [AGENTS.md](AGENTS.md) | Arbeitsregeln für dieses Repository |

Der Einstieg für die nächste Sitzung ist
[docs/ai/HANDOFF.md](docs/ai/HANDOFF.md): erst Epic #3 ausgrillen, dann Spike #1.

## Verwandte Projekte

- [polished-recognition](https://github.com/georgernstgraf/polished-recognition)
  — Android-Tastatur; liefert Fragmentierung, Naht-Konditionierung (Q-Gate) und
  das Evidenzformat, das hier als Spezifikation wiederverwendet wird.
- [aitranscribe](https://github.com/georgernstgraf/aitranscribe) — Terminal-App;
  liefert das Modell „Rohtext bleibt Basis, letztes Ergebnis wird überschrieben".

## Lizenz

Noch nicht festgelegt — siehe Issue. Bis dahin gilt alleiniges Urheberrecht.
