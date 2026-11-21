---
okf_version: "0.2"
---

# Wissensbasis

Einstiegspunkt des OKF-Bundles `docs/ai/`. Kurze Zeiger; Details stehen in den
jeweiligen Konzeptdateien. Der Einstieg für die Arbeit ist
[HANDOFF.md](HANDOFF.md); [CONTEXT.md](../../CONTEXT.md) liegt außerhalb des
Bundles, weil es das Glossar für das ganze Repository ist.

## Aktive Dateien

* [HANDOFF.md](HANDOFF.md) - Wo die nächste Sitzung anfängt: Domänenkern aus #4 und DSP-Port aus #5 stehen, #10 Konverter ist der nächste Bauabschnitt, #12 Linux-Spike in Arbeit, bekannte Fallen
* [STATE.md](STATE.md) - Was existiert (`crates/core` mit Schema, Arbeitsverzeichnis, DSP-Kern und 88 Tests), was nicht, und wie man es prüft
* [DOMAIN.md](DOMAIN.md) - Zwei Tabellen, zehn Invarianten, Audio (Slice, Q, Naht), Fähigkeitsleiter mit gemessener Matrix Wayland/X11, Abläufe, Lieferformate je Plattform, Fehlerfälle
* [DECISIONS.md](DECISIONS.md) - 36 Architektur- und Produktentscheidungen (darunter E1–E10: drei Ziele, kein Tastenhook, Fähigkeitsleiter), je mit verworfener Alternative
* [PITFALLS.md](PITFALLS.md) - Hart erarbeitete Fallstricke aus Audio, STT, Textübergabe, Bedienung, Windows, Linux/Wayland, Linux/X11 und macOS — mit Belegen für Windows (Word-Paste, Hotkeys, Tray, Sitzungsgrenzen), Linux/X11 (Hotkeys, Injection, Zwischenablage, Tray, Audio gemessen) und GNOME 50 (App-ID-Gate); macOS sind Warnungen aus dem Bau, ihre Belege liefert #13
* [INTEGRATION.md](INTEGRATION.md) - Wie Handy und OpenWhispr Hotkey, Einfügen, Tray und Installer lösen — mit Linux- und macOS-Spalte und was wir übernehmen

## Glossar und Einstieg außerhalb des Bundles

* [../../CONTEXT.md](../../CONTEXT.md) - Vokabular: was die Wörter hier genau bedeuten, inkl. Plattformschale, Fähigkeitsleiter, Diktierplatz
* [../../README.md](../../README.md) - Beschreibung, Architekturbild, Lizenz
* [../../AGENTS.md](../../AGENTS.md) - Arbeitsregeln, Stack, Verifikation

## Referenz

* [log.md](log.md) - Persistenz-Chronik dieses Bundles
