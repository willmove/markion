<a id="german"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion-Logo" width="128" height="128">
</p>

<p align="center">
  <a href="README.md#english">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.fr.md">Français</a> · <strong>Deutsch</strong> · <a href="README.es.md">Español</a>
</p>

# Markion

Markion ist ein nativer Desktop-Markdown-Editor, gebaut mit Rust und GPUI. Er vereint reaktionsschnelle Quelltextbearbeitung, einen quellgestützten visuellen Bearbeitungsmodus, Live-Vorschau, Arbeitsbereichswerkzeuge und Mehrformat-Export in einer leichtgewichtigen Anwendung. Markdown bleibt das kanonische Dokumentformat — kein Electron, Tauri oder WebView.

## Installation

Lade den aktuellen Build von [GitHub Releases](https://github.com/willmove/markion/releases) herunter.

| Plattform | Release-Pakete | Ziel |
|---|---|---|
| Windows | NSIS-`.exe`-Installer | x86_64 |
| Linux | `.deb` und AppImage | x86_64 |
| macOS | `.app` und `.dmg` | Apple Silicon (arm64), macOS 11+ |

Releases sind nicht plattformseitig codesigniert. Windows SmartScreen verlangt eventuell **Weitere Informationen → Trotzdem ausführen**, und macOS Gatekeeper eventuell ein Rechtsklicken der App und die Wahl von **Öffnen**. **Hilfe → Nach Updates suchen…** bietet auf jeder Plattform eine umsetzbare Update-Aufforderung: Getaggte Windows-x86_64-NSIS-Installationen erhalten einen kryptografisch verifizierten (cargo-packager-Minisign) Ein-Klick-Download mit Installation, der den Start verweigert, solange ein Dokument ungespeicherte Änderungen enthält; macOS und Linux öffnen die passende Release-Datei im Systembrowser. Intel-Macs können den arm64-Build über Rosetta ausführen; ein Universal-Binary und eine Apple-Notarisierung werden derzeit nicht bereitgestellt.

## Bearbeitungsmodi

Markion hat vier Ansichtsmodi. Standard ist die geteilte Vorschau.

- **Bearbeiten** — ein fokussierter Editor für rohen Markdown-Quelltext.
- **Visuelle Bearbeitung** — eine WYSIWYG-first, quellgestützte Oberfläche. Fließtext bleibt gerendert mit progressiver Syntaxeinblendung; gewöhnliche Fenced-Code-Inhalte, Blockformeln, registrierte Diagramme (Mermaid), Inline-Bildfelder und GFM-Tabellenzellen haben exakte Direkteditoren. Konstrukte, deren WYSIWYG-Rendering noch nicht implementiert ist — YAML-Frontmatter, eingerückter Code sowie fehlerhafte oder byte-mehrdeutige Syntax — behalten als Übergangslösung eine exakte quellgestützte Bearbeitungsmöglichkeit, verfolgt in der [WYSIWYG-Abdeckungsmatrix und Roadmap der visuellen Bearbeitung](docs/visual-editing-quality.md). Eine Slash-Befehlspalette und ein kompaktes Block-Kontextmenü bieten exakte Blockumwandlungen (Absatz, Überschriften, Listen, Zitat, Fenced Code, Trennlinie, Tabelle), Duplizieren, Nach-oben/unten-Verschieben, quellsicheres Ziehen zum Umordnen und Löschen; eine auswahlkontextbezogene Formatierungsleiste und ein visueller Link-Editor führen pro Aktion genau eine quellgestützte Mutation aus. Markdown-Begrenzer werden standardmäßig automatisch gepaart, `:shortcode` öffnet die Emoji-Vervollständigung, Aufgabenlisten-Kästchen lassen sich direkt umschalten, und das Verlassen einer Überschriften-, Listen-, Aufgaben- oder Zitatzeile stellt sie sofort wieder gerendert dar. Dies ist kein separates Rich-Text-Dokumentmodell — das zugrunde liegende Markdown bleibt stets die Wahrheitsquelle.
- **Geteilte Vorschau** — Quelltext und gerenderte Vorschau nebeneinander, mit optionaler quellbasierter Sync-Scroll-Einstellung, die beide Bereiche an derselben Dokumentposition hält, statt nach Prozent des Gesamtdokuments zu scrollen.
- **Lesen** — eine gerenderte, nicht editierbare Ansicht, standardmäßig zentriert auf eine lesbare Maximalbreite von 860 px; die adaptive Vorschaubreite kann den ganzen Bereich nutzen.

Beim Moduswechsel bleiben aktives Dokument, Cursor und Auswahl, Rückgängig-Verlauf und der Scroll-Zustand pro Tab erhalten.

Der Befehl **Ansicht → Quelltext/Geteilte Vorschau** (`Ctrl+/` unter Windows/Linux, `Cmd+/` unter macOS) wechselt zwischen den beiden quellorientierten Layouts. Aus der visuellen Bearbeitung oder dem Lesemodus führt der erste Aufruf in den Bearbeitungsmodus.

## Dokumente und Arbeitsbereich

- Mehrtab-Bearbeitung mit Cursor, Auswahl, Scrollen, Rückgängig/Wiederholen, Vorschau, Gliederung und zwischengespeichertem abgeleitetem Markdown-Zustand pro Tab.
- Das Öffnen einer bereits geöffneten Markdown- oder Textdatei fokussiert ihren vorhandenen Tab, statt ein Duplikat zu erzeugen.
- **Ordner öffnen** wechselt die Arbeitsbereichswurzel und füllt die Dateien-Seitenleiste mit Markdown-Dateien, einer kuratierten Auswahl an Textdateien (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc`) und unterstützten Bilddateien (`.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`/`.tiff`, `.svg`), verschachtelt unter ihren Ordnern; leere Ordner werden ebenfalls gelistet. Markdown bleibt visuell abgehoben, Textdateien öffnen als UTF-8-Text, und Bilddateien öffnen als schreibgeschützte Bild-Tabs, die übergroße Bilder in den Inhaltsbereich einpassen.
- Das Aufklappen eines Ordners zeigt genau eine Ebene von Kindern, sodass tief verschachtelte Arbeitsbereiche Ebene für Ebene erkundet werden können.
- Die Einstellung **Versteckte Dateien anzeigen** (standardmäßig aus) zeigt Dotfile-Einträge plus das Windows-Attribut „Versteckt“, während stets ausgeschlossener Build-, Abhängigkeits- und VCS-Ballast (`target`, `node_modules`, `.git`, …) unabhängig davon verborgen bleibt.
- Kontextmenüs im Dateibaum bieten je nach Ziel: Öffnen, in neuem Tab öffnen, Datei/Ordner anlegen, Umbenennen, Löschen, im System-Dateimanager anzeigen, Filtern und Aktualisieren.
- Dateien und Ordner lassen sich direkt im Baum benennen; das Löschen eines nicht leeren Ordners erfordert eine zusätzliche Bestätigung.
- Markdown-Dateien können aus dem Dateimanager des Betriebssystems in Markion gezogen werden.
- Die Bereiche Dateien und Gliederung sind umschaltbar, und die Trennlinien von Seitenleiste und geteilter Ansicht sind verschiebbar.
- Der Gliederungsbereich listet die Überschriftenhierarchie des Dokuments als einklappbaren Baum: Jede Überschrift mit Nachkommen zeigt ein Auf-/Zuklapp-Steuerelement, Gliederungen starten vollständig aufgeklappt, das Einklappen ist pro Dokument und nur sitzungsbezogen, und der Abschnitt mit dem Cursor wird hervorgehoben. Ein Klick auf eine Überschrift springt zu ihrer Quellposition — oder im Lesemodus zur gerenderten Überschrift.
- Der native Fenstertitel zeigt den aktiven Dateinamen hinter der Markion-Marke, mit einem `*`-Suffix bei ungespeicherten Änderungen. Die Statusleiste hält Speicherzustand und flüchtige Operationsrückmeldungen sowie einen kompakten dauerhaften Kontext: Zeichen- und Wortzahl des aktiven Dokuments, die einsbasierte Zeile und Spalte des Cursors, sofern eine Bearbeitungsfläche vorhanden ist, und den aktuellen Git-Branch, wenn Dokument oder Arbeitsbereich zu einem Repository gehören.
- **Sicherung und Synchronisierung** schützt einen Notizordner an einem Synchronisierungsort mit einem Ein-Klick-Fluss **Jetzt synchronisieren** und verständlichen Statusmeldungen. Git liefert darunter die sichere Versionshistorie; technische Kontrollen bleiben in den erweiterten Git-Details. Siehe die [Anleitung zu Sicherung und Synchronisierung](docs/git-sync.md) für Einrichtung, Authentifizierung, Konflikte und Wiederherstellung.

## Markdown-Bearbeitung und -Vorschau

- Das Parsen übernimmt `pulldown-cmark` mit CommonMark- und GFM-orientierter Unterstützung.
- Formatierungsbefehle decken Absatz (`Ctrl+0` unter Windows/Linux, `Cmd+0` unter macOS), Überschriften, Fett, Kursiv, Inline-Code, Links, Bilder, Listen, Aufgabenlisten, Blockzitate, Fenced-Code-Blöcke und Markdown-Quelltabellen ab. „Absatz“ wandelt überschnittene ATX-Überschriften zurück in normalen Text, ohne Nicht-Überschriften-Zeilen zu verändern.
- Die Markdown-Autopaarung ist in Quelltext und unterstützten Feldern der visuellen Bearbeitung standardmäßig aktiv: Begrenzer können gepaart, Auswahlen umschlossen, vorhandene Schließer übersprungen und leere Paare mit der Rücktaste entfernt werden. Während der IME-Komposition und in Code-, Blockformel- und Diagramm-Inhaltseditoren bleibt sie deaktiviert.
- Das Tippen eines offenen `:shortcode` an einer Wortgrenze öffnet die Emoji-Vervollständigung in Quelltext und visueller Bearbeitung; die Bestätigung fügt kanonisches Markdown ein, das ein einziger rückgängig machbarer Quelleingriff bleibt.
- Das Einfügen von Rich Text aus Word, Google Docs oder Webseiten wandelt die HTML-Variante der Zwischenablage in formatiertes Markdown um — Überschriften, Hervorhebungen, Links, verschachtelte Listen, Tabellen, Code — als ein einziger rückgängig machbarer Schritt; reine Text-Zwischenablagen werden weiterhin wörtlich eingefügt.
- Überschriftenbefehle bieten standardmäßig H1–H5, mit einer H1–H6-Option in den Einstellungen.
- Suchen und Ersetzen unterstützt Groß-/Kleinschreibung, reguläre Ausdrücke, Vor/Zurück-Navigation, Ersetzen der aktuellen Fundstelle und Alles ersetzen.
- Quell-Tabellenbefehle können Tabellen formatieren und Zeilen und Spalten hinzufügen, löschen oder verschieben. Tabellen in der visuellen Bearbeitung bieten zusätzlich direkte quellgestützte Zellbearbeitung, Tab-Navigation, deterministischen Breiten-Umbruch und dieselben Zeilen-/Spaltenoperationen; gewöhnliche Vorschautabellen bleiben schreibgeschützt.
- Ein Bildressourcen-Workflow nimmt Zwischenablagebilder, hereingezogene Dateien, explizites Einfügen per Datei/URL und vorhandene Dokumentbilder auf. Richtlinien können behalten, kopieren/herunterladen oder über PicGo HTTP, PicGo Core oder einen wörtlichen benutzerdefinierten Befehl hochladen; siehe die [Anleitung zu Bildverarbeitung und -upload](docs/image-handling.md). Markion verwendet kollisionsresistente dokumentrelative Ressourcen, bewahrt exakte Bildmetadaten und hält die Übertragungswiederherstellung außerhalb von Markdown.
- YAML-Frontmatter wird geparst und in der Vorschau ausgeblendet; `title`, `author` und `date` speisen die Export-Metadaten.
- Dokumentschreibvorgänge sind atomare Ersetzungen im selben Verzeichnis, die bei einem Schreibfehler den vorhandenen Pfad und den Änderungsstatus bewahren. Markion verfolgt die zuletzt bekannte Dateiidentität auf dem Datenträger, erkennt externe Änderungen vor dem Speichern und während ein Dokument geöffnet ist, lädt nur saubere Dokumente automatisch neu und bietet geänderten Dokumenten eine explizite Konfliktwahl zwischen Neu laden, Überschreiben oder Kopie speichern. Der Wiederherstellungsmanager inventarisiert jeden Wiederherstellungs-Snapshot mit ursprünglichem Pfad und Datenträgerbeziehung und unterstützt Wiederherstellen, Verwerfen, Alle wiederherstellen und Alle verwerfen, ohne unlesbare oder nicht ausgewählte Daten zu löschen.
- Die automatische Speicherung erfolgt standardmäßig nach fünf Sekunden Inaktivität und schreibt Wiederherstellungskopien für ungespeicherte Dokumente; wiederhergestellte Snapshots bleiben dauerhaft, bis ein erfolgreiches Speichern, ein explizites Verwerfen oder ein atomar geschriebener Nachfolger sie ablöst.

Die gerenderte Vorschau unterstützt:

- Fett, Kursiv, Durchgestrichen, Inline-Code, Links, Hervorhebungen, Hochstellung, Tiefstellung, Fußnoten (Referenz berühren, um ihre Definition zu lesen), Aufgabenlisten, gängige Emoji-Shortcodes und automatische Links.
- `[TOC]` / `[toc]`-Token im Dokument werden als live, klickbare Gliederung gerendert, und `[text](#heading)` / `{#id}`-Überschriftensprünge bleiben im Dokument.
- Korrekte Startnummern geordneter Listen, verschachtelte Listen, Aufzählungszeichen je Tiefe, hängende Einzüge, Bilder und eingebettetes HTML.
- Unterstütztes Inline-HTML hat konsistente Semantik über gemischtes Markdown, eigenständige HTML-Blöcke, Tabellenzellen und visuelle Bearbeitung hinweg, einschließlich sicherer Farb-Spans, Links und verlinkter Bilder, `kbd`/`samp`, autorisierter Umbrüche und positionierter Hoch-/Tiefstellung; fehlerhaftes oder nicht unterstütztes Markup fällt zurück, ohne Skripte auszuführen.
- Auswählbarer Vorschautext mit Kontextmenü zum Kopieren als reinen Text, Markdown oder HTML, plus Kopieren der Linkadresse, wo zutreffend.
- `$...$`-Inline- und `$$...$$`-Blockformeln, offline mit einer eingebetteten RaTeX-Engine (KaTeX-kompatibel) mit gebündelten Schriften zu zwischengespeichertem SVG gesetzt — kein Netzwerkzugriff und keine externe LaTeX-Installation erforderlich.
- ` ```mermaid `-Fenced-Diagramme (Flussdiagramme, Sequenzdiagramme und mehr), zu bereinigtem, zwischengespeichertem SVG gerendert.
- Syntaxhervorhebung für Fenced Code mit syntect und dem erweiterten two-face-Grammatiksatz, mit Fallback-Lexer und optionalen Zeilennummern.

## Designs, Sprachen und Einstellungen

- Vierzehn eingebaute Designs: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark und Tokyo Night/Light.
- Eigene Designs nutzen `.toml`-Dateien in Markions lokalem Design-Verzeichnis. Bei der ersten Verwendung wird ein `typewriter.toml`-Beispiel — einschließlich der optionalen `[fonts]`-Tabelle (`editor`, `rendered`, `code`), die Schriftfamilien für den Markdown-Quelltexteditor, gerenderten Fließtext und Code-Oberflächen liefert, solange der Benutzer keine ausdrückliche Präferenz hat — dort als Ausgangspunkt installiert. Alte `.theme`-Dateien werden beim ersten Laden automatisch migriert.
- Sieben Oberflächensprachen: Englisch, vereinfachtes Chinesisch, traditionelles Chinesisch, Japanisch, Französisch, Deutsch und Spanisch.
- Das integrierte Einstellungspanel deckt auf **Allgemein** Sprache, Sichtbarkeit der Seitenleiste, adaptive Vorschaubreite, Fokus-/Schreibmaschinenmodus, Markdown-Autopaarung, Code-Zeilennummern, synchrones Scrollen, Versteckte-Dateien-anzeigen und die Tiefe des Überschriftenmenüs ab; auf **Darstellung** Design, Schriftfamilien pro Ebene (Quelltext, Lesen, Code), Schriftgrößen und Absatzabstand.
- Einstellungen bleiben in `config.toml` erhalten; alte `preferences.conf`-Dateien werden automatisch migriert.

Alle Konfigurationsfelder sind optional. Die wichtigsten Standardwerte und dateibasierten Einstellungen:

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 oder 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" oder "outline"
show_hidden_files = false
markdown_auto_pair = true

# Optionale Schriftfamilien pro Ebene; fehlend = dem Design folgen, dann dem
# eingebauten Standard (System-UI-Schrift für Quelltext/Lesen, JetBrains Mono für Code).
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
delay_secs = 5

[git]
background_check = false

[export]
pdf_engine = "xelatex"
```

Konfiguration, Wiederherstellungsdateien, Designs und rotierende Diagnoseprotokolle nutzen plattformgerechte Markion-Datenverzeichnisse. Setze `RUST_LOG=debug` vor dem Start für ausführlichere Protokolle.

## Export

Markion exportiert nach:

- Einem lokalen MarkNice-Arbeitsbereich zur Vorbereitung von Rich Content für WeChat
- Markdown
- Formatiertem HTML und reinem HTML
- LaTeX
- DOCX
- PDF
- PNG- und JPEG-Textschnappschüssen

PDF und DOCX versuchen zuerst die übernommene Typune/pandoc-Export-Engine. Falls pandoc oder die gewählte PDF-Engine nicht verfügbar ist, fällt Markion auf einen einfacheren eingebauten Writer zurück und meldet das Backend in der Statusleiste. Die Installation von pandoc und einer geeigneten PDF-Engine erzeugt reichhaltigere Ausgaben. PNG/JPEG- und eingebaute PDF-Ausgaben sind bewusst einfache Textschnappschüsse.

## Word-Import

Wähle **Datei → Word (.docx) importieren**, um ein DOCX vollständig lokal in ein neues gespeichertes Markdown-Dokument umzuwandeln. Markion zeigt vor dem Speicherdialog einen Umwandlungsbericht und verlangt eine ausdrückliche Entscheidung, wenn Inhalte verloren gehen könnten. Unterstützte eingebettete Bilder werden neben dem Markdown in `<Name>.assets/` gespeichert; behalte dieses Verzeichnis zusammen mit der `.md`-Datei. Der Importer ist begrenzt, abbrechbar, offline und überschreibt niemals einen vorhandenen Markdown- oder Ressourcenpfad. Er verwendet die angenommene Ansicht der Änderungsverfolgung, bewahrt Word-Überschriftenebenen getreuer und normalisiert Fett-zu-Wort-Grenzen für interoperables Markdown. Alte `.doc`-, `.docm`-, verschlüsselte Dateien, PDF/OCR und Seitenlayout-Treue liegen außerhalb dieses Workflows. Siehe [`docs/word-import.md`](docs/word-import.md) für die unterstützte semantische Teilmenge, Grenzen, Diagnosen und Aufräumverhalten.

Wähle **Exportieren → Für WeChat veröffentlichen (MarkNice)**, um das aktive In-Memory-Dokument in einem privaten Loopback-Arbeitsbereich im Standardbrowser zu öffnen. Die mitgelieferte Editor-Oberfläche folgt dem angepinnten MarkNice-Editor-Abschnitt (Doppelkarten, Ampel-Kopfzeilen, SVG-Werkzeugleiste, 375-px-Phone-Rahmen) so nahe, wie es eine lokale, nicht werbliche Hülle erlaubt. Designs, Renderer, Formelsatz und Anwendungsskripte funktionieren offline. Formeln werden als in sich geschlossenes Inline-SVG (MathJax) gesetzt und überstehen so die Einfüge-Filterung des WeChat-Editors ohne Duplikate. Browser-Änderungen bleiben in diesem Tab und werden niemals nach Markion zurückgeschrieben. Sein **Word importieren** wandelt eine `.docx`-Datei nur innerhalb dieser Browsersitzung um; hole sie mit **MD kopieren** nach Markion zurück oder speichere das Sitzungs-Markdown anderweitig. Verwaltete lokale Bilder können in der Vorschau angezeigt werden, doch das Kopieren erfordert ihr ausdrückliches Weglassen, da ein lokaler Blob nicht von WeChat veröffentlicht werden kann; Remote-Bilder können weiterhin ihre ursprünglichen Hosts kontaktieren. Der Arbeitsbereich kann außerdem sein exakt aktuelles Markdown kopieren, eigenständiges designtes HTML speichern, eine browsererzeugte MarkNice-Word-Datei speichern oder über den Druckdialog **Als PDF speichern** (gedruckt wird die aktuelle designte, bereinigte MarkNice-Vorschau; im Druckdialog „Als PDF speichern“ wählen). Browser-Word und -PDF unterscheiden sich von Markions nativen/Pandoc-Exporteuren. Der Arbeitsbereich importiert keine Markdown-Dateien, PDFs oder lokalen Bilder und hat keine Beispieldokument-Aktion. Läuft die Zwischenablage-Berechtigung oder die Zweistundensitzung ab, erteile die Berechtigung oder starte erneut aus Markion.

## Leistung

- Vorschaublöcke, Blöcke der visuellen Bearbeitung, Gliederung, Statistiken und Zeilenzahlen werden pro Dokumentversion zwischengespeichert und via `Arc` geteilt.
- Die Syntaxhervorhebung wird über Bearbeitungen hinweg memoisiert, und das Laden der Grammatiken wird im Hintergrund vorgewärmt.
- Rückgängig-Snapshots überspringen abgeleitete Caches, während der Editor einen pro Version zwischengespeicherten Text-Handle wiederverwendet.
- Vorschau-/Visual-Edit-Listen aktualisieren nur geänderte Bereiche, der Dateibaum rendert eine begrenzte Zeilenmenge, und umbrochene Quellzeilen messen ihre tatsächliche Rendehöhe.

Die quellbasierte visuelle Bearbeitung verwendet nach lokalen Eingriffen unabhängig parsbare Regionen inkrementell wieder und fällt auf eine vollständige Ableitung zurück, sobald Markdown-Kontext oder Byte-Bereiche unsicher sind. Die Ableitung der geteilten/Lese-Vorschau bleibt entprellt und zwischengespeichert. Markion nutzt weiterhin einen `String`-Puffer statt eines Rope, und manche semantischen Lesevorgänge erfordern absichtlich ein vollständiges Parsen.

## Aktuelle Einschränkungen

- Die visuelle Bearbeitung ist WYSIWYG-first bei Beibehaltung des kanonischen Markdown; Konstrukte ohne nachgewiesenes byte-exaktes Rendering zeigen als Übergangslösung exakten Quelltext (verfolgt auf der [WYSIWYG-Abdeckungsroadmap](docs/visual-editing-quality.md)), statt eine geratene Rich-Tree-Mutation zu akzeptieren, und das Umordnen von Blöcken wird nur angeboten, wenn nicht überlappende Quellgrenzen beweisbar sind. Aktuelle Hauptlücken sind dekodierte HTML-Entities, Frontmatter und eingerückte Codeblöcke; sekundäre fehlerhafte oder nicht unterstützte Konstrukte bleiben in der Matrix gelistet.
- Die Bildschirmdarstellung (geteilte/Lese-Vorschau und visuelle Bearbeitung) setzt Formeln mit der eingebetteten RaTeX-Engine; der LaTeX-Export behält natives `$...$`/`$$...$$`-Quellmaterial für die eigene Werkzeugkette des Lesers, und der eingebaute DOCX-Export-Fallback (nur bei fehlendem pandoc) stuft Formeln weiterhin auf eine lesbare Klartext-Annäherung herab, statt gesetzte Glyphen einzubetten.
- Tabellenzellen der visuellen Bearbeitung unterstützen direkte Bearbeitung plus Fett/Kursiv/Inline-Code/Link auf einer Auswahl, die innerhalb einer Zelle bleibt. GFM-Spaltenbreiten lassen sich ziehen und werden als HTML-Kommentar (`<!-- markion-cols:… -->`) unmittelbar vor der Tabelle persistiert. Inline-Bilder akzeptieren ganzzahlige Breiten von 10–100 % und lassen sich per Ziehen skalieren. Referenz-/mehrzeilige Bilder und fehlerhafte Tabellen bleiben bekannte Lücken der WYSIWYG-Abdeckungsroadmap mit quellgestützten Bearbeitungspfaden.
- Ein-Klick-Updates installieren die Windows-NSIS-Distribution nach Minisign-Verifikation; der Austausch des macOS-Bundles und die Selbstersetzung von Linux-`.deb`/AppImage bleiben zukünftige Arbeit, und die Update-Authentifizierung ist weder Windows Authenticode noch Apple-Notarisierung.
- Drag-and-drop-Verschiebungen im Dateibaum und eine vollständige Installations-UI für eigene Designs sind nicht implementiert.
- Der Bildexport ist ein einfacher Textschnappschuss, und sehr große Dokumente verwenden noch kein Rope oder vollständig inkrementelles Parsen über alle abgeleiteten Subsysteme hinweg.

## Entwicklung

Rust stable ist erforderlich. Vom Repository-Stamm aus:

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

Der Qualitätsbefehl prüft Rust-Formatierung und -Lints, die vollständige Test-Suite des Cargo-Workspace, das angepinnte MarkNice-Bundle und jedes OpenSpec-Artefakt im Strict-Modus. Siehe die [Anleitung zum lokalen MarkNice-Arbeitsbereich](docs/marknice-workspace.md) und den [Unterstützungs- und Engineering-Vertrag der visuellen Bearbeitung](docs/visual-editing-quality.md).

Das Wurzelpaket ist das Anwendungs-Crate `markion`. Von Typune übernommene, GPUI-freie Bibliotheks-Crates liegen unter `crates/*`:

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

Ein einfaches `cargo test` testet nur das Wurzelpaket; nutze `cargo test --workspace` für jedes Mitglied. Unter Windows ist die App ein GUI-Subsystem-Executable und kann nach einem Debug-Build auch so gestartet werden:

```powershell
.\target\debug\markion.exe
```

## Lizenz

Markion steht unter der [MIT-Lizenz](LICENSE).
