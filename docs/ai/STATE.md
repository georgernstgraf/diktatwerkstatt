---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode/general, at: 2026-11-20T16:36:52Z }
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
| `docs/ai/PITFALLS.md` | Referenz: Fallstricke aus Audio, STT, Textübergabe, Bedienung, **Linux/Wayland** und **macOS**, mit Quellen (Linux/macOS: Warnungen aus dem Bau, Belege folgen mit #12/#13) |
| `docs/ai/DECISIONS.md` | 34 Entscheidungen, je mit verworfener Alternative — darunter die Beschlusslage E1–E10 vom 2026-11-16 |
| `docs/ai/index.md`, `docs/ai/log.md` | OKF-Bundle-Einstieg (`okf_version: "0.2"`) und Persistenz-Chronik |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung inkl. VM-Lage und Provisioning-Befund |
| `docs/ai/STATE.md` | diese Datei |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| `LICENSE-MIT`, `LICENSE-APACHE` | dual lizenziert, `MIT OR Apache-2.0` |
| Issues | Epic #3 mit offenen Sub-Issues #1, #4–#10 und #12–#19; #2 und #11 geschlossen; #3 ausgegrillt |

## Existiert nicht

- Kein `Cargo.toml`, kein `src/`, kein `package.json` — **null Produktcode**.
- Keine Tests, keine CI. `cargo test` läuft ins Leere, weil es kein Crate gibt
  und auf einem frischen Linux-Host auch kein `cargo` (Befund auf claw,
  2026-11-16; #19 richtet Toolchain und Build-Abhängigkeiten ein).
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

Der erste grüne `cargo`-Commit setzt #19 (Provisioning) voraus. Danach kommen
**#4** (Domänenkern), **#10** (Konverter) und **#5** (DSP-Port) — alle drei ohne
Hardware und ohne Netz, also auf Linux lauffähig. Die Spikes folgen: #12 (Linux)
vor #13 (macOS) vor #1 (Windows).

Ohne Hardware testbar bleibt der Fast Path (#8), sobald die **Fähigkeitsleiter**
existiert: ein Fake mit `{foreground_window: nein, insert_keystroke: nein}` ist
Wayland, einer mit `{ja, ja}` ist X11 oder Windows.

## Nächster Schritt

`docs/ai/HANDOFF.md`, Schritt 3: **#19 Provisioning** — rustup, `libwebkit2gtk-4.1-dev`,
`libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `patchelf`,
`libfuse2`, `appimagetool` auf claw und dell. Danach #12 (Linux-Schale), parallel
#4, #5 und #10.
