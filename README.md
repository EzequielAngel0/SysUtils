# 🛠️ SysUtils (Hardware Automation & Diagnostics)

**SysUtils** es una suite de automatización de hardware y diagnóstico de alto rendimiento construida nativamente en Rust. Utiliza un microcontrolador físico (ESP32-S3) para emular señales de hardware HID (Teclado y Ratón) a nivel físico, logrando una automatización indetectable para sistemas antitrampas (Anti-Cheats).

El proyecto está diseñado para ejecutarse en entornos aislados mediante virtualización (VirtualBox), lo que garantiza que la lógica de automatización y el sistema objetivo se mantengan completamente separados del host principal, maximizando la seguridad y la privacidad mediante el enrutamiento VPN.

---

## 🎯 Características Principales

- **Emulación a Nivel Hardware:** Utiliza un ESP32-S3 físico conectado por USB nativo para actuar como un Teclado y Ratón independientes.
- **Backend Nativo y Rápido:** Interfaz y motor rescritos completamente en Rust (`egui`/`eframe`) para una ejecución en tiempo real y sin sobrecarga en la CPU.
- **Aislamiento por Máquina Virtual (VM):** Operación diseñada para ejecutarse dentro de VirtualBox (Windows o Linux), capturando el dispositivo USB y aislando la automatización del PC físico.
- **Módulos de Diagnóstico y Trabajo:**
  - **Workflows (Secuencias):** Grabación nativa de eventos del sistema para replicar tareas a la perfección.
  - **Scheduler (Módulo Pulse):** Bucles de tareas y automatizaciones programadas (Auto-Clickers, pulsadores de teclas).
  - **Background (Módulo KeepAlive):** Movimientos aleatorios y acciones para mantener sesiones activas.
  - **Diagnostics (Monitor):** Visión artificial avanzada para rastreo de píxeles, colores y cambios en la pantalla.
  - **Security (Panic):** Botón de emergencia para detener instantáneamente todas las operaciones y hardware.
- **Evasión de Bans por IP (VPN):** Arquitectura pensada para soportar VPNs instaladas directamente en la VM, separando tu IP pública de automatización de tu IP personal.

---

## 🚀 Arquitectura del Proyecto

El repositorio está dividido en dos partes fundamentales:

1. **`phantom_firmware/` (Firmware del Hardware):** El código en C++ para flashear el ESP32-S3. Utiliza librerías nativas de USB HID para registrarse ante el sistema operativo como un periférico real.
2. **`phantom_native/` (Software SysUtils):** El cliente de escritorio en Rust que actúa como centro de control. Se comunica con el ESP32 mediante puerto Serial (`/dev/ttyACM0` en Linux o `COMx` en Windows) enviando comandos encriptados o en texto plano.

---

## 📖 Guías de Instalación y Uso

Dado que SysUtils está diseñado para el máximo sigilo y rendimiento, la instalación depende del entorno que elijas para tu Máquina Virtual. Revisa la documentación correspondiente en la carpeta `docs/`:

1. **[Guía de Virtualización para Windows](docs/virtualbox_windows_guide.md)**
   Aprende a configurar una VM de Windows, inyectar el ESP32 y enrutar tu VPN.
2. **[Guía de Virtualización para Linux (Recomendado para Minecraft Java)](docs/virtualbox_linux_guide.md)**
   Linux consume muchos menos recursos y es ideal para automatizar juegos basados en Java. Incluye instrucciones para compilar SysUtils desde cero en Linux.
3. **[Guía de Configuración del Hardware ESP32](docs/HARDWARE_SETUP.md)** *(Asegúrate de leerla antes de conectar la placa)*

---

## 📦 Compilación y Despliegue Rápido (Solo Windows Host)

Si deseas probar el software en tu PC físico (Windows) antes de pasarlo a la VM:

1. Asegúrate de tener **Rust** instalado (`rustup.rs`).
2. Ejecuta el archivo **`SysUtils Manager.bat`** ubicado en la raíz del proyecto.
3. Selecciona la opción `[2] Instalar`.

El script compilará automáticamente el núcleo en modo `release` utilizando la máxima optimización de tu procesador (`target-cpu=native`), colocará el ejecutable de forma segura en tu carpeta `AppData` y creará accesos directos discretos.

---

## 🛠️ Contribución y Desarrollo

- Se recomienda utilizar `cargo run` dentro de la carpeta `phantom_native` para pruebas de desarrollo.
- Consulta el archivo `docs/roadmap.md` para ver los futuros planes de implementación (Multi-Threading en el ESP32, OCR, Cadenas de acciones complejas).

---

*Disclaimer: Esta herramienta ha sido desarrollada con fines educativos para la investigación del comportamiento de dispositivos HID, sistemas de virtualización y programación de sistemas en Rust. Los autores no se hacen responsables del uso de este software en entornos, plataformas o juegos que prohíban la automatización.*
