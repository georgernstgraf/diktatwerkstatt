---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode/general, at: 2026-11-20T18:20:00Z }
status: stable
---

# Zustand

## Existiert

| Artefakt | Stand |
|---|---|
| `README.md` | Beschreibung, Abgrenzung, Architekturbild, Lizenz — drei Ziele, ein Kern |
| `CONTEXT.md` | Vokabular inkl. **Plattformschale**, **Fähigkeitsleiter**, **Diktierplatz**, bewusster Abgrenzungen und Zustandsübergängen |
| `docs/ai/DOMAIN.md` | zwei Tabellen, zehn Invarianten, **Fähigkeitsleiter**, Abläufe, Arbeitsverzeichnis, Lieferformate je Plattform, Fehlerfälle |
| `docs/ai/INTEGRATION.md` | Referenz: wie Handy/OpenWhispr die Bausteine lösen, mit Linux- und macOS-Spalte, mit Quellen; „was wir übernehmen" ist auf Zwischenablage, Tray und Hotkeys zusammengeschrumpft |
| `docs/ai/PITFALLS.md` | Referenz: Fallstricke aus Audio, STT, Textübergabe, Bedienung, **Windows**, **Linux/Wayland**, **Linux/X11** und **macOS**, mit Quellen; X11 und Windows gemessen, macOS und Wayland-Fragen 3/5/7 noch aus dem Bau |
| `docs/ai/DECISIONS.md` | 34 Entscheidungen, je mit verworfener Alternative — darunter die Beschlusslage E1–E10 vom 2026-11-16 |
| `docs/ai/index.md`, `docs/ai/log.md` | OKF-Bundle-Einstieg (`okf_version: "0.2"`) und Persistenz-Chronik |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung inkl. VM-Lage und Provisioning-Befund |
| `docs/ai/STATE.md` | diese Datei |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| `LICENSE-MIT`, `LICENSE-APACHE` | dual lizenziert, `MIT OR Apache-2.0` |
| Issues | Epic #3 mit offenen Sub-Issues #1, #4–#10 und #12–#18; #2, #11 und #19 geschlossen; #3 ausgegrillt |

## Existiert nicht

- Kein `Cargo.toml`, kein `src/`, kein `package.json` — **null Produktcode**.
- Keine Tests, keine CI. `cargo test` läuft ins Leere, weil es kein Crate gibt.
  Die Toolchain auf **claw** ist seit #19 bereit (2026-11-20: Rust 1.100.0 mit
  `clippy`/`rustfmt`, Tauri-Abhängigkeiten, `libasound2-dev`, `appimagetool`);
  **elite** hat seit 2026-11-21 ebenfalls `cargo` 1.100.0 mit `clippy`/`rustfmt`
  (#12, Spike-Prüfstand X11, ohne die Tauri-`-dev`-Pakete); **think** hat kein
  `cargo`.
- Keine Windows-VM auf think: qemu, libvirt, OVMF und swtpm sind installiert,
  es ist aber noch kein Gast definiert.
- Keine Linux- und keine macOS-Schale; die Linux-Schale ist der nächste
  Bauabschnitt (#12), die Fähigkeitsleiter liefert dafür nur das Gerüst in der
  Doku.

## Verifikation

Im Moment gibt es nichts zu verifizieren außer der Dokumentenkonsistenz:
`CONTEXT.md`, `DOMAIN.md` und `DECISIONS.md` müssen dieselben Begriffe und
Regeln nennen. `CONTEXT.md` ist dabei die Quelle der Begriffe, `DOMAIN.md` die
Quelle der Invarianten; wer einen Begriff ändert, ändert ihn zuerst dort.

Der erste grüne `cargo`-Commit setzt #19 (Provisioning) voraus — das ist seit
2026-11-20 erledigt (claw). Danach laufen **#4** (Domänenkern), **#5** (DSP-Port)
und **#10** (Konverter) parallel zu den Spikes — alle drei ohne Hardware und ohne
Netz, also auf Linux lauffähig. Die Spikes haben ihre eigene Reihenfolge:
#12 (Linux) vor #13 (macOS) vor #1 (Windows).

Ohne Hardware testbar bleibt der Fast Path (#8), sobald die **Fähigkeitsleiter**
existiert: ein Fake mit `{foreground_window: nein, insert_keystroke: nein}` ist
Wayland, einer mit `{ja, ja}` ist X11 oder Windows.

## Nächster Schritt

`docs/ai/HANDOFF.md`, Schritt 4: **#12 Linux-Spike**. Der **X11-Teil ist gemessen**
(elite, 2026-11-21: `XGrabKey`, `_NET_ACTIVE_WINDOW`, XTEST, Zwischenablage,
Tray, Audio). Offen sind die Wayland-Fragen 2 (Erst-Einwilligung des Portals über
einen Neustart), 3 (Injection in GTK/Writer/Brave) und 5/7 auf GNOME. #19
(Provisioning auf claw) ist erledigt; parallel zu den Spikes laufen #4, #5 und
#10.
