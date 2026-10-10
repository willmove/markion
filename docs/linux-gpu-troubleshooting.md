# Linux graphics troubleshooting

Markion draws its window with Vulkan (through GPUI's blade renderer). On Linux
it needs a working Vulkan driver at startup: a hardware driver for your GPU, or
Mesa's software driver (lavapipe / llvmpipe) on machines without one.

If no usable driver is found, Markion prints a message on stderr, shows a
native dialog when `zenity`, `kdialog` or `xmessage` is available, and exits
with status 1. This guide explains what the message means and how to fix it.

## The AppImage includes a software Vulkan driver

The Linux AppImage ships Mesa's lavapipe driver, a Vulkan loader and the few
libraries they need in `usr/lib/markion-vulkan/` inside the image (about
20 MB of the download). It is only a fallback:

1. Markion always starts with the system Vulkan stack first, so a real GPU
   driver is used whenever one works.
2. If GPU initialization fails, Markion restarts itself once with the bundled
   driver (`llvmpipe` in the log). The window works normally; animations and
   scrolling use the CPU and can feel less smooth on slow machines.
3. If the bundled driver also fails, the error dialog says so and Markion
   exits.

The `.deb`, `.rpm` and source builds do not bundle a driver; install one from
your distribution as described below.

Two environment variables control the fallback:

| Variable | Effect |
| --- | --- |
| `MARKION_SOFTWARE_VULKAN=1` | Skip the system Vulkan stack and use the bundled software driver right away. |
| `MARKION_SOFTWARE_VULKAN=0` | Never use the bundled driver; show the error instead. |

For example:

```sh
MARKION_SOFTWARE_VULKAN=1 ./Markion_x86_64.AppImage
```

## Check what Vulkan sees

```sh
vulkaninfo --summary          # package: vulkan-tools
ls /usr/share/vulkan/icd.d/ /etc/vulkan/icd.d/ 2>/dev/null
```

Markion's log also records the adapter it picked. Look for a line like
`Adapter: "llvmpipe (LLVM 15.0.7, 256 bits)"` or the name of your GPU. Logs are
in `~/.cache/markion/logs/` (or `$XDG_CACHE_HOME/markion/logs/`; the dialog
shows the exact directory).

## Cause 1: no Vulkan driver installed

Typical on virtual machines, cloud desktops, CI runners and minimal installs.
The panic text contains `NoSupportedDeviceFound` or
`Instance extension "VK_KHR_surface" is not supported`.

Install your GPU's Vulkan driver, or Mesa's software driver:

```sh
sudo apt install mesa-vulkan-drivers     # Debian, Ubuntu
sudo dnf install mesa-vulkan-drivers     # Fedora
sudo pacman -S vulkan-swrast             # Arch
```

To use one specific driver regardless of what else is installed, point the
Vulkan loader at its manifest:

```sh
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json markion
# Older Vulkan loaders only understand the previous name:
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json markion
```

`VK_LOADER_DRIVERS_DISABLE` and `VK_LOADER_LAYERS_DISABLE`, used below, need
Vulkan loader 1.3.234 or newer (Ubuntu 22.04 ships 1.3.204; check with
`vulkaninfo --summary`). On older loaders, use `VK_DRIVER_FILES` /
`VK_ICD_FILENAMES` to select a driver instead.

## Cause 2: a stale driver manifest

A leftover manifest from a removed driver (often NVIDIA after switching GPUs or
uninstalling the proprietary driver) can make the loader fail before it reaches
a working one. Find manifests whose `library_path` no longer exists:

```sh
grep -H library_path /usr/share/vulkan/icd.d/*.json /etc/vulkan/icd.d/*.json
```

Remove the stale package, or disable the manifest for one run:

```sh
VK_LOADER_DRIVERS_DISABLE='nvidia*' markion
```

## Cause 3: a crashing implicit layer

Implicit layers load into every Vulkan application. Mesa's `device_select`
layer has crashed on some setups without a GPU. Disable implicit layers for one
run to check:

```sh
VK_LOADER_LAYERS_DISABLE='~implicit~' markion
```

If that fixes it, disable only the offending layer (for example
`VK_LOADER_LAYERS_DISABLE=VK_LAYER_MESA_device_select`) or set
`NODEVICE_SELECT=1`. The AppImage's software fallback uses its own loader and
sets both, so it is not affected by broken host layers.

## Reproducing the error dialog

Set `MARKION_SIMULATE_RENDERER_FAILURE=1` to make Markion fail as if GPU
initialization had failed, without touching your drivers. This works in every
build and on every platform, and is how the dialog and its translations are
tested:

```sh
MARKION_SIMULATE_RENDERER_FAILURE=1 MARKION_SOFTWARE_VULKAN=0 markion
```

Without `MARKION_SOFTWARE_VULKAN=0`, the AppImage first retries with its
bundled driver; the simulated failure repeats there, and the dialog then also
reports that the bundled driver could not start.

## Reporting a problem

If none of this helps, open an issue at
<https://github.com/willmove/markion/issues> with:

- your distribution and desktop session (X11 or Wayland),
- the output of `vulkaninfo --summary`,
- the latest file from Markion's log directory.
