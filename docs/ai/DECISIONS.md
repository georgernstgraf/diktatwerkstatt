---
type: Decisions
title: Entscheidungen
description: Getroffene Architektur- und Produktentscheidungen mit Begründung.
tags: [entscheidungen, architektur]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T05:04:02Z }
status: stable
---

# Entscheidungen

Neueste oben. Jeder Eintrag nennt Entscheidung, Grund und verworfene Alternative.

## 2026-11-16 — Korrigieren heißt Hören reparieren

**Entscheidung:** Das polierende Modell repariert die Transkription, nicht die
Aussage. Dialekt, Umgangssprache und Satzbau des Sprechers bleiben; korrigiert
werden Erkennungsfehler. Namen, Zahlen, Daten und Beträge werden repariert, wo
die Lesart sicher ist, und sonst gelassen — nie erfunden, nie stillschweigend
„verbessert". Sichtbare Marker wie `[? Grund]` gibt es in v0.1 **nicht**.
**Grund:** Übernommen aus dem `protokoll`-Skill der `opencode-helpers`
(„Repair the hearing, not the speech", „Flag, don't fix" für die tragenden
Fakten). Ein falscher Nachname ist ein Wort von hundert — 1 % WER und trotzdem
ein Kanzleifehler; der WER misst hier das Falsche. Marker im Text scheiden aus,
weil das polierte Transkript in einen Brief wandert und `[? unklar]` dort eine
Peinlichkeit ist.
**Verworfen:** das Modell als Lektor, der glättet und umformuliert (verändert
Aussagen, die niemand ändern wollte); sichtbare Flag-Marker im polierten
Transkript (im Brief unbrauchbar); Flag-Marker als **spätere** Prompt-Option für
Protokolle (bleibt möglich, gehört aber nicht in v0.1).

## 2026-11-16 — Lizenz: dual MIT OR Apache-2.0

**Entscheidung:** Das Projekt steht unter `MIT OR Apache-2.0`. Zwei
Lizenzdateien (`LICENSE-MIT`, `LICENSE-APACHE`), in `Cargo.toml` später
`license = "MIT OR Apache-2.0"`.
**Grund:** Das ist die Konvention des Rust-Ökosystems und löst den Konflikt, der
den Eintrag vom 2026-11-15 offen hielt: MIT für maximale Anschlussfähigkeit,
Apache-2.0 für den Patentabsatz, den Firmen erwarten — der Empfänger wählt.
**Verworfen:** nur MIT (kein Patentabsatz; unattraktiv für eine mitentwickelnde
Firma), nur Apache-2.0 (NOTICE-Pflicht, eine Hürde mehr für private
Beitragende), AGPL (schließt genau die Wiederverwendung aus, die gewünscht ist).

## 2026-11-16 — FIFO statt Ring

**Entscheidung:** Die letzten N Diktate bilden eine Warteschlange. N steht in
`config.json`. Kommt eines hinzu, fällt das älteste heraus, samt
Arbeitsverzeichnis. Es gibt kein `pinned`.
**Grund:** „Ring" behauptete eine kreisförmige Struktur, die es nie gab, und
`pinned` machte aus einer Warteschlange eine Regel mit Ausnahme: „ältestes
*nicht angeheftetes*". Diese Ausnahme hat ein Loch — sind alle N angeheftet,
wächst der Bestand unbegrenzt, und die Invariante fällt lautlos.
**Verworfen:** Ring mit `pinned` (Ausnahme ohne Obergrenze), feste Größe ohne
Einstellung (der Bedarf schwankt zwischen Briefserie und Protokollwoche).

## 2026-11-16 — Kein Zustandsfeld: das Arbeitsverzeichnis ist der Zustand

**Entscheidung:** Ein Diktat hat kein `status`, kein `closed_at`, keine
Unveränderlichkeit. Ob es offen ist, sagt allein `audio_dir`: existiert das
Verzeichnis, ist die Transkription nicht abgeschlossen. Enter beendet die
Aufnahme, nicht das Diktat — angehängt werden darf jederzeit, auch Tage später.
**Grund:** „Geschlossen" bedeutete „eingefroren" — und das wurde verneint, weil
`aitranscribe` das Anhängen später erlaubt und weil ein Vormittag mehrere
Diktate hat. Damit hätte ein Statusfeld zwei Namen für einen Zustand geführt,
der kein Verhalten ändert. Ein Feld, das man setzen kann, ohne dass sich etwas
ändert, ist genau das, was der Eintrag „Politur wird verworfen, nicht markiert"
ausschließt.
**Verworfen:** `status` + `closed_at` (Zustand ohne Verhalten);
Unveränderlichkeit geschlossener Diktate (verhindert das Nachtragen eines
Satzes); Schließen per Zeitablauf (ein Timer, der schließt, während man
nachdenkt); mehrere gleichzeitig offene Diktate (braucht „wieder öffnen").

## 2026-11-16 — Rohes Transkript als ein Text, keine Blöcke

**Entscheidung:** `diktat.raw_text` ist ein einziger Text. Es gibt keine Tabelle
`chunk`, kein `seq`, keine Blockgrenzen in der Datenbank.
**Grund:** Der Rohtext wurde bisher als Verkettung von Blöcken definiert und
„nicht redundant" gespeichert — was bedeutete, dass es die Spalte gar nicht
gab. Mit einem Text ist Invariante 4 trivial wahr statt dauernd zu beweisen.
`seq` war ein zweiter Sortierschlüssel neben dem ohnehin monotonen `id`, und
„lückenlos 1-basiert" kostete beim Löschen eines Blocks ein Umnummerieren.
**Verworfen:** Tabelle `chunk` mit `seq` (zweite Wahrheit, Umnummerierung,
Blockgrenzen als Anzeige, die niemand angefordert hat); `raw_text` zusätzlich
zu `chunk` (Redundanz, die auseinanderlaufen kann).

## 2026-11-16 — Text vor Audio

**Entscheidung:** Ein transkribiertes Fragment steht auf der Platte, **bevor**
seine Audiodatei gelöscht wird. Auf Diktatebene gilt dasselbe: `raw_text` ist
committet, bevor das Arbeitsverzeichnis verschwindet.
**Grund:** Zwei Dateisystem-Operationen sind nie atomar. Löscht man zuerst das
Audio und stürzt dazwischen ab, sind Audio und Text weg — im Protokoll fehlt
still ein Satz, und niemand merkt es. In der anderen Reihenfolge bleiben beide
liegen, und die Wiederaufnahme räumt die überflüssige `.wav` weg. Geschrieben
wird über `.tmp` + `rename`, damit eine halb geschriebene `.txt` nicht als
erledigt gilt.
**Verworfen:** „Audio zuerst" (stiller Datenverlust); Zustand in der Datenbank
statt im Verzeichnis (hätte bei jedem Absturz eine Reparatur gebraucht);
direktes Schreiben ohne `rename` (abgeschnittene Dateien sehen fertig aus).

## 2026-11-16 — Zwei Prompt-Auswahlen aus einer Tabelle

**Entscheidung:** Die Tabelle `prompt` hält Zeilen mit `is_system`. Der Nutzer
wählt genau einen System-Prompt (Rolle, Ausgabeformat) und genau einen
User-Prompt (die Aufgabe); beide gehen in den Aufruf. Keine Verknüpfung, keine
Fremdschlüsselspalte. Die Zielsprachen-Klausel steht im `user`-Text, nicht in
einem eigenen Feld.
**Grund:** Die Chat-API will genau eine System- und eine User-Nachricht. Zwei
unabhängige Auswahlen bilden das ohne Zwischentabelle ab. `zielsprache` und
`ton` waren von Anfang an „optional, der Alias genügt" — eine Klausel im Text
leistet dasselbe ohne Spalte.
**Verworfen:** `prompt` mit `system` und `user` als zwei Spalten einer Zeile
(hätte den Nutzer gezwungen, Paare zu pflegen, die er nicht will);
Fremdschlüssel User-Prompt → System-Prompt (Verknüpfung ohne Anwendungsfall);
`zielsprache`/`ton`/`default_model`/`archiviert` (Spekulation bzw. Zustand, den
bei acht Prompts niemand braucht).

## 2026-11-16 — Politur pro Diktat, über zwei Hotkeys

**Entscheidung:** Es gibt „diktieren roh" und „diktieren poliert". Der zweite
wendet die aktuelle Auswahl automatisch an. **Prompt anwenden** bleibt für die
Wiederholung mit einer anderen Auswahl.
**Grund:** In `polished` ist das Keyboard immer im Blick; dort genügt eine
sichtbare Auswahl. Auf dem Desktop ist das Fenster im Normalfall zu — ein
Schalter, den man erst öffnen muss, ist kein Schalter. Zwei Hotkeys sind das
Gegenstück zur Send-Taste: eine Handlung, kein Zustand, an den man sich erinnern
muss.
**Verworfen:** Politur-Schalter im Fenster (nicht erreichbar, wenn das Fenster
zu ist); globaler Schalter (man muss vor dem Sprechen an etwas denken); dritter
Hotkey zum Umschalten (zwei Schalter für dieselbe Sache).

## 2026-11-16 — Ein Aufruf, kein Abschneiden

**Entscheidung:** Das ganze rohe Transkript geht in **einem** Aufruf an das
LLM. Passt es nicht in das Modell (Kontext oder Ausgabelänge), erscheint ein
Fehler; es wird nie gekürzt und nie in Abschnitte zerlegt.
**Grund:** Ein 90-Minuten-Protokoll hat ~20.000 Token. Modelle wie Luna oder
Flash tragen das; ein kleines Modell trägt es nicht — und dann ist eine
Fehlermeldung billiger und ehrlicher als eine abschnittsweise Politur, deren
Nähte man nachträglich wieder zusammenkleben müsste. Das Nahtproblem der
Audioseite ein zweites Mal auf der Textseite zu lösen, wäre der teuerste Teil
des Projekts.
**Verworfen:** Chunked Politur mit unsichtbaren Nähten (neues Nahtproblem,
Overlap, Dedupe, Stilbruch zwischen Abschnitten); stilles Kürzen (der Nutzer
merkt es erst am fehlenden Absatz).

## 2026-11-16 — Zwischenablage genau einmal, atomar

**Entscheidung:** Die Zwischenablage wird pro Diktat **einmal** geschrieben, im
selben Zug, in dem der Text im Fenster erscheint. Bei „diktieren roh" ist das der
Rohtext, bei „diktieren poliert" das polierte Transkript — vorher bleibt die
Zwischenablage unberührt. Scheitert die Politur, geht der Rohtext als Rückfall
hinein.
**Grund:** Würde der Rohtext sofort in die Zwischenablage gehen und die Politur
ihn später überschreiben, gäbe es einen Wettlauf: wer früh einfügt, hat roh,
wer spät einfügt, poliert — und im Dokument steht etwas anderes als in der
Zwischenablage. Atomar gibt es diesen Zustand nicht. Der Rückfall im Fehlerfall
ist die einzige Stelle, an der die App sonst fertigen Text vernichten würde.
**Verworfen:** Rohtext sofort, Politur überschreibt später (Wettlauf, zwei
Wahrheiten); nur im Fenster, Zwischenablage auf Knopfdruck (der Nutzer hat den
Text, aber nicht dort, wo er ihn braucht); Rohtext im Fehlerfall verwerfen
(vernichtet bezahlte Arbeit).

## 2026-11-16 — Semantisches HTML auf der Zwischenablage, constrained Markdown als Format

**Entscheidung:** Auf die Zwischenablage gehen `CF_HTML` (semantische Tags,
kein CSS, keine Klassen) und `CF_UNICODETEXT`. Das polierte Transkript darf nur
Absatz, Überschrift, ungeordnete Liste und fett enthalten; geordnete Listen
werden als Text mit ihren Ziffern ausgegeben. Vor dem Schreiben wird validiert.
**Grund:** Reiner Text transportiert keine Struktur, und Words nativer
„Paste from Markdown" ist undokumentiert und je nach Variante nicht vorhanden
(Mac: gar nicht, Windows: Insider). HTML versteht Word seit jeher und bildet
`h1`–`h6` auf seine Formatvorlagen ab — damit trägt der Prompt die Struktur und
die Vorlage das Aussehen, ohne Word-Kopplung. Die Einschränkung auf vier
Konstrukte, weil Markdown gegen juristischen Text nicht neutral ist:
`1)` wird zur Liste (CommonMark erlaubt `.` und `)`), `5*6*78` wird kursiv,
`\§` bleibt als Backslash stehen, und Words alte AutoFormat-Option macht aus
`*fett*` **fett** statt kursiv.
**Verworfen:** Markdown roh in die Zwischenablage (Word nimmt das HTML und sieht
es nie; textonly-Ziele zeigen Sternchen); volles CommonMark (Kollisionen mit
Rubrum-Nummerierung); Word-COM (nur installiertes Word, nicht Google Docs oder
Outlook Web); docx-Datei (Dateidialog im Alltag).
**Offenes Risiko:** Seit Word-Build 2405 ist „Merge Formatting" der Default für
Einfügen aus anderen Programmen; Microsofts Beschreibung nennt Fett und Listen,
aber **nicht** Überschriften. Ob Überschriften verflachen, ist nur auf echtem
Windows messbar und gehört zu den Abnahmekriterien von #1.

## 2026-11-16 — Kein Audio-Import; Diktat aus der Zwischenablage

**Entscheidung:** Die App nimmt Audio nur über das Mikrofon auf. Ein langes
Transkript (Besprechungsprotokoll) kommt über ein neues Diktat aus der
Zwischenablage herein. Woher der Text stammt, wird nicht gespeichert.
**Grund:** Ein Datei-Eingang ist ein zweiter Ablauf ohne Hotkey, ohne
Tray-Zustand und ohne Fragmentierung — und er zieht die Frage nach der
Abschnittspolitur sofort in v0.1. `aitranscribe` deckt den Fall heute ab; ein
Kopieren ist billiger als ein zweiter Eingang.
**Verworfen:** Audio-Import in v0.1 (verdoppelt den Eingang); Spalte `quelle`
(der Nutzer braucht sie nicht — sie wurde eingeführt und wieder gestrichen).

## 2026-11-16 — Keine Herkunftsangabe am polierten Transkript

**Entscheidung:** Es wird nicht gespeichert, welcher Prompt und welches Modell
das polierte Transkript erzeugt haben — weder als Fremdschlüssel noch als
Schnappschuss.
**Grund:** Die Anzeige „erzeugt von Prompt X, Modell Y" war eine Anforderung
ohne Nutzer. Ein Fremdschlüssel hätte zusätzlich gelogen, sobald ein Prompt
umbenannt wird: die Herkunft hätte sich rückwirkend geändert.
**Verworfen:** `polished_prompt_id` (Lüge nach Umbenennung, Fremdschlüssel auf
löschbare Zeilen); Alias-Schnappschuss (wahr, aber ohne Abnehmer);
`polished_model` (dito); `polished_at` (eine Spalte, die niemand liest).

## 2026-11-16 — Einstellungen in `config.json`, nicht in der Datenbank

**Entscheidung:** FIFO-Größe, Hotkeys, Endpunkte, Modellnamen und die
Prompt-Auswahl stehen in `config.json` neben der Datenbank.
**Grund:** Die Datenbank enthält Mandantendaten. „Daten löschen" darf die
API-Konfiguration nicht mitnehmen, und eine Einstellungsdatei soll man von Hand
lesen und sichern können. Eine `setting`-Tabelle wäre außerdem eine dritte
Tabelle für Daten, die nicht in die Datenbank gehören.
**Verworfen:** `setting`-Tabelle (vermischt Konfiguration mit Mandantendaten);
Einstellungen im Windows-Register (nicht portierbar, nicht lesbar).

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
**Verworfen:** Kotlin/JVM (Bytecode + JIT — nicht „nativ" im hier gemeinten Sinn);
Python (interpretiert, dazu PortAudio-/PyInstaller-Reibung); C# (nur sinnvoll,
wenn Word-Automation zentral wird — dann später ein eigener Helfer).

## 2026-11-15 — Poliertes Transkript wird verworfen, nicht markiert

**Entscheidung:** Ändert sich das rohe Transkript, wird `polished_text` auf
`NULL` gesetzt. Es folgt keine automatische Neupolitur.
**Grund:** Ein als „veraltet" markiertes Ergebnis bleibt sichtbar und wird
irgendwann kopiert. Bei Sekretariatsarbeit ist ein Zustand, in dem die App etwas
Falsches als richtig anzeigt, schlimmer als ein verlorenes Ergebnis. Die
Neupolitur bleibt eine bewusste Handlung, weil sie Geld kostet — bei einem
90-Minuten-Protokoll viel Geld.
**Verworfen:** `raw_fingerprint` plus Veraltet-Kennzeichnung (mehr Zustand, mehr
Fehlerquellen); automatische Neupolitur nach jedem Append (ein API-Aufruf ohne
Zutun, ausgelöst durch eine Tipp-Korrektur).

## 2026-11-15 — Kein Run-Verlauf

**Entscheidung:** keine Tabelle `run`, keine Prompt-Versionierung, kein
Vergleich zweier Ergebnisse.
**Grund:** Der Nutzer braucht 3–5 Diktate und jeweils das letzte Ergebnis. Der
Verlauf ist Ballast, und bei einem FIFO von 7 Einträgen wäre er ohnehin kurz.
**Verworfen:** `runs`-Tabelle mit Snapshots (war der erste Entwurf; verworfen,
weil sie eine Funktion ohne Nutzerbedarf trägt).

## 2026-11-15 — Prompt anwenden fügt nicht ein

**Entscheidung:** **Prompt anwenden** schreibt das polierte Transkript in die
Workbench und in die Zwischenablage. Es tippt nichts in die Zielanwendung.
**Grund:** Ein zweiter Lauf bei bereits eingefügtem Rohtext erzeugt sonst
doppelten Text in einem fremden Dokument. Die Übergabe an die Zielanwendung
bleibt eine bewusste Handlung.
**Verworfen:** automatisches Einfügen nach dem Polieren.

## 2026-11-15 — Tray als Zustandsanzeige

**Entscheidung:** Zustandsanzeige im Tray (`IDLE`, `RECORDING`, `PAUSED`,
`TRANSCRIBING`, `POLISHING`, `READY`, `POLISHED`, `FEHLER`), mit Blinken während
der Aufnahme und **einem** Fehlerzustand. Zusätzlich eine kleine Overlay-Pille,
weil Windows 11 neue Tray-Symbole standardmäßig im Überlauf versteckt.
**Grund:** Der Nutzer soll diktieren können, ohne die App anzusehen; das Tray
allein ist dafür nicht verlässlich sichtbar genug. Zwei verschiedene Rottöne für
STT- und LLM-Fehler unterscheidet niemand im Augenwinkel — die Ursache steht im
Fenster.
**Verworfen:** Tray als Diagnosewerkzeug (zeigt an, was die App tut, statt was
der Nutzer tun soll); Fortschrittsanzeige im Tray (dafür ist das Fenster da).

## 2026-11-15 — Keine geteilte Codebasis mit polished-recognition

**Entscheidung:** Der Rust-Kern ist eine Neuimplementierung; geteilt werden
Spezifikation, Tests als Prüfstein, das Dateiformat — und seit 2026-11-16 auch
der Prompt-Text (ergänzt).
**Grund:** `polished-recognition` bleibt die Android-Tastatur in Kotlin. Ein
gemeinsamer Kern über Android und Desktop hieße JNI oder Rust in der App — dafür
gibt es keinen Bedarf.

Was konkret geborgt wird und was nicht:

| Was | Woher | Wie |
|---|---|---|
| Vokabular (Fragment, Naht, Q, Q-Gate, Pre-roll, Echo-Trim) | `polished-recognition` | übernommen, hier weitergeführt |
| Algorithmen (Fragmentierung, Naht, Stillemaß) | `polished-recognition` (Kotlin), `aitranscribe` (Python, 33 Tests) | nach Rust **portiert**, Tests als Prüfstein (#5) |
| Prompt-Text (`prompts.json`) | `polished-recognition` | wörtlich als Standard-Prompt |
| Bedienmuster (Send-Taste, Prompt-Auswahl, mitlaufende Fragmente) | `polished-recognition` (IME) | auf den Desktop übertragen: zwei Hotkeys, zwei Auswahlfelder |
| Datenmodell „ein Record in SQLite, später appendable" | `aitranscribe` | übernommen, ohne Blöcke |
| Evidenzformat `session.json` | `polished-recognition` | bleibt lesbar für den dortigen Kotlin-`analyzer` |
| Code | — | keiner |

**Verworfen:** auf `polished` aufbauen (Android-Tastatur ohne Tray, Hook, Fenster
und FIFO); einen gemeinsamen Rust-Kern in die Android-App ziehen (JNI ohne
Nutzen).
