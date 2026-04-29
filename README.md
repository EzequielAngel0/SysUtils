# 🛠️ SysUtils

**SysUtils** es una suite de automatización de hardware y diagnóstico de alto rendimiento construida nativamente en Rust. Utiliza un microcontrolador físico (ESP32-S3) para emular señales HID (Teclado y Ratón) a nivel de hardware, logrando una automatización indetectable para sistemas antitrampas.

El proyecto está diseñado para ejecutarse en entornos aislados mediante virtualización (VirtualBox), manteniendo la lógica de automatización completamente separada del host principal.

> **Versión actual:** 1.1.0

---

## 🎯 Módulos

| Módulo | Nombre UI | Descripción |
|--------|-----------|-------------|
| **Pulse** | Scheduler | Auto-clicker y pulsador de teclas con delays aleatorios humanizados |
| **KeepAlive** | Background | Presiona teclas periódicamente para evitar detección AFK |
| **Monitor** | Diagnostics | Visión artificial: detecta cambios de color/píxel y ejecuta acciones |
| **Panic** | Security | Botón de emergencia — detiene todo al detectar cambio en pantalla |
| **Sequence** | Workflows | Graba y reproduce secuencias completas de teclado y ratón |

---

## ✨ Características

- **Emulación a nivel hardware** — ESP32-S3 físico actúa como periférico HID real (no software)
- **Backend 100% Rust** — `egui`/`eframe` para UI nativa sin overhead
- **Aislamiento por VM** — Diseñado para VirtualBox (Windows o Linux)
- **Perfiles de configuración** — Guarda y carga múltiples configuraciones con nombre
- **Preview en tiempo real** — Vista previa actualizada del Monitor cada 500ms
- **Hotkeys globales** — Soporta `Ctrl+`, `Alt+`, `Shift+` + alfanumérico
- **Detección de Anti-Cheat** — Alerta si EasyAntiCheat, BattlEye o Vanguard están activos
- **Logging persistente** — Logs diarios en `phantom_logs/`
- **Evasión de IP** — Arquitectura compatible con VPN instalada en la VM

---

## 🚀 Inicio Rápido (Windows)

### Opción 1: Script automático
```
Ejecuta: SysUtils.bat
Selecciona: [2] Instalar
```

### Opción 2: Manual
```powershell
# Requiere Rust instalado (rustup.rs)
cd phantom_native
cargo build --release
```

---

## 📁 Estructura del Proyecto

```
phantom_native/          # Software cliente (Rust)
phantom_firmware/        # Firmware ESP32-S3 (C++ Arduino)
docs/                    # Guías de instalación y hardware
```

---

## 📖 Documentación

| Documento | Descripción |
|-----------|-------------|
| [Hardware Setup](docs/HARDWARE_SETUP.md) | Configurar y flashear el ESP32-S3 |
| [Guía Windows VM](docs/virtualbox_windows_guide.md) | VM Windows + ESP32 + VPN |
| [Guía Linux VM](docs/virtualbox_linux_guide.md) | VM Linux (recomendado para Minecraft Java) |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Solución de problemas comunes |
| [Roadmap](docs/roadmap.md) | Funciones implementadas y planificadas |
| [KIRO.md](KIRO.md) | Guía técnica para desarrollo con Kiro AI |
| [CLAUDE.md](CLAUDE.md) | Guía técnica para desarrollo con Claude AI |

---

## 🔌 Protocolo ESP32

Los comandos se envían como strings ASCII por puerto serial:

```
CLK_DOWN:L|R|M        — Presionar botón del ratón
CLK_UP:L|R|M          — Soltar botón del ratón
KEY_DOWN:char         — Presionar tecla
KEY_UP:char           — Soltar tecla
MOUSE_MOVE_REL:dx:dy  — Mover ratón (coordenadas relativas)
DELAY:min:max         — Delay aleatorio
MODE:PULSE|HOLD       — Modo de operación
PING                  — Verificar conexión
```

---

## 📦 Archivos de Datos

Todos los archivos se crean junto al ejecutable:

| Archivo | Descripción |
|---------|-------------|
| `sysutils_config.json` | Configuración principal |
| `sysutils_profile_{nombre}.json` | Perfiles guardados |
| `sysutils_sequence.json` | Secuencia activa |
| `phantom_logs/` | Logs diarios |

---

## 🛠️ Desarrollo

```powershell
cd phantom_native
cargo run              # Desarrollo
cargo build --release  # Build optimizado
```

Consulta `KIRO.md` o `CLAUDE.md` para guías de desarrollo detalladas.

---

*Disclaimer: Herramienta desarrollada con fines educativos para investigación de dispositivos HID, virtualización y programación de sistemas en Rust. Los autores no se responsabilizan del uso en entornos que prohíban la automatización.*
