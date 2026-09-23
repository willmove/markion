<a id="spanish"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Logotipo de Markion" width="128" height="128">
</p>

<p align="center">
  <a href="README.md#english">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <strong>Español</strong>
</p>

# Markion

Markion es un editor Markdown de escritorio nativo construido con Rust y GPUI. Combina edición de código fuente ágil, un modo de edición visual respaldado por el código fuente, vista previa en vivo, herramientas de espacio de trabajo y exportación multiformato en una sola aplicación ligera. Markdown sigue siendo el formato canónico del documento: sin Electron, Tauri ni WebView.

## Instalación

Descarga la compilación más reciente desde [GitHub Releases](https://github.com/willmove/markion/releases).

| Plataforma | Paquetes de publicación | Destino |
|---|---|---|
| Windows | Instalador NSIS `.exe` | x86_64 |
| Linux | `.deb` y AppImage | x86_64 |
| macOS | `.app` y `.dmg` | Apple Silicon (arm64), macOS 11+ |

Las publicaciones no llevan firma de código de plataforma. Windows SmartScreen puede requerir **Más información → Ejecutar de todas formas**, y macOS Gatekeeper puede requerir hacer clic derecho en la aplicación y elegir **Abrir**. **Ayuda → Buscar actualizaciones…** ofrece un aviso de actualización accionable en todas las plataformas: las instalaciones NSIS etiquetadas de Windows x86_64 obtienen una descarga e instalación de un clic verificada criptográficamente (Minisign de cargo-packager) que se niega a iniciar mientras algún documento tenga cambios sin guardar, mientras que macOS y Linux abren el archivo de publicación correspondiente en el navegador del sistema. Los Mac Intel pueden ejecutar la compilación arm64 mediante Rosetta; no se ofrecen actualmente un binario universal ni la notarización de Apple.

## Modos de edición

Markion tiene cuatro modos de vista. La vista previa dividida es el modo predeterminado.

- **Edición** — un editor de código fuente Markdown en bruto y concentrado.
- **Edición visual** — una superficie ante todo WYSIWYG, respaldada por el código fuente. El texto en prosa permanece renderizado con revelación progresiva de la sintaxis; el contenido de los bloques de código con cercas ordinarios, las fórmulas matemáticas en bloque, los diagramas registrados (Mermaid), los campos de imagen en línea y las celdas de tablas GFM tienen editores directos exactos. Las construcciones cuya representación WYSIWYG aún no está implementada —front matter YAML, código indentado y sintaxis malformada o ambigua a nivel de bytes— conservan una vía de edición exacta respaldada por el código fuente como medida transitoria, registrada en la [matriz de cobertura WYSIWYG y hoja de ruta de la edición visual](docs/visual-editing-quality.md). Una paleta de comandos de barra y un menú contextual de bloque compacto con clic derecho ofrecen transformaciones exactas de bloque (párrafo, encabezados, listas, cita, código con cercas, separador, tabla), duplicar, subir/bajar, reordenación por arrastre segura para el código fuente y eliminar; una barra de formato contextual a la selección y un editor visual de enlaces realizan exactamente una mutación respaldada por el código fuente por acción. Los delimitadores Markdown se emparejan automáticamente de forma predeterminada, `:shortcode` abre la autocompletación de emoji, las casillas de listas de tareas se alternan directamente y, al salir de una fila de encabezado, lista, tarea o cita, esta vuelve de inmediato a su forma renderizada. No es un modelo de documento de texto enriquecido independiente: el Markdown subyacente es siempre la fuente de verdad.
- **Vista previa dividida** — código fuente y vista previa renderizada lado a lado, con un ajuste opcional de desplazamiento sincronizado mapeado al código fuente que mantiene ambos paneles en la misma ubicación del documento en lugar de desplazarse por porcentaje del documento completo.
- **Lectura** — una vista renderizada, no editable, centrada en un ancho máximo legible de 860 px de forma predeterminada; el ancho adaptativo de vista previa puede usar todo el panel.

Cambiar de modo conserva el documento activo, el cursor y la selección, el historial de deshacer y el estado de desplazamiento de cada pestaña.

El comando **Ver → Código fuente/Vista previa dividida** (`Ctrl+/` en Windows/Linux, `Cmd+/` en macOS) alterna los dos diseños orientados al código fuente. Desde la edición visual o la lectura, su primera invocación entra en el modo de edición.

## Documentos y espacio de trabajo

- Edición multipestaña con cursor, selección, desplazamiento, deshacer/rehacer, vista previa, esquema y estado Markdown derivado en caché por pestaña.
- Abrir un archivo Markdown o de texto plano ya abierto enfoca su pestaña existente en lugar de crear un duplicado.
- **Abrir carpeta** cambia la raíz del espacio de trabajo y llena la barra lateral Archivos con archivos Markdown, un conjunto seleccionado de archivos de texto plano (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc`) y los archivos de imagen admitidos (`.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`/`.tiff`, `.svg`), anidados bajo sus carpetas; las carpetas vacías también se listan. Markdown se distingue visualmente, los archivos de texto plano se abren como texto UTF-8 y las imágenes se abren como pestañas de imagen de solo lectura que ajustan las imágenes demasiado grandes al área de contenido.
- Al expandir una carpeta se revela exactamente un nivel de elementos secundarios, de modo que los espacios de trabajo muy anidados pueden explorarse nivel a nivel.
- La preferencia **Mostrar archivos ocultos** (desactivada de forma predeterminada) revela las entradas de punto (dotfiles) además del atributo de oculto de Windows, mientras que el ruido de compilación, dependencias y VCS siempre excluido (`target`, `node_modules`, `.git`, …) permanece oculto en cualquier caso.
- Los menús contextuales del árbol de archivos ofrecen, según corresponda: abrir, abrir en pestaña nueva, crear archivo/carpeta, renombrar, eliminar, revelar en el gestor de archivos del sistema, filtrar y actualizar.
- Los archivos y carpetas pueden nombrarse en el propio árbol; eliminar una carpeta no vacía requiere una confirmación adicional.
- Se pueden arrastrar archivos Markdown desde el gestor de archivos del sistema operativo hasta Markion.
- Los paneles Archivos y Esquema se pueden mostrar u ocultar, y los separadores de la barra lateral y del panel dividido se pueden arrastrar.
- El panel Esquema lista la jerarquía de encabezados del documento como un árbol plegable: cada encabezado con descendientes expone un control de expansión, los esquemas comienzan totalmente expandidos, el plegado es por documento y solo dura la sesión, y se resalta la sección que contiene el cursor. Al hacer clic en un encabezado se salta a su posición en el código fuente —o al encabezado renderizado en el modo lectura.
- El título nativo de la ventana muestra el nombre del archivo activo tras la marca Markion, con un sufijo `*` cuando el documento tiene cambios sin guardar. La barra de estado conserva el estado de guardado y la retroalimentación transitoria de operaciones, además de un contexto persistente compacto: el recuento de caracteres y palabras del documento activo, la línea y columna del cursor (empezando en 1) cuando hay una superficie de edición, y la rama Git actual cuando el documento o el espacio de trabajo pertenece a un repositorio.
- **Copia y sincronización** mantiene una carpeta de notas protegida en una ubicación de sincronización con un flujo **Sincronizar ahora** de un clic y estados en lenguaje sencillo. Git proporciona debajo el historial de versiones seguro; los controles técnicos permanecen en los detalles avanzados de Git. Consulta la [guía de copia y sincronización](docs/git-sync.md) para la configuración, la autenticación, los conflictos y la recuperación.

## Edición y vista previa de Markdown

- El análisis corre a cargo de `pulldown-cmark` con soporte orientado a CommonMark y GFM.
- Los comandos de formato cubren párrafo (`Ctrl+0` en Windows/Linux, `Cmd+0` en macOS), encabezados, negrita, cursiva, código en línea, enlaces, imágenes, listas, listas de tareas, citas, bloques de código con cercas y tablas Markdown en el código fuente. «Párrafo» convierte los encabezados ATX intersectados de vuelta a texto normal sin cambiar las líneas que no son encabezado.
- El autopareado de Markdown está activado de forma predeterminada en el código fuente y en los campos compatibles de la edición visual: los delimitadores se pueden emparejar, las selecciones envolver, los cierres existentes saltar y los pares vacíos eliminar con Retroceso. Permanece desactivado durante la composición IME y dentro de los editores de contenido de código, fórmulas en bloque y diagramas.
- Escribir un `:shortcode` abierto en un límite de palabra abre la autocompletación de emoji en el código fuente y la edición visual; al confirmar se inserta Markdown canónico que sigue siendo una única edición de código fuente deshacible.
- Pegar texto enriquecido copiado desde Word, Google Docs o páginas web convierte la variante HTML del portapapeles en Markdown con formato —encabezados, énfasis, enlaces, listas anidadas, tablas, código— como un único paso deshacible; los portapapeles de texto plano se pegan literalmente.
- Los comandos de encabezado exponen H1–H5 de forma predeterminada, con una opción H1–H6 en Preferencias.
- Buscar y reemplazar admite distinción de mayúsculas y minúsculas, expresiones regulares, navegación siguiente/anterior, reemplazar la coincidencia actual y reemplazar todo.
- Los comandos de tabla en el código fuente pueden formatear tablas y añadir, eliminar o mover filas y columnas. Las tablas de la edición visual además ofrecen edición directa de celdas respaldada por el código fuente, recorrido con Tab, reajuste de ancho determinista y las mismas operaciones de filas/columnas; las tablas ordinarias de la vista previa permanecen de solo lectura.
- Un flujo de trabajo de recursos de imagen ingiere imágenes del portapapeles, archivos arrastrados, inserción explícita por archivo/URL e imágenes ya presentes en el documento. Las políticas pueden conservar, copiar/descargar o subir mediante PicGo HTTP, PicGo Core o un comando personalizado literal; consulta la [guía de gestión y subida de imágenes](docs/image-handling.md). Markion utiliza recursos relativos al documento resistentes a colisiones, conserva con exactitud los metadatos de las imágenes y mantiene la recuperación de transferencias fuera del Markdown.
- El front matter YAML se analiza y se oculta de la vista previa; `title`, `author` y `date` alimentan los metadatos de exportación.
- Las escrituras de documentos son reemplazos atómicos en el mismo directorio que conservan la ruta existente y el estado de modificación cuando una escritura falla. Markion rastrea la última identidad conocida del archivo en disco, detecta cambios externos antes de guardar y mientras el documento está abierto, recarga automáticamente solo los documentos limpios y ofrece a los documentos modificados una elección explícita de conflicto entre recargar, sobrescribir o guardar una copia. El gestor de recuperación inventaría cada instantánea de recuperación con su ruta original y su relación con el disco, y admite Restaurar, Descartar, Restaurar todo y Descartar todo sin eliminar datos ilegibles o no seleccionados.
- El autoguardado se ejecuta de forma predeterminada tras cinco segundos de inactividad y escribe copias de recuperación para los documentos sin guardar; las instantáneas restauradas permanecen duraderas hasta un guardado correcto, un descarte explícito o que un sucesor escrito atómicamente las reemplace.

La vista previa renderizada admite:

- Negrita, cursiva, tachado, código en línea, enlaces, resaltado, superíndice, subíndice, notas al pie (pasa el cursor sobre una referencia para leer su definición), listas de tareas, códigos cortos de emoji habituales y enlaces automáticos.
- Los tokens `[TOC]` / `[toc]` dentro del documento se renderizan como un esquema dinámico y clicable, y los saltos de encabezado `[text](#heading)` / `{#id}` permanecen dentro del documento.
- Números de inicio correctos en listas ordenadas, listas anidadas, viñetas por profundidad, sangría francesa, imágenes y HTML incrustado.
- El HTML en línea admitido tiene una semántica coherente en Markdown mixto, bloques HTML independientes, celdas de tabla y edición visual, incluidos spans de color seguros, enlaces e imágenes enlazadas, `kbd`/`samp`, saltos de línea creados y superíndices/subíndices correctamente posicionados; el marcado malformado o no admitido recurre a una alternativa sin ejecutar scripts.
- Texto de vista previa seleccionable con menú contextual para copiar como texto plano, Markdown o HTML, además de copiar la dirección del enlace cuando corresponde.
- Fórmulas `$...$` en línea y `$$...$$` en bloque, compuestas sin conexión en SVG en caché por un motor RaTeX integrado (compatible con KaTeX) con fuentes incluidas: sin acceso a la red ni instalación externa de LaTeX.
- Diagramas con cercas ` ```mermaid ` (diagramas de flujo, de secuencia y más) renderizados a SVG saneado y en caché.
- Código con cercas resaltado sintácticamente mediante syntect y el conjunto de gramáticas ampliado two-face, con un analizador léxico de reserva y números de línea opcionales.

## Temas, idiomas y preferencias

- Catorce temas integrados: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark y Tokyo Night/Light.
- Los temas personalizados usan archivos `.toml` en el directorio local de temas de Markion. En el primer uso se instala allí un ejemplo `typewriter.toml` —que incluye la tabla opcional `[fonts]` (`editor`, `rendered`, `code`) que aporta familias tipográficas para el editor de código fuente Markdown, el texto renderizado y las superficies de código cuando el usuario no tiene una preferencia explícita— como punto de partida. Los archivos `.theme` antiguos migran automáticamente al cargarse por primera vez.
- Siete idiomas de interfaz: inglés, chino simplificado, chino tradicional, japonés, francés, alemán y español.
- El panel de Preferencias de la aplicación cubre en **General**: idioma, visibilidad de la barra lateral, ancho adaptativo de vista previa, modos concentración/máquina de escribir, autopareado Markdown, números de línea de código, desplazamiento sincronizado, mostrar archivos ocultos y profundidad del menú de encabezados; y en **Apariencia**: tema, familias tipográficas por plano (código fuente, lectura, código), tamaños de fuente y espaciado de párrafos.
- Las preferencias persisten en `config.toml`; los archivos `preferences.conf` antiguos migran automáticamente.

Todos los campos de configuración son opcionales. Los valores predeterminados principales y los ajustes solo por archivo son:

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 o 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" o "outline"
show_hidden_files = false
markdown_auto_pair = true

# Familias tipográficas opcionales por plano; ausentes = seguir el tema y luego
# el valor integrado (fuente de UI del sistema para código/lectura, JetBrains Mono para código).
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

La configuración, los archivos de recuperación, los temas y los registros de diagnóstico rotativos usan los directorios de datos de Markion apropiados para cada plataforma. Define `RUST_LOG=debug` antes de iniciar para obtener registros más detallados.

## Exportación

Markion exporta a:

- Un espacio de trabajo MarkNice local para preparar contenido enriquecido para WeChat
- Markdown
- HTML con estilos y HTML plano
- LaTeX
- DOCX
- PDF
- Instantáneas de texto PNG y JPEG

PDF y DOCX prueban primero el motor de exportación Typune/pandoc integrado. Si pandoc o el motor de PDF seleccionado no está disponible, Markion recurre a un escritor integrado más sencillo e informa del backend en la barra de estado. Instalar pandoc y un motor de PDF adecuado produce resultados más ricos. Las salidas PNG/JPEG y PDF integradas son deliberadamente instantáneas de texto básicas.

## Importación de Word

Elige **Archivo → Importar Word (.docx)** para convertir un DOCX localmente en un nuevo documento Markdown guardado. Markion muestra un informe de conversión antes del selector de guardado y exige una elección explícita cuando puede perderse contenido. Las imágenes incrustadas admitidas se guardan junto al Markdown en `<nombre>.assets/`; mantén ese directorio junto al archivo `.md`. El importador está acotado, es cancelable, funciona sin conexión y nunca sobrescribe una ruta Markdown o de recursos existente. Utiliza la vista aceptada del control de cambios, conserva con mayor fidelidad los niveles de encabezado de Word y normaliza los límites entre negrita y palabras para un Markdown interoperable. Los antiguos `.doc`, `.docm`, los archivos cifrados, el PDF/OCR y la fidelidad del diseño de página quedan fuera de este flujo de trabajo. Consulta [`docs/word-import.md`](docs/word-import.md) para conocer el subconjunto semántico admitido, los límites, los diagnósticos y el comportamiento de limpieza.

Elige **Exportar → Publicar para WeChat (MarkNice)** para abrir el documento activo en memoria en un espacio de trabajo privado de bucle local en tu navegador predeterminado. La apariencia de editor incluida sigue la sección de editor MarkNice fijada (tarjetas dobles, cabeceras de semáforo, barra de herramientas SVG, marco de teléfono de 375 px) tan fielmente como permite una envoltura local y no promocional. Temas, renderizador, composición matemática y scripts de la aplicación funcionan sin conexión. Las fórmulas se componen como SVG en línea autocontenido (MathJax), por lo que sobreviven al filtrado al pegar del editor de WeChat sin duplicación. Las ediciones del navegador permanecen en esa pestaña y nunca se escriben de vuelta en Markion. Su **Importar Word** convierte un archivo `.docx` solo dentro de esa sesión del navegador; recupéralo en Markion con **Copiar MD** o guardando el Markdown de la sesión en otro lugar. Las imágenes locales gestionadas pueden previsualizarse, pero copiar requiere omitirlas explícitamente porque un blob local no puede publicarse en WeChat; las imágenes remotas aún pueden contactar con sus hosts originales. El espacio de trabajo también puede copiar su Markdown actual exacto, guardar HTML independiente con tema, guardar un archivo Word de MarkNice generado por el navegador o **Guardar como PDF** imprimiendo la vista previa actual con tema y saneada (elige «Guardar como PDF» en el diálogo de impresión). El Word y el PDF del navegador son distintos de los exportadores nativos/Pandoc de Markion. El espacio de trabajo no importa archivos Markdown, PDF ni imágenes locales, y no tiene acción de documento de ejemplo. Si el permiso del portapapeles o la sesión de dos horas expira, concede el permiso o relanza desde Markion.

## Rendimiento

- Los bloques de vista previa, los bloques de edición visual, el esquema, las estadísticas y los recuentos de líneas se almacenan en caché por versión del documento y se comparten mediante `Arc`.
- El resaltado de sintaxis se memoiza entre ediciones, y la carga de gramáticas se precalienta en segundo plano.
- Las instantáneas de deshacer omiten las cachés derivadas, mientras que el editor reutiliza un identificador de texto en caché por versión.
- Las listas de vista previa/edición visual actualizan solo los rangos modificados, el árbol de archivos renderiza un conjunto acotado de filas y las líneas de código fuente ajustadas miden su altura renderizada real.

La edición visual mapeada al código fuente reutiliza incrementalmente las regiones analizables de forma independiente tras ediciones localizadas y recurre a una derivación completa cuando el contexto Markdown o los rangos de bytes son inciertos. La derivación de la vista previa dividida/lectura sigue siendo antirrebote y en caché. Markion todavía usa un búfer `String` en lugar de una rope, y algunas lecturas semánticas requieren intencionadamente un análisis completo.

## Limitaciones actuales

- La edición visual es ante todo WYSIWYG conservando el Markdown canónico; las construcciones sin una representación exacta a nivel de bytes demostrada exponen el código fuente exacto como vía transitoria (registrado en la [hoja de ruta de cobertura WYSIWYG](docs/visual-editing-quality.md)) en lugar de aceptar una mutación conjeturada del árbol enriquecido, y la reordenación de bloques solo se ofrece cuando son demostrables límites de código fuente no solapados. Las principales carencias actuales incluyen las entidades HTML decodificadas, el front matter y los bloques de código indentados; las construcciones secundarias malformadas o no admitidas permanecen listadas en la matriz.
- La representación en pantalla (vista previa dividida/lectura y edición visual) compone las fórmulas con el motor RaTeX integrado; la exportación LaTeX conserva el código fuente nativo `$...$`/`$$...$$` para la cadena de herramientas del lector, y la vía alternativa integrada de exportación DOCX (usada solo cuando pandoc no está disponible) aún degrada las fórmulas a una aproximación de texto plano legible en lugar de incrustar glifos compuestos.
- Las celdas de tabla de la edición visual admiten edición directa además de Negrita/Cursiva/Código en línea/Enlace sobre una selección que permanece dentro de una celda. Los anchos de columna GFM se pueden arrastrar y persisten como un comentario HTML (`<!-- markion-cols:… -->`) inmediatamente antes de la tabla. Las imágenes en línea aceptan anchos enteros del 10 al 100 % y se pueden redimensionar arrastrando. Las imágenes por referencia/multilínea y las tablas malformadas siguen siendo carencias conocidas de la hoja de ruta de cobertura WYSIWYG que conservan vías de edición respaldadas por el código fuente.
- Las actualizaciones de un clic instalan la distribución NSIS de Windows tras la verificación Minisign; el reemplazo del bundle de macOS y el autorreemplazo de `.deb`/AppImage en Linux siguen siendo trabajo futuro, y la autenticación del actualizador no es Windows Authenticode ni la notarización de Apple.
- No están implementados los movimientos por arrastrar y soltar en el árbol de archivos ni una interfaz completa de instalación de temas personalizados.
- La exportación de imágenes es una instantánea de texto básica, y los documentos muy grandes aún no usan una rope ni análisis totalmente incremental en todos los subsistemas derivados.

## Desarrollo

Se requiere Rust stable. Desde la raíz del repositorio:

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

El comando de calidad comprueba el formato y los lints de Rust, la suite de pruebas completa del espacio de trabajo de Cargo, el bundle MarkNice fijado y cada artefacto OpenSpec en modo estricto. Consulta la [guía del espacio de trabajo MarkNice local](docs/marknice-workspace.md) y el [contrato de soporte e ingeniería de la edición visual](docs/visual-editing-quality.md).

El paquete raíz es el crate de aplicación `markion`. Los crates de biblioteca derivados de Typune y libres de GPUI viven en `crates/*`:

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

Un simple `cargo test` prueba solo el paquete raíz; usa `cargo test --workspace` para cada miembro. En Windows la aplicación es un ejecutable del subsistema GUI y también puede iniciarse tras una compilación de depuración con:

```powershell
.\target\debug\markion.exe
```

## Licencia

Markion está disponible bajo la [Licencia MIT](LICENSE).
