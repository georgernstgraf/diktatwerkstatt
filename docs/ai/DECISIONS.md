---
type: Decisions
title: Entscheidungen
description: Getroffene Architektur- und Produktentscheidungen mit Begründung.
tags: [entscheidungen, architektur]
generated: { by: opencode, at: 2026-11-15T00:00:00Z }
status: draft
---

# Entscheidungen

Neueste oben. Jeder Eintrag nennt Entscheidung, Grund und verworfene Alternative.

## 2026-11-15 — Lizenz noch offen

**Entscheidung:** keine, bewusst aufgeschoben.
**Grund:** Ein Repository ohne Lizenz ist „alle Rechte vorbehalten" und
blockiert jede Mitarbeit. Vor der ersten Veröffentlichung muss sie stehen.
**Kandidaten:** MIT (wie Handy und OpenWhispr, maximale Anschlussfähigkeit) oder
Apache-2.0 (Patentrechte, für Firmen angenehmer). AGPL scheidet aus: sie schließt
genau die Wiederverwendung aus, die hier gewünscht ist.

## 2026-11-15 — Name: diktatwerkstatt

**Entscheidung:** technischer Name `diktatwerkstatt`.
**Grund:** frei auf GitHub, crates.io und PyPI; ein Token; beschreibend; keine
Kollision mit dem übersättigten `whisper-*`-Namensfeld (und „Whisper" ist
zugleich Modellmarke von OpenAI).
**Verworfen:** `zumdiktat` (GitHub-User belegt, und „zum Diktat" ist im Deutschen
eine Allerweltswendung — nicht suchbar), `polished-dictation` (generisch,
englisch), `briefsteller` (bleibt als Markenidee in Reserve).

## 2026-11-15 — Rust-Kern + Tauri/WebView2

**Entscheidung:** Domänen- und Plattformlogik in Rust; UI als Tauri-App mit
WebView2; der Fast Path läuft vollständig ohne Webview.
**Grund:** Kompiliert, kleiner Footprint, Start unter 200 ms, und der einzige
realistische Kooperationspartner im Feld (Handy, 38 k Sterne) fährt genau diese
Kombination. TMOG, der neue Task Manager seines Originalautors, zeigt dasselbe
Muster: ein geteilter kompilierter Kern plus dünne Plattformschalen.
**Verworfen:** Kotlin/JVM (Bytecode + JIT — nicht „nativ" im hier gemeinten Sinn;
hätte den Kern von `polished-recognition` wiederverwendbar gemacht, aber
Wiederverwendung ist kein Kriterium mehr); Python (interpretiert, dazu
PortAudio-/PyInstaller-Reibung); C# (nur sinnvoll, wenn Word-Automation zentral
wird — dann später ein eigener Helfer, kein Grund für den ganzen Stack).

## 2026-11-15 — Rohtext blockweise, Politur ganzheitlich

**Entscheidung:** Rohtext wächst in Blöcken; die Politur ist genau eine Fläche
über dem ganzen Diktat.
**Grund:** Zwei verschiedene Granularitäten für zwei verschiedene Dinge. Blöcke
machen Append und Korrektur nachvollziehbar; ein einziger Politur-Slot hält die
Anzeige einfach und entspricht dem, was tatsächlich gebraucht wird.

## 2026-11-15 — Politur wird verworfen, nicht markiert

**Entscheidung:** Ändert sich der Rohtext, werden `polished_*` auf `NULL`
gesetzt.
**Grund:** Ein als „veraltet" markiertes Ergebnis bleibt sichtbar und wird
irgendwann kopiert. Bei Sekretariatsarbeit ist ein Zustand, in dem die App etwas
Falsches als richtig anzeigt, schlimmer als ein verlorenes Ergebnis.
**Verworfen:** `raw_fingerprint` plus Veraltet-Kennzeichnung (mehr Zustand, mehr
Fehlerquellen, kein Gewinn).

## 2026-11-15 — Kein Run-Verlauf

**Entscheidung:** keine Tabelle `run`, keine Prompt-Versionierung, kein
Vergleich zweier Ergebnisse.
**Grund:** Der Nutzer braucht 3–5 Diktate und jeweils das letzte Ergebnis. Der
Verlauf ist Ballast, und bei einem Ring von 7 Einträgen wäre er ohnehin kurz.
**Verworfen:** `runs`-Tabelle mit Snapshots (war der erste Entwurf; verworfen,
weil sie eine Funktion ohne Nutzerbedarf trägt).

## 2026-11-15 — Play fügt nicht ein

**Entscheidung:** Play schreibt die Politur in die Workbench und in die
Zwischenablage. Es tippt nichts in die Zielanwendung.
**Grund:** Ein zweiter Play bei bereits eingefügtem Rohtext erzeugt sonst
doppelten Text in einem fremden Dokument. Die Übergabe an die Zielanwendung
bleibt eine bewusste Handlung.
**Verworfen:** automatisches Einfügen nach dem Play.

## 2026-11-15 — Ring von 5–7, mit `pinned`

**Entscheidung:** Standard 7 Einträge, konfigurierbar 5–7, kein Archiv. Ein
Eintrag kann `pinned` sein und überlebt den Ring.
**Grund:** Konsequent klein halten. Die Ausnahme kostet eine Spalte und verhindert
den Fall „das war der wichtige Brief", ohne ein Archiv zu öffnen.

## 2026-11-15 — Höchstens ein offenes Diktat

**Entscheidung:** Nur das jüngste Diktat ist offen; geschlossene sind
eingefroren.
**Grund:** Sonst braucht es „wieder öffnen", einen zweiten Bearbeitungszustand
und Regeln für Append in die Vergangenheit.
**Verworfen:** mehrere gleichzeitig offene Diktate.

## 2026-11-15 — Tray als Bedienoberfläche

**Entscheidung:** Zustandsanzeige im Tray (IDLE/RECORDING/PAUSED/TRANSCRIBING/
POLISHING/READY/POLISHED/FEHLER), zusätzlich eine kleine Overlay-Pille, weil
Windows 11 neue Tray-Symbole standardmäßig im Überlauf versteckt.
**Grund:** Der Nutzer soll diktieren können, ohne die App anzusehen; das Tray
allein ist dafür nicht verlässlich sichtbar genug.

## 2026-11-15 — Keine geteilte Codebasis mit polished-recognition

**Entscheidung:** Der Rust-Kern ist eine Neuimplementierung; geteilt werden
Spezifikation und Dateiformat, nicht Code.
**Grund:** `polished-recognition` bleibt die Android-Tastatur in Kotlin. Ein
gemeinsamer Kern über Android und Desktop hieße JNI oder Rust in der App — dafür
gibt es keinen Bedarf.
**Was überlebt:** Vokabular, Invarianten, Tests als Prüfstein, und das
`session.json`-Format, das der dortige Kotlin-`:analyzer` weiterhin lesen kann.
