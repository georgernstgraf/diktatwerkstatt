# Diktatwerkstatt

Ein Diktier-Werkzeug für Menschen, die professionelle Texte schreiben, ohne zu
tippen: Sekretariate, Kanzleien, Assistenz.

**Status: Kern in Arbeit.** `crates/core` hält seit 2026-11-21 die Domäne
(Aufgabenstellung #4): zwei SQLite-Tabellen, das Arbeitsverzeichnis mit
`NNN.wav`/`NNN.txt`, den FIFO und die zehn Invarianten, jede mit einem Test.
Vokabular, Domänenmodell und Entscheidungen stehen davor fest — nach dem Muster,
mit dem TMOG (der neue Task Manager seines Originalautors) aus einer 107-seitigen
Spezifikation entstanden ist.

Beschlossen ist der Rahmen: **drei Ziele** (Windows, Linux, macOS), **ein
geteilter Kern**, **drei dünne Plattformschalen**, **kein globaler Tastenhook**
und **keine Plattformweiche im Kern** — der Kern fragt Fähigkeiten ab. Gebaut
wird zuerst auf Linux; die Reihenfolge der Auslieferung ist offen. Die
Entscheidungen stehen in [docs/ai/DECISIONS.md](docs/ai/DECISIONS.md), die
Begriffe in [CONTEXT.md](CONTEXT.md).

Das Epic ist **ausgegrillt**: die teuren Annahmen sind geprüft, zwei davon
gekippt (Word-Formatierung, Whisper-Qualität auf Spontansprache). Ergebnis im
Kommentar an [#3](https://github.com/georgernstgraf/diktatwerkstatt/issues/3).

## Was es ist

- Ein **Diktier-Werkzeug im Tray für drei Ziele** — Windows, Linux, macOS — mit
  einem geteilten Rust-Kern und drei dünnen **Plattformschalen**. Es bringt
  Diktate in jede Anwendung (Word, Google Docs, Notepad) — über die
  Systemzwischenablage, ohne Fenster im Weg.
- Gebaut wird zuerst auf **Linux** (erste Schale, weil dort entwickelt wird);
  Windows bleibt das Produkt. Die Reihenfolge der Auslieferung ist offen, die
  Architektur nicht.
- Eine **Workbench** mit zwei Reitern: rohes Transkript links, poliertes
  rechts, Prompt-Auswahl darunter, „Prompt anwenden".
- Ein **FIFO der letzten Diktate** — kein Archiv, keine Datenbankpflege.

## Was es nicht ist

- Keine weitere generische Diktier-App (die gibt es: Handy, OpenWhispr).
- Kein Cloud-Dienst. Rohtext und Politur bleiben lokal; nur Audio geht an die
  konfigurierte STT-API.
- Kein Verlauf. Das polierte Transkript ist ein Slot, kein Protokoll.
- Keine Word-Automation. Ein optionales Strg+V (Cmd+V) nach dem Diktat ist eine
  Einstellung (`never`/`same_window`/`always`); die Zwischenablage ist in jedem
  Fall das Ergebnis und behält den Text. In das eigene Fenster wird nie
  eingefügt.
- **Kein globaler Tastenhook.** Drei globale Hotkeys starten, beenden und
  pausieren; Space und Enter gelten nur im fokussierten Workbench-Fenster.

## Architektur in einem Bild

```
Kern (Rust, ohne Webview)               Workbench (Tauri-Webview)
Hotkey → Audio → Fragmente → STT            rohes Transkript
     → rohes Transkript                     poliertes Transkript
     → [ein LLM-Aufruf]                     Prompt-Auswahl (System + User)
     → Zwischenablage                       Prompt anwenden
              ↕                                     ↕
     eine SQLite-Datei (diktat, prompt), config.json,
     ein Arbeitsverzeichnis pro laufendem Diktat
              ↑
     Fähigkeitsleiter der Plattformschale
     (Windows · Linux · macOS — ein Kern, drei Schalen)
```

Der Kern (`crates/core`) hat keine UI- und **keine Plattform-Abhängigkeit**: kein
`cfg(target_os)`, kein `enum Platform`. Er ist ohne laufende Anwendung testbar,
auf jedem Ziel. Was eine Plattform kann, meldet ihre Schale beim Start als
**Fähigkeitsleiter**; was fehlt, fällt auf die sichere Seite.

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
[docs/ai/HANDOFF.md](docs/ai/HANDOFF.md): Domänenkern (#4), DSP-Port (#5) und
Konverter (#10) stehen, als Nächstes kommt der STT-/LLM-Client (#6); parallel
laufen die Spikes — Linux (#12), macOS (#13), Windows (#1).

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
[Apache-2.0](LICENSE-APACHE), nach Wahl des Empfängers. In `Cargo.toml` steht das
als `license = "MIT OR Apache-2.0"`.
