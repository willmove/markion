<a id="french"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Logo Markion" width="128" height="128">
</p>

<p align="center">
  <a href="README.md#english">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <strong>Français</strong> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a>
</p>

# Markion

Markion est un éditeur Markdown de bureau natif, construit avec Rust et GPUI. Il réunit dans une application légère une édition de source réactive, un mode d'édition visuelle adossé au source, un aperçu en direct, des outils d'espace de travail et une exportation multiformat. Le Markdown reste le format canonique du document — ni Electron, ni Tauri, ni WebView.

## Installation

Téléchargez la dernière version depuis [GitHub Releases](https://github.com/willmove/markion/releases).

| Plateforme | Paquets de publication | Cible |
|---|---|---|
| Windows | Installeur NSIS `.exe` | x86_64 |
| Linux | `.deb` et AppImage | x86_64 |
| macOS | `.app` et `.dmg` | Apple Silicon (arm64), macOS 11+ |

Les publications ne sont pas signées au niveau de la plateforme. Windows SmartScreen peut exiger **Informations complémentaires → Exécuter quand même**, et macOS Gatekeeper peut exiger un clic droit sur l'application puis **Ouvrir**. **Aide → Rechercher des mises à jour…** propose une invite de mise à jour exploitable sur toutes les plateformes : les installations NSIS Windows x86_64 étiquetées bénéficient d'un téléchargement-installation en un clic vérifié cryptographiquement (Minisign via cargo-packager) qui refuse de démarrer tant qu'un document comporte des modifications non enregistrées, tandis que macOS et Linux ouvrent le fichier de publication correspondant dans le navigateur système. Les Mac Intel peuvent exécuter la version arm64 via Rosetta ; un binaire universel et la notarisation Apple ne sont pas fournis actuellement.

## Modes d'édition

Markion propose quatre modes d'affichage. L'aperçu fractionné est le mode par défaut.

- **Édition** — un éditeur de code source Markdown brut et concentré.
- **Édition visuelle** — une surface WYSIWYG d'abord, adossée au source. Le texte reste rendu avec révélation progressive de la syntaxe ; les contenus de blocs de code ordinaires, les formules mathématiques en bloc, les diagrammes enregistrés (Mermaid), les champs d'images en ligne et les cellules de tableaux GFM disposent d'éditeurs directs exacts. Les constructions dont le rendu WYSIWYG n'est pas encore implémenté — front matter YAML, code indenté et syntaxe malformée ou ambiguë en octets — conservent une possibilité d'édition exacte adossée au source à titre transitoire, suivie dans la [matrice de couverture WYSIWYG et la feuille de route de l'édition visuelle](docs/visual-editing-quality.md). Une palette de commandes slash et un menu contextuel de bloc compact au clic droit offrent des transformations de bloc exactes (paragraphe, titres, listes, citation, bloc de code, séparateur, tableau), la duplication, le déplacement vers le haut/bas, le réordonnancement par glisser sans risque pour le source et la suppression ; une barre d'outils de formatage contextuelle à la sélection et un éditeur de liens visuel effectuent une mutation exacte du source par action. Les délimiteurs Markdown s'apparient automatiquement par défaut, `:shortcode` ouvre la complétion d'emoji, les cases des listes de tâches se basculent directement, et quitter une ligne de titre, de liste, de tâche ou de citation la ramène immédiatement à sa forme rendue. Il ne s'agit pas d'un modèle de document riche séparé — le Markdown sous-jacent reste toujours la source de vérité.
- **Aperçu fractionné** — le source et l'aperçu rendu côte à côte, avec un réglage optionnel de défilement synchronisé fondé sur le source qui maintient les deux panneaux au même emplacement du document plutôt que de défiler selon un pourcentage du document entier.
- **Lecture** — une vue rendue, non éditable, centrée avec une largeur maximale de 860 px confortable par défaut ; la largeur adaptative de l'aperçu peut utiliser tout le panneau.

Changer de mode conserve le document actif, le curseur et la sélection, l'historique d'annulation et l'état de défilement de chaque onglet.

La commande **Affichage → Source/Aperçu fractionné** (`Ctrl+/` sous Windows/Linux, `Cmd+/` sous macOS) alterne les deux dispositions orientées source. Depuis l'édition visuelle ou la lecture, sa première invocation ouvre le mode édition.

## Documents et espace de travail

- Édition multi-onglets avec, par onglet, curseur, sélection, défilement, annulation/rétablissement, aperçu, plan et état Markdown dérivé mis en cache.
- Ouvrir un fichier Markdown ou texte déjà ouvert focalise son onglet existant au lieu de créer un doublon.
- **Ouvrir un dossier** change la racine de l'espace de travail et remplit la barre latérale Fichiers avec les fichiers Markdown, un ensemble choisi de fichiers texte (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc`) et les fichiers image pris en charge (`.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`/`.tiff`, `.svg`), imbriqués sous leurs dossiers ; les dossiers vides sont également listés. Le Markdown reste visuellement distingué, les fichiers texte s'ouvrent en UTF-8 et les images s'ouvrent dans des onglets image en lecture seule qui ajustent les images trop grandes à la zone de contenu.
- Déployer un dossier ne révèle qu'un seul niveau d'enfants, ce qui permet d'explorer les espaces de travail profondément imbriqués niveau par niveau.
- Une préférence **Afficher les fichiers cachés** (désactivée par défaut) révèle les entrées en point (dotfiles) ainsi que l'attribut caché Windows, tandis que le bruit toujours exclu des builds, dépendances et VCS (`target`, `node_modules`, `.git`, …) reste masqué quoi qu'il arrive.
- Les menus contextuels de l'arborescence proposent selon le cas : ouvrir, ouvrir dans un nouvel onglet, créer un fichier/dossier, renommer, supprimer, révéler dans le gestionnaire de fichiers système, filtrer et actualiser.
- Les fichiers et dossiers se nomment en place ; la suppression d'un dossier non vide exige une confirmation supplémentaire.
- On peut glisser des fichiers Markdown depuis le gestionnaire de fichiers du système vers Markion.
- Les panneaux Fichiers et Plan sont affichables/masquables, et les séparateurs de la barre latérale et des panneaux fractionnés sont déplaçables.
- Le panneau Plan liste la hiérarchie des titres du document sous forme d'arborescence repliable : chaque titre ayant des descendants expose un contrôle de déploiement, les plans démarrent entièrement déployés, le repli est propre au document et limité à la session, et la section contenant le curseur est mise en surbrillance. Cliquer un titre saute à sa position dans le source — ou au titre rendu en mode lecture.
- Le titre de la fenêtre native affiche le nom du fichier actif après la marque Markion, avec un suffixe `*` lorsque le document comporte des modifications non enregistrées. La barre d'état conserve l'état d'enregistrement et les retours d'opération transitoires, plus un contexte persistant compact : le nombre de caractères et de mots du document actif, la ligne et la colonne (à partir de 1) du curseur lorsqu'une surface d'édition est présente, et la branche Git courante lorsque le document ou l'espace de travail appartient à un dépôt.
- **Sauvegarde et synchro** protège un dossier de notes à un emplacement de synchronisation avec un flux **Synchroniser maintenant** en un clic et un état en langage clair. Git fournit l'historique de versions sûr en dessous ; les contrôles techniques restent dans les détails Git avancés. Voir le [guide Sauvegarde et synchronisation](docs/git-sync.md) pour la configuration, l'authentification, les conflits et la récupération.

## Édition et aperçu Markdown

- L'analyse est assurée par `pulldown-cmark` avec une prise en charge orientée CommonMark et GFM.
- Les commandes de formatage couvrent le paragraphe (`Ctrl+0` sous Windows/Linux, `Cmd+0` sous macOS), les titres, le gras, l'italique, le code en ligne, les liens, les images, les listes, les listes de tâches, les citations, les blocs de code et les tableaux Markdown en source. « Paragraphe » reconvertit les titres ATX intersectés en texte ordinaire sans modifier les lignes non-titre.
- L'appariement automatique Markdown est activé par défaut dans le source et les champs d'édition visuelle pris en charge : les délimiteurs peuvent être appariés, les sélections encadrées, les fermants existants ignorés et les paires vides supprimées avec Retour arrière. Il reste désactivé pendant la composition IME et dans les éditeurs de contenu de code, de formules en bloc et de diagrammes.
- Taper un `:shortcode` ouvert à une frontière de mot ouvre la complétion d'emoji dans le source et l'édition visuelle ; la confirmation insère un Markdown canonique qui reste une seule modification de source annulable.
- Coller du texte riche copié depuis Word, Google Docs ou des pages web convertit la saveur HTML du presse-papiers en Markdown formaté — titres, emphase, liens, listes imbriquées, tableaux, code — en une seule étape annulable ; les presse-papiers en texte brut restent collés mot à mot.
- Les commandes de titre exposent H1–H5 par défaut, avec une option H1–H6 dans les Préférences.
- La recherche et le remplacement prennent en charge la sensibilité à la casse, les expressions régulières, la navigation suivant/précédent, le remplacement de l'occurrence courante et le remplacement global.
- Les commandes de tableau en source peuvent formater les tableaux et ajouter, supprimer ou déplacer des lignes et des colonnes. Les tableaux de l'édition visuelle offrent en plus l'édition directe des cellules adossée au source, la traversée par Tab, un réajustement de largeur déterministe et les mêmes opérations de lignes/colonnes ; les tableaux d'aperçu ordinaires restent en lecture seule.
- Un flux de ressources d'images ingère les images du presse-papiers, les fichiers glissés, l'insertion explicite par fichier/URL et les images déjà présentes dans le document. Les règles peuvent conserver, copier/télécharger ou envoyer via PicGo HTTP, PicGo Core ou une commande personnalisée littérale ; voir le [guide de gestion et d'envoi des images](docs/image-handling.md). Markion utilise des ressources relatives au document résistantes aux collisions, préserve exactement les métadonnées des images et garde la récupération de transfert hors du Markdown.
- Le front matter YAML est analysé et masqué de l'aperçu ; `title`, `author` et `date` alimentent les métadonnées d'exportation.
- Les écritures de documents sont des remplacements atomiques dans le même répertoire qui préservent le chemin existant et l'état modifié en cas d'échec. Markion suit la dernière identité connue du fichier sur disque, détecte les modifications externes avant l'enregistrement et pendant que le document est ouvert, ne recharge automatiquement que les documents propres et offre aux documents modifiés un choix explicite de rechargement, d'écrasement ou d'enregistrement d'une copie. Le gestionnaire de récupération inventorie chaque instantané de récupération avec son chemin d'origine et sa relation au disque, et prend en charge Restaurer, Ignorer, Tout restaurer et Tout ignorer sans supprimer les données illisibles ou non sélectionnées.
- L'enregistrement automatique se déclenche par défaut après cinq secondes d'inactivité et écrit des copies de récupération pour les documents non enregistrés ; les instantanés restaurés restent durables jusqu'à un enregistrement réussi, une suppression explicite ou le remplacement par un successeur écrit atomiquement.

L'aperçu rendu prend en charge :

- Gras, italique, barré, code en ligne, liens, surlignages, exposant, indice, notes de bas de page (survolez une référence pour lire sa définition), listes de tâches, codes courts d'emoji courants et liens automatiques.
- Les marqueurs `[TOC]` / `[toc]` dans le document sont rendus comme un plan dynamique cliquable, et les sauts de titre `[text](#heading)` / `{#id}` restent à l'intérieur du document.
- Numéros de départ corrects des listes ordonnées, listes imbriquées, puces par profondeur, indentation suspendue, images et HTML incorporé.
- Le HTML en ligne pris en charge a une sémantique cohérente entre Markdown mixte, blocs HTML autonomes, cellules de tableau et édition visuelle, y compris les spans de couleur sûrs, les liens et images liées, `kbd`/`samp`, les sauts de ligne créés et les exposants/indices correctement positionnés ; le balisage malformé ou non pris en charge se replie sans exécuter de scripts.
- Texte d'aperçu sélectionnable avec menu contextuel pour copier en texte brut, Markdown ou HTML, plus la copie de l'adresse des liens le cas échéant.
- Les formules `$...$` en ligne et `$$...$$` en bloc, composées hors ligne en SVG mis en cache par un moteur RaTeX embarqué (compatible KaTeX) avec polices intégrées — aucun accès réseau ni installation LaTeX externe requis.
- Les diagrammes en bloc ` ```mermaid ` (organigrammes, diagrammes de séquence, etc.) rendus en SVG assaini et mis en cache.
- Code en bloc coloré syntaxiquement avec syntect et l'ensemble de grammaires étendu two-face, avec un lexeur de repli et des numéros de ligne optionnels.

## Thèmes, langues et préférences

- Quatorze thèmes intégrés : Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark et Tokyo Night/Light.
- Les thèmes personnalisés utilisent des fichiers `.toml` dans le répertoire local des thèmes de Markion. À la première utilisation, un exemple `typewriter.toml` — incluant la table optionnelle `[fonts]` (`editor`, `rendered`, `code`) qui fournit des familles de polices pour l'éditeur de source Markdown, le texte rendu et les surfaces de code lorsque l'utilisateur n'a pas de préférence explicite — y est installé comme point de départ. Les anciens fichiers `.theme` migrent automatiquement au premier chargement.
- Sept langues d'interface : anglais, chinois simplifié, chinois traditionnel, japonais, français, allemand et espagnol.
- Le panneau Préférences intégré couvre dans **Général** : langue, visibilité de la barre latérale, largeur adaptative de l'aperçu, modes concentré/machine à écrire, appariement automatique Markdown, numéros de ligne de code, défilement synchronisé, affichage des fichiers cachés et profondeur du menu des titres ; et dans **Apparence** : thème, familles de polices par plan (source, lecture, code), tailles de police et espacement des paragraphes.
- Les préférences persistent dans `config.toml` ; les anciens fichiers `preferences.conf` migrent automatiquement.

Tous les champs de configuration sont optionnels. Les principales valeurs par défaut et les réglages accessibles uniquement par fichier :

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 ou 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" ou "outline"
show_hidden_files = false
markdown_auto_pair = true

# Familles de polices optionnelles par plan ; absentes = suivre le thème, puis
# la valeur intégrée par défaut (police UI système pour source/lecture, JetBrains Mono pour le code).
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

Configuration, fichiers de récupération, thèmes et journaux de diagnostic à rotation utilisent les répertoires de données Markion appropriés à chaque plateforme. Définissez `RUST_LOG=debug` avant le lancement pour des journaux plus détaillés.

## Exportation

Markion exporte vers :

- Un espace de travail MarkNice local pour préparer du contenu riche pour WeChat
- Markdown
- HTML stylé et HTML brut
- LaTeX
- DOCX
- PDF
- Instantanés texte PNG et JPEG

PDF et DOCX essaient d'abord le moteur d'exportation Typune/pandoc intégré. Si pandoc ou le moteur PDF choisi est indisponible, Markion se replie sur un rédacteur intégré plus simple et indique le moteur utilisé dans la barre d'état. Installer pandoc et un moteur PDF adapté produit des sorties plus riches. Les sorties PNG/JPEG et PDF intégrées restent volontairement de simples instantanés texte.

## Import Word

Choisissez **Fichier → Importer Word (.docx)** pour convertir un DOCX localement en un nouveau document Markdown enregistré. Markion affiche un rapport de conversion avant le sélecteur d'enregistrement et exige un choix explicite lorsque du contenu peut être perdu. Les images incorporées prises en charge sont enregistrées à côté du Markdown dans `<nom>.assets/` ; conservez ce répertoire avec le fichier `.md`. L'importateur est borné, annulable, hors ligne et n'écrase jamais un Markdown ou un chemin de ressources existant. Il utilise la vue acceptée du suivi des modifications, préserve plus fidèlement les niveaux de titre Word et normalise les frontières entre gras et mots pour un Markdown interopérable. Les anciens `.doc`, `.docm`, les fichiers chiffrés, le PDF/OCR et la fidélité de mise en page sont hors de ce flux. Voir [`docs/word-import.md`](docs/word-import.md) pour le sous-ensemble sémantique pris en charge, les limites, les diagnostics et le comportement de nettoyage.

Choisissez **Exporter → Publier pour WeChat (MarkNice)** pour ouvrir le document actif en mémoire dans un espace de travail privé en boucle locale dans votre navigateur par défaut. L'habillage d'éditeur fourni suit la section d'éditeur MarkNice épinglée (double carte, en-têtes façon feux tricolores, barre d'outils SVG, cadre téléphone 375 px) d'aussi près qu'un shell local et non promotionnel le permet. Thèmes, moteur de rendu, composition mathématique et scripts applicatifs fonctionnent hors ligne. Les formules sont composées en SVG en ligne autonome (MathJax), elles survivent donc au filtrage au collage de l'éditeur WeChat sans duplication. Les modifications faites dans le navigateur restent dans cet onglet et ne sont jamais réécrites dans Markion. Son **Importer Word** ne convertit un fichier `.docx` qu'à l'intérieur de cette session de navigateur ; récupérez-le dans Markion avec **Copier MD** ou en enregistrant le Markdown de session ailleurs. Les images locales gérées peuvent être prévisualisées, mais la copie exige de les omettre explicitement car un blob local ne peut pas être publié par WeChat ; les images distantes peuvent encore contacter leurs hôtes d'origine. L'espace de travail peut aussi copier son Markdown courant exact, enregistrer du HTML autonome thémé, enregistrer un fichier Word MarkNice généré par le navigateur, ou **Enregistrer en PDF** en imprimant l'aperçu thémé assaini courant (choisissez « Enregistrer au format PDF » dans la boîte de dialogue d'impression). Les Word et PDF du navigateur sont distincts des exportateurs natifs/Pandoc de Markion. L'espace de travail n'importe ni fichiers Markdown, ni PDF, ni images locales, et n'a pas d'action de document d'exemple. Si la permission du presse-papiers ou la session de deux heures expire, accordez la permission ou relancez depuis Markion.

## Performances

- Blocs d'aperçu, blocs d'édition visuelle, plan, statistiques et nombres de lignes sont mis en cache par version de document et partagés via `Arc`.
- La coloration syntaxique est mémoïsée entre les éditions, et le chargement des grammaires est préchauffé en arrière-plan.
- Les instantanés d'annulation ignorent les caches dérivés, tandis que l'éditeur réutilise un handle de texte mis en cache par version.
- Les listes d'aperçu/édition visuelle ne mettent à jour que les plages modifiées, l'arborescence de fichiers ne rend qu'un ensemble borné de lignes, et les lignes de source enveloppées mesurent leur hauteur rendue réelle.

L'édition visuelle cartographiée au source réutilise incrémentalement les régions analysables indépendamment après des éditions localisées et se replie sur une dérivation complète dès que le contexte Markdown ou les plages d'octets sont incertains. La dérivation des aperçus fractionné/lecture reste débouncée et mise en cache. Markion utilise encore un tampon `String` plutôt qu'une rope, et certaines lectures sémantiques exigent volontairement une analyse complète.

## Limites actuelles

- L'édition visuelle est WYSIWYG d'abord tout en conservant le Markdown canonique ; les constructions sans rendu exact à l'octet prouvé exposent le source exact à titre transitoire (suivi sur la [feuille de route de couverture WYSIWYG](docs/visual-editing-quality.md)) plutôt que d'accepter une mutation d'arbre riche devinée, et le réordonnancement de blocs n'est proposé que lorsque des frontières de source non chevauchantes sont prouvables. Les principales lacunes actuelles incluent les entités HTML décodées, le front matter et les blocs de code indentés ; les constructions secondaires malformées ou non prises en charge restent listées dans la matrice.
- Le rendu à l'écran (aperçus fractionné/lecture et édition visuelle) compose les mathématiques avec le moteur RaTeX embarqué ; l'exportation LaTeX conserve le source natif `$...$`/`$$...$$` pour la chaîne d'outils du lecteur, et le repli d'exportation DOCX intégré (utilisé seulement quand pandoc est indisponible) dégrade encore les formules en une approximation texte lisible au lieu d'embarquer des glyphes composés.
- Les cellules de tableau de l'édition visuelle prennent en charge l'édition directe plus Gras/Italique/Code en ligne/Lien sur une sélection qui reste dans une seule cellule. Les largeurs de colonnes GFM se déplacent à la souris et persistent comme commentaire HTML (`<!-- markion-cols:… -->`) immédiatement avant le tableau. Les images en ligne acceptent des largeurs entières de 10 à 100 % et se redimensionnent par glisser. Les images par référence/multilignes et les tableaux malformés restent des lacunes connues de la feuille de route de couverture WYSIWYG qui conservent des chemins d'édition adossés au source.
- Les mises à jour en un clic installent la distribution NSIS Windows après vérification Minisign ; le remplacement du bundle macOS et l'auto-remplacement des `.deb`/AppImage Linux restent des travaux futurs, et l'authentification des mises à jour n'est ni Windows Authenticode ni la notarisation Apple.
- Le déplacement par glisser-déposer dans l'arborescence de fichiers et une UI complète d'installation de thèmes personnalisés ne sont pas implémentés.
- L'exportation d'images est un instantané texte basique, et les très gros documents n'utilisent pas encore de rope ni une analyse entièrement incrémentale dans tous les sous-systèmes dérivés.

## Développement

Rust stable est requis. Depuis la racine du dépôt :

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

La commande qualité vérifie le formatage et les lints Rust, la suite de tests complète de l'espace de travail Cargo, le bundle MarkNice épinglé et chaque artefact OpenSpec en mode strict. Voir le [guide de l'espace de travail MarkNice local](docs/marknice-workspace.md) et le [contrat de prise en charge et d'ingénierie de l'édition visuelle](docs/visual-editing-quality.md).

Le paquet racine est le crate applicatif `markion`. Les crates de bibliothèque issus de Typune et sans GPUI vivent sous `crates/*` :

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

Un simple `cargo test` ne teste que le paquet racine ; utilisez `cargo test --workspace` pour chaque membre. Sous Windows, l'application est un exécutable du sous-système GUI et peut aussi se lancer après un build de débogage avec :

```powershell
.\target\debug\markion.exe
```

## Licence

Markion est disponible sous [licence MIT](LICENSE).
