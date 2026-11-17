<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>Écrivez naturellement. Gardez votre Markdown.</strong><br>Un éditeur natif pour vos notes, documents techniques et prochains articles. Conçu avec Rust + GPUI.</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <strong>Français</strong> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>Télécharger</strong></a> · <a href="#new-in-v061">Nouveautés</a> · <a href="#performance">Performances</a> · <a href="docs/user-guide.md">Guide</a></p>

![Markion sous Windows : Markdown rendu, tâches et tableau entre les fichiers et le plan indépendant à droite](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Capture réelle sous Windows, version de développement v0.6.0 · <a href="docs/images/README.md">Captures et espace de démonstration</a></sub></p>

## Pourquoi Markion

| ✍️ Écrire sans friction | 📁 Garder la maîtrise | 📤 Aller jusqu’à la publication |
| --- | --- | --- |
| L’édition visuelle masque la syntaxe selon les besoins. Passez à la source ou à l’aperçu partagé à tout moment. | De simples fichiers Markdown locaux, sans compte. Un éditeur natif Rust + GPUI, sans Electron ni WebView intégré. | Importez Word, rédigez du code, des formules et des diagrammes, puis exportez en PDF, Word, HTML ou images. |

<a id="new-in-v060"></a><a id="new-in-v061"></a>

## Nouveautés de la v0.6.1

| À découvrir | Pour votre écriture |
| --- | --- |
| **Interface coréenne** | Huit langues complètes ; choisissez **한국어** dans les préférences. |
| **Espacement à votre goût** | Interligne de 75–300% et espacement des paragraphes de 0–128px dans **Préférences → Apparence**. |
| **Mises à jour préparées en arrière-plan** | Vérification toutes les cinq heures, téléchargement automatique activé par défaut. Installation après confirmation, modifications non enregistrées protégées. |
| **Insertion sans perte de contenu** | Menu compact et ajout de tableau/séparateur en dessous sans remplacer le contenu existant ; poursuivez la saisie après un séparateur. |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Menu de bloc réel dans Markion : titres, listes, actions de bloc et mise en forme en ligne"></p>
<p align="center"><sub>v0.6.0</sub></p>

[Notes de version](docs/release-notes/release-notes-v0.6.1.md) · [État du projet et vérifications](docs/project-status.md)

<a id="install"></a>

## Téléchargez et commencez à écrire

| Plateforme | Téléchargements v0.6.1 |
| --- | --- |
| **Windows x64** | [Installateur](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-setup.exe) · [ZIP portable](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-portable.zip) |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_x64.dmg) |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x86_64.rpm) |

[Toutes les versions et vérifications](https://github.com/willmove/markion/releases)

- **Windows :** lancez l’installateur ou décompressez le ZIP. Pas encore de signature Authenticode ; SmartScreen peut demander **Informations complémentaires → Exécuter quand même**.
- **macOS :** glissez Markion dans Applications. **La v0.6.1 est signée ad-hoc et non notarisée.** Pour une copie fiable téléchargée par navigateur, retirez la quarantaine après installation :

  ```bash
  xattr -cr /Applications/Markion.app
  ```

  À répéter après chaque mise à jour téléchargée par navigateur. [Premier lancement](docs/user-guide.md#install).
- **Linux :** rendez l’AppImage exécutable (`chmod +x markion*.AppImage`), puis lancez-la, ou installez le DEB/RPM. Sans FUSE, utilisez `--appimage-extract-and-run`.

Ouvrez un fichier ou dossier Markdown. Avant de modifier les exemples, **copiez tout le dossier examples, images comprises**. [Premiers pas et modèles](assets/examples/README.md).

## Un document, quatre façons de travailler

| Mode | Usage |
| --- | --- |
| **Édition visuelle** · par défaut | Écrivez dans le Markdown rendu et modifiez directement texte, tableaux, code et formules. |
| **Source** | Maîtrisez toute la syntaxe Markdown. |
| **Aperçu partagé** | Source et aperçu côte à côte, avec défilement synchronisé facultatif. |
| **Lecture** | Un affichage épuré, sans édition. |

Changez de mode en conservant position, sélection et historique d’annulation. [Raccourcis](docs/keyboard-shortcuts.md).

| Du brouillon au document final | Fonctions incluses |
| --- | --- |
| **Rédaction technique** | Tableaux GFM, coloration du code, formules LaTeX, Mermaid, notes de bas de page, encadrés et plan navigable. |
| **Reprendre un travail existant** | Import DOCX local et collage enrichi depuis Word, Google Docs ou le Web. [Import Word](docs/word-import.md) |
| **Exporter et publier** | Markdown, HTML avec/sans styles, LaTeX, DOCX, PDF, PNG/JPEG et espace MarkNice local pour WeChat. [Publication](docs/marknice-workspace.md) |
| **Images et ressources** | Collage, glisser-déposer, copie/téléchargement ou envoi via PicGo/commande personnalisée. [Images](docs/image-handling.md) |
| **À votre goût** | Thèmes, polices par surface et typographie dans **Préférences → Apparence** ; modes concentration/machine à écrire et raccourcis personnalisés. |
| **Connexions facultatives** | Sauvegarde et synchronisation via Git ; AI Agent avec votre clé ou des modèles locaux. L’IA est **désactivée par défaut**. [Synchronisation](docs/git-sync.md) · [IA](docs/ai-agent-guide.md) |

<a id="performance"></a>

## Des performances vérifiables

**Mesures historiques sous Windows, compilation release · 2026-11-11.** Intel Core Ultra 5 225U, ~31 GiB de RAM ; 100 modifications locales par taille. Comparaison du **traitement du modèle**, hors disposition/paint de l’interface et présentation GPU.

| Markdown mixte | Construction initiale | Édition P50 | Édition P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

Lors d’un essai distinct de **1080 événements clavier système** sur 1 MiB, le délai rappel d’entrée de l’application → paint du curseur était de **26.63 ms P50 / 33.14 ms P95**. Le transport matériel/OS et la présentation GPU sont exclus. Ces résultats antérieurs ne sont ni un nouveau test de la v0.6.1 ni un classement des concurrents. [Méthode, données brutes et reproduction](docs/performance.md).

## À savoir avant de commencer

- Markdown reste la source de vérité. Les syntaxes non prises en charge ou ambiguës au niveau des octets utilisent une édition source prudente. Pas de cellules fusionnées GFM ni de recadrage d’image au pixel. [Couverture et limites](docs/visual-editing-quality.md).
- Les exports PDF/DOCX intégrés n’exigent aucun convertisseur externe ; Pandoc est facultatif, avec repli intégré en cas d’échec. La fidélité dépend du format. L’espace MarkNice dans le navigateur suit l’apparence de son éditeur épinglé. L’import Word reste dans la session, sans écriture dans Markion : utilisez **Copy MD** ou enregistrez explicitement le Markdown pour le récupérer. Ses exports Word et PDF par impression thémée diffèrent des exports natifs ; cet espace ne propose ni import Markdown/PDF/image ni actions sur des exemples.
- Les installations Windows et AppImage Linux permettent l’installation des mises à jour vérifiées par Minisign. macOS et DEB/RPM ouvrent un téléchargement. Cette vérification est distincte de la signature du système.
- **README et interface en 8 langues** : anglais, chinois simplifié/traditionnel, japonais, coréen, français, allemand et espagnol. Choisissez **한국어** dans les préférences.

## Compiler et contribuer

Avec Rust stable et les outils natifs de votre plateforme ([recettes CI](.github/workflows/release.yml)), depuis la racine du dépôt :

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

[Guide / 使用指南](docs/user-guide.md) · [FAQ](docs/faq.md) · [Signaler un problème](https://github.com/willmove/markion/issues) · [Publication des versions](docs/release-process.md) · [Licence MIT](LICENSE)
