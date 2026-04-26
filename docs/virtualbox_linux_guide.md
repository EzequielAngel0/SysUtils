# Guía de Automatización en Segundo Plano con VirtualBox y ESP32 (Linux / Minecraft Java)

Esta guía te explicará paso a paso cómo configurar una Máquina Virtual (VM) con **Linux** para ejecutar **SysUtils** y juegos como **Minecraft Java** de manera aislada. Linux consume muchos menos recursos de fondo que Windows, por lo que juegos nativos de Java como Minecraft suelen funcionar con más FPS en una VM Linux.

---

## 1. Descargas Previas

1. **VirtualBox:** Descarga e instala [VirtualBox](https://www.virtualbox.org/wiki/Downloads) en tu PC principal (Windows).
2. **VirtualBox Extension Pack:** Descarga el "VirtualBox Extension Pack" e instálalo (es **crucial** para el USB 3.0 hacia el ESP32).
3. **Imagen ISO de Linux:** 
   > [!TIP]
   > Te recomiendo descargar **Ubuntu Desktop** o **Linux Mint**. Son distribuciones amigables, fáciles de instalar y perfectas para ejecutar Minecraft Java.

---

## 2. Creación y Configuración de la Máquina Virtual

### A. Creación Básica
1. Abre VirtualBox y haz clic en **Nueva**.
2. Dale un nombre (ej. `Ubuntu_MC`), selecciona la ISO de Linux que descargaste.
3. Marca la casilla **Omitir instalación desatendida** (Skip Unattended Installation).

### B. Asignación de Recursos
> [!IMPORTANT]
> Minecraft Java requiere buena RAM y CPU.

1. **Memoria Base (RAM):** Asigna al menos **6144 MB (6 GB) a 8192 MB (8 GB)**. 
2. **Procesadores:** Asigna la **mitad de tus núcleos físicos**. Si tu procesador tiene 8 núcleos, asígnale 4.
3. **Disco Duro:** Crea un disco virtual de al menos **40 GB - 60 GB**.

### C. Configuración de Pantalla
1. Selecciona tu VM creada y haz clic en **Configuración** ➔ **Pantalla**.
2. **Memoria de video:** Súbelo al máximo posible (128 MB).
3. **Aceleración 3D:** Marca la casilla **Habilitar aceleración 3D**.

---

## 3. Instalación del Sistema Operativo
1. Inicia la Máquina Virtual.
2. Sigue los pasos de instalación de Ubuntu o Linux Mint (es un proceso muy visual y rápido).
3. Una vez en el escritorio de Linux, abre una terminal y actualiza el sistema:
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```
4. Instala las Guest Additions desde VirtualBox: **Dispositivos ➔ Insertar imagen de CD de las "Guest Additions"**. Ejecuta el script de instalación que aparece en Linux y reinicia la VM.

---

## 4. Redirección del USB (El ESP32)

Para que el ESP32 interactúe con la VM y no con tu computadora física:

1. Con la VM **apagada**, conecta tu ESP32 por USB a tu PC física.
2. Ve a la **Configuración** de tu VM en VirtualBox ➔ **USB**.
3. Selecciona **Controlador USB 3.0 (xHCI)**.
4. Haz clic en el ícono del cable USB con el símbolo `+` y selecciona tu placa ESP32 (*Silicon Labs CP210x*, *CH340*, etc.).
5. Haz clic en **Aceptar**.

> [!WARNING]
> En Linux, los puertos seriales (USB) requieren permisos especiales. Una vez dentro de Linux, abre una terminal y ejecuta este comando para darte permisos de lectura/escritura al USB:
> `sudo usermod -a -G dialout $USER`
> Luego, **reinicia tu sesión** (o reinicia la VM completa).

### ¿Cómo probar que Linux lo detecta como Mouse/Teclado físico?
1. En Linux, el hardware funciona automáticamente. Para comprobarlo, abre una Terminal.
2. Ejecuta el comando `xinput list`.
3. Busca en la lista algo que diga "Keyboard" y "Pointer" que coincida con el momento en que conectas el ESP32. Linux crea interfaces HID virtuales automáticamente en cuanto se enchufa.
4. Para la prueba de oro: Abre un documento de texto en Linux, envía un comando desde SysUtils, y verás cómo se escribe o se mueve el ratón en la pantalla de Linux.

---

## 5. Red, Cambio de IP y VPN (Aislamiento)

Al ser una máquina virtual, funciona como una computadora físicamente separada de tu Windows:

* **¿Tiene una IP diferente?** Por defecto (modo NAT), Linux comparte la IP pública de tu Windows. Si cambias la configuración de red de la VM a **"Adaptador Puente"**, tu router le dará a Linux su propia IP (ej. `192.168.1.80`), operando como una PC separada en tu casa.
* **¿Puedo conectarme con una VPN?** **Sí, y es sumamente recomendado.** Puedes instalar un cliente VPN en tu Linux virtual (casi todos los servicios de VPN tienen apps para Ubuntu/Mint o archivos OpenVPN).
  1. Al conectar la VPN dentro de la VM, **sólo el tráfico de Minecraft en Linux** usará esa IP extranjera.
  2. Tu Windows anfitrión seguirá con tu internet normal.
  3. Esto es el método definitivo para evitar bans por IP (IP tracking), ya que tu actividad de automatización en Linux y tu actividad personal en Windows usan direcciones IP totalmente distintas.

---

## 6. Compilar SysUtils en Linux

> [!IMPORTANT]
> El archivo `.exe` que usas en Windows **no funciona de forma nativa en Linux**. Para usar SysUtils en tu VM de Linux, debes compilar el código fuente en Rust directamente allí.

1. Instala los requerimientos en la terminal de Linux:
   ```bash
   sudo apt update
   sudo apt install curl build-essential pkg-config libssl-dev libgtk-3-dev
   ```
2. Instala el lenguaje de programación Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   *(Cierra y vuelve a abrir la terminal después de esto)*
3. Pasa **toda la carpeta del código fuente** (`phantom_native`) de tu Windows a la VM Linux usando Carpetas Compartidas o subiéndola a Google Drive.
4. Abre la terminal en esa carpeta y compila la aplicación:
   ```bash
   cargo build --release
   ```
5. Una vez termine, puedes ejecutar el binario resultante:
   ```bash
   ./target/release/sysutils_native
   ```

---

## 6. Flujo de Trabajo (Día a Día)

1. Abres VirtualBox e inicias tu VM Linux.
2. Abres Minecraft Java en Linux.
3. Abres tu terminal y ejecutas `./target/release/sysutils_native`.
4. Conectas la interfaz gráfica al puerto USB del ESP32.
5. Inicias tus rutinas/workflows de farmeo o automatización.
6. Minimizas VirtualBox en tu Windows. Todo sucederá en la VM y tu PC principal estará libre.
