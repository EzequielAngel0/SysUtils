# Guía de Automatización en Segundo Plano con VirtualBox y ESP32 (Windows)

Esta guía te explicará paso a paso cómo configurar una Máquina Virtual (VM) con **Windows** para ejecutar **SysUtils** (tu herramienta de automatización) y tu aplicación objetivo de manera totalmente aislada. Esto permitirá que el ESP32 interactúe libremente con la VM, mientras tú sigues usando tu PC física sin interrupciones.

---

## 1. Descargas Previas

1. **VirtualBox:** Descarga e instala [VirtualBox](https://www.virtualbox.org/wiki/Downloads) para Windows.
2. **VirtualBox Extension Pack:** En la misma página, descarga el "VirtualBox Extension Pack" e instálalo (es **crucial** para que funcione el paso a través de USB 2.0/3.0 hacia el ESP32).
3. **Imagen ISO de Windows:** 
   > [!TIP]
   > Te recomiendo usar **Windows 10** (preferiblemente una versión ligera como Windows 10 LTSC). Evita Windows 11 para la VM ya que consume muchos más recursos (RAM y CPU).

---

## 2. Creación y Configuración de la Máquina Virtual

### A. Creación Básica
1. Abre VirtualBox y haz clic en **Nueva**.
2. Dale un nombre (ej. `Win10_Auto`), selecciona la ISO de Windows 10 que descargaste.
3. Marca la casilla **Omitir instalación desatendida** (Skip Unattended Installation) para hacerlo manualmente y tener más control.

### B. Asignación de Recursos
> [!IMPORTANT]
> El juego o aplicación objetivo debe ejecutarse fluidamente en la VM, de lo contrario, los tiempos de procesamiento de imágenes del monitor (Diagnostics) fallarán por lentitud.

1. **Memoria Base (RAM):** Asigna al menos **8192 MB (8 GB)**. Si tu PC tiene 32GB, puedes darle 12GB o 16GB.
2. **Procesadores:** Asigna la **mitad de tus núcleos físicos**. Si tu procesador tiene 8 núcleos, asígnale 4.
3. **Disco Duro:** Crea un disco virtual de al menos **60 GB - 80 GB** (o más dependiendo del peso del juego).

### C. Configuración de Pantalla
1. Selecciona tu VM creada y haz clic en **Configuración** ➔ **Pantalla**.
2. **Memoria de video:** Súbelo al máximo posible (128 MB o 256 MB si lo permite).
3. **Aceleración 3D:** Marca la casilla **Habilitar aceleración 3D**.

---

## 3. Instalación del Sistema Operativo
1. Inicia la Máquina Virtual.
2. Sigue los pasos normales de instalación de Windows. 
3. Una vez en el escritorio de la VM, ve al menú superior de VirtualBox: **Dispositivos ➔ Insertar imagen de CD de las "Guest Additions"**.
4. Ve al explorador de archivos en la VM, abre la unidad de CD e instala las Guest Additions. Reinicia la VM. Esto mejorará drásticamente la resolución y fluidez.

> [!WARNING]
> Si el juego/aplicación tiene un Anti-Cheat muy agresivo (ej. Vanguard, BattlEye, EasyAntiCheat), es posible que detecte que está corriendo dentro de VirtualBox y no te deje abrir el juego. Si esto pasa, requerirías técnicas avanzadas de "VM Cloaking" (Ocultamiento de VM).

---

## 4. Redirección del USB (El paso más importante)

Para que el ESP32 interactúe con la VM y no con tu computadora física, debemos decirle a VirtualBox que capture el ESP32.

1. Con la VM **apagada**, conecta tu ESP32 por USB a tu PC física.
2. Ve a la **Configuración** de tu VM ➔ **USB**.
3. Selecciona **Controlador USB 3.0 (xHCI)** (o 2.0 si tu cable/puerto es antiguo).
4. Haz clic en el ícono del cable USB con el símbolo `+` (Agregar filtro desde dispositivo).
5. Aparecerá una lista con tus dispositivos conectados. **Selecciona tu placa ESP32** (Suele aparecer como *Silicon Labs CP210x*, *CH340*, *USB Serial*, o genérico).
6. Haz clic en **Aceptar**.

A partir de ahora, cada vez que enciendas esta VM y el ESP32 esté conectado, "aparecerá" mágicamente dentro de la VM y desaparecerá de tu PC real.

### ¿Cómo probar que Windows lo detecta como Mouse/Teclado físico?
1. En la máquina virtual, presiona `Win + X` y abre el **Administrador de dispositivos**.
2. Despliega las categorías **Teclados** y **Mouse y otros dispositivos señaladores**.
3. Desconecta y vuelve a conectar el USB de tu ESP32. Verás que aparece un nuevo *"Teclado HID"* y un *"Mouse compatible con HID"*. ¡Eso confirma que el sistema operativo lo ve como hardware real!
4. Para la prueba final: Abre un Bloc de notas en la VM, usa SysUtils para enviar una tecla o clic, y verás cómo reacciona.

---

## 5. Red, Cambio de IP y VPN (Aislamiento)

Al ser una máquina virtual, funciona como una computadora físicamente separada, lo que te da un control total sobre su conexión a internet:

* **¿Tiene una IP diferente?** Por defecto (modo NAT), la VM comparte la misma IP pública que tu PC principal. Si en VirtualBox cambias la red de la VM a **"Adaptador Puente"** (Bridged), tu router le asignará una IP local completamente nueva y separada (ej. `192.168.1.50`), apareciendo como una PC distinta en tu red.
* **¿Puedo conectarme con una VPN?** **Sí, y es la mejor práctica.** Puedes instalar cualquier programa de VPN (NordVPN, ProtonVPN, etc.) **directamente dentro de la máquina virtual**. Al encender la VPN dentro de la VM:
  1. Todo el tráfico de la VM (el juego, SysUtils, etc.) saldrá con la IP que provee la VPN.
  2. Tu PC física (anfitrión) seguirá usando tu conexión a internet normal, con tu IP real.
  3. Esto aísla por completo tu cuenta de juego de tus rutinas de automatización, evitando rastreos por IP (IP tracking) y manteniendo tu PC real segura.

---

## 6. Mover tu `.exe` (SysUtils) a la Máquina Virtual

Puedes pasar el archivo compilado `.exe` de *SysUtils* fácilmente:
1. **Carpetas Compartidas (Recomendado):** Ve a Configuración ➔ Carpetas Compartidas en VirtualBox, añade tu carpeta de Windows anfitrión, marca "Autromontar" y "Hacer permanente".
2. **Arrastrar y Soltar:** Dispositivos ➔ Arrastrar y soltar ➔ Bidireccional.
3. **Nube:** Súbelo a Google Drive/Discord y descárgalo desde la VM.

---

## 6. Flujo de Trabajo (Día a Día)

1. Enciendes tu computadora física.
2. Abres VirtualBox e inicias tu VM.
3. El ESP32 se conecta automáticamente a la VM.
4. Abres tu juego/aplicación objetivo dentro de la VM.
5. Abres **SysUtils** dentro de la VM y lo conectas.
6. Inicias tu flujo de trabajo (Workflows).
7. Puedes minimizar VirtualBox, o arrastrar la ventana a tu segundo monitor.
8. ¡Listo! Todo sucede dentro de esa "burbuja". Tu PC real está libre.
