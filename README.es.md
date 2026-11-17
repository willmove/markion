<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>Escribe con naturalidad. Conserva tu Markdown.</strong><br>Un editor nativo para notas, documentación técnica y tu próximo artículo. Creado con Rust + GPUI.</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <strong>Español</strong></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>Descargar</strong></a> · <a href="#new-in-v061">Novedades</a> · <a href="#performance">Rendimiento</a> · <a href="docs/user-guide.md">Guía</a></p>

![Markion en Windows: Markdown renderizado, tareas y tabla entre los archivos y el esquema independiente a la derecha](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Captura real en Windows, compilación de desarrollo v0.6.0 · <a href="docs/images/README.md">Capturas y espacio de ejemplo</a></sub></p>

## Por qué elegir Markion

| ✍️ Escribe sin fricción | 📁 Mantén el control | 📤 Del borrador a la publicación |
| --- | --- | --- |
| La edición visual oculta la sintaxis según sea necesario. Cambia al código fuente o a la vista dividida cuando quieras. | Archivos Markdown locales normales, sin cuenta. Editor nativo Rust + GPUI, sin Electron ni WebView integrado. | Importa Word, escribe código, fórmulas y diagramas y exporta a PDF, Word, HTML o imágenes. |

<a id="new-in-v060"></a><a id="new-in-v061"></a>

## Novedades de v0.6.1

| Destacado | Para tu escritura |
| --- | --- |
| **Interfaz en coreano** | Ocho idiomas completos; selecciona **한국어** en Preferencias. |
| **Espaciado a tu gusto** | Interlineado del 75–300% y separación entre párrafos de 0–128px en **Preferencias → Apariencia**. |
| **Actualizaciones en segundo plano** | Comprobación cada cinco horas, descarga automática activada por defecto. Se instala tras confirmar y se protegen los cambios sin guardar. |
| **Insertar sin perder contenido** | Menú compacto y acciones para añadir tabla/separador debajo conservan el contenido; sigue escribiendo después del separador. |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Menú real de bloques de Markion: encabezados, listas, acciones y formato en línea"></p>
<p align="center"><sub>v0.6.0</sub></p>

[Notas de versión](docs/release-notes/release-notes-v0.6.1.md) · [Estado del proyecto y verificaciones](docs/project-status.md)

<a id="install"></a>

## Descarga y empieza a escribir

| Plataforma | Descargas v0.6.1 |
| --- | --- |
| **Windows x64** | [Instalador](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-setup.exe) · [ZIP portátil](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-portable.zip) |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_x64.dmg) |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x86_64.rpm) |

[Todas las versiones y verificaciones](https://github.com/willmove/markion/releases)

- **Windows:** ejecuta el instalador o extrae el ZIP. Aún no hay firma Authenticode; SmartScreen puede requerir **Más información → Ejecutar de todas formas**.
- **macOS:** arrastra Markion a Aplicaciones. **v0.6.1 tiene firma ad-hoc y no está notarizada.** Para una copia de confianza descargada por navegador, elimina la cuarentena tras instalar:

  ```bash
  xattr -cr /Applications/Markion.app
  ```

  Repite tras cada actualización descargada por navegador. [Primer inicio](docs/user-guide.md#install).
- **Linux:** permite ejecutar el AppImage (`chmod +x Markion*.AppImage`) y ábrelo, o instala DEB/RPM. Sin FUSE, usa `--appimage-extract-and-run`.

Abre un archivo o carpeta Markdown. Antes de editar los ejemplos, **copia toda la carpeta examples, incluidas las imágenes**. [Primeros pasos y plantillas](assets/examples/README.md).

## Un documento, cuatro formas de trabajar

| Modo | Ideal para |
| --- | --- |
| **Edición visual** · predeterminado | Escribir sobre Markdown renderizado y editar texto, tablas, código y fórmulas directamente. |
| **Código fuente** | Controlar toda la sintaxis Markdown. |
| **Vista previa dividida** | Fuente y vista previa juntas, con desplazamiento sincronizado opcional. |
| **Lectura** | Una vista limpia sin edición. |

Al cambiar se conservan la posición, selección e historial de deshacer. [Atajos](docs/keyboard-shortcuts.md).

| Del borrador a la entrega | Incluye |
| --- | --- |
| **Documentación técnica** | Tablas GFM, resaltado de código, fórmulas LaTeX, Mermaid, notas al pie, avisos y esquema navegable. |
| **Recuperar trabajo existente** | Importación DOCX local y pegado enriquecido desde Word, Google Docs o la web. [Importar Word](docs/word-import.md) |
| **Exportar y publicar** | Markdown, HTML con/sin estilos, LaTeX, DOCX, PDF, PNG/JPEG y espacio MarkNice local para WeChat. [Publicación](docs/marknice-workspace.md) |
| **Imágenes y recursos** | Pegar, arrastrar, copiar/descargar o subir mediante PicGo/comandos personalizados. [Guía de imágenes](docs/image-handling.md) |
| **A tu gusto** | Temas, fuentes por superficie y tipografía en **Preferencias → Apariencia**; modos enfoque/máquina de escribir y atajos personalizados. |
| **Conexiones opcionales** | Copias y sincronización con Git; AI Agent con tu clave o modelos locales. La IA está **desactivada por defecto**. [Sincronización](docs/git-sync.md) · [IA](docs/ai-agent-guide.md) |

<a id="performance"></a>

## Rendimiento que puedes comprobar

**Mediciones históricas en Windows, compilación release · 2026-11-11.** Intel Core Ultra 5 225U, ~31 GiB de RAM; 100 ediciones locales por tamaño. Compara el **procesamiento del modelo**, sin disposición/paint de la interfaz ni presentación GPU.

| Markdown mixto | Modelo inicial | Edición P50 | Edición P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

En otra prueba con **1080 eventos de teclado del sistema** sobre 1 MiB, el tiempo desde el callback de entrada hasta el paint del cursor fue de **26.63 ms P50 / 33.14 ms P95**. Excluye el transporte del hardware/OS y la presentación GPU. Son registros anteriores, no una nueva prueba de v0.6.1 ni una clasificación de competidores. [Método, datos originales y reproducción](docs/performance.md).

## Antes de empezar

- Markdown sigue siendo la fuente original. La sintaxis no compatible o ambigua a nivel de bytes usa edición fuente conservadora. No se admiten celdas GFM combinadas ni recorte de imágenes por píxeles. [Cobertura y límites](docs/visual-editing-quality.md).
- PDF/DOCX integrados no necesitan conversor externo; Pandoc es opcional y vuelve al exportador integrado si falla. La fidelidad varía según el formato. El espacio MarkNice del navegador sigue la apariencia de su versión fijada. Importar Word solo afecta a la sesión y no escribe en Markion; usa **Copy MD** o guarda Markdown explícitamente para recuperarlo. Word y PDF mediante impresión con tema del navegador difieren del exportador nativo. Ese espacio no ofrece importación Markdown/PDF/imágenes ni acciones de ejemplos.
- Las instalaciones Windows y los AppImage Linux permiten instalar actualizaciones verificadas con Minisign dentro de la app. macOS y DEB/RPM abren una descarga. La verificación de actualización es distinta de la firma de código del sistema.
- **README e interfaz en 8 idiomas**: inglés, chino simplificado/tradicional, japonés, coreano, francés, alemán y español. Selecciona **한국어** en Preferencias.

## Compilar y contribuir

Con Rust stable y las herramientas nativas de tu plataforma ([configuración CI](.github/workflows/release.yml)), ejecuta desde la raíz del repositorio:

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

[Guía / 使用指南](docs/user-guide.md) · [FAQ](docs/faq.md) · [Informar de un error](https://github.com/willmove/markion/issues) · [Proceso de publicación](docs/release-process.md) · [Licencia MIT](LICENSE)
