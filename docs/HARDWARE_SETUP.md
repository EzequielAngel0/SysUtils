# 🔌 Hardware Setup Guide (ESP32-S3)

Esta guía detalla cómo compilar y grabar el firmware del ESP32-S3 para que Phantom pueda operar e interactuar con tu equipo a nivel hardware puro como un Emulador HID (Device) + Serial independiente.

## 🎯 Requisitos Previos
1. **Placa**: Debe ser estrictamente un **ESP32-S3** (S2 también funciona, pero la arquitectura base está diseñada para el microcontrolador S3 ya que soporta modo USB OTG Nativo).
2. **Cable USB**: Asegúrate de usar un cable con transferencia de datos. Algunos cables baratos solo proveen voltaje.
3. **IDE de Arduino**: Descargar e instalar la última versión del [Arduino IDE](https://www.arduino.cc/en/software).

## 🛠️ Configuración del Entorno en Arduino

1. Abre Arduino IDE.
2. Ve a **Archivo > Preferencias** e ingresa la siguiente URL en el campo *"Gestor de URLs Adicionales de Tarjetas"*:
   `https://espressif.github.io/arduino-esp32/package_esp32_index.json`
3. Ve a **Herramientas > Placa > Gestor de Tarjetas**, busca `esp32` por Espressif Systems e instálalo.
4. Conecta tu ESP32-S3 al PC. Asegúrate de insertarlo por el puerto etiquetado habitualmente como **"USB"**, no por el que pone "UART" (si tu placa tiene dos).

## ⚙️ Configuración Crítica de "Herramientas (Tools)"

Este es el paso fundamental para saltarse los bloqueadores y evitar que la placa actúe como un COM port aburrido que no es reconocido como un ratón o teclado.

En el menú superior, ve a **Herramientas (Tools)** y clona estrictamente esta configuración:

- **Placa / Board:** `ESP32S3 Dev Module`
- **USB Mode:** `USB OTG (TinyUSB)` -> *ESTO ES VITAL PARA EMPLEAR HID.*
- **USB CDC On Boot:** `Enabled` -> *Establece el puente puente serial para que Rust le hable al C++*.
- **USB Firmware MSC On Boot:** `Disabled`
- **USB DFU On Boot:** `Disabled`
- **Upload Mode:** `UART0 / Hardware CDC`
- **Puerto:** Selecciona el puerto COM donde haya sido reconocida (ej. `COM4`).

## 💾 Carga del Firmware

1. Abre el archivo localizado en la raíz del proyecto correspondiente a la carpeta `phantom_firmware/phantom_firmware.ino` dentro del IDE de Arduino.
2. Pincha en compilar/verificar para asegurar que no faltan librerías nativas.
3. Pincha en **Subir (Upload)**.
   > **Nota:** Si la consola de subida emite puntos sucesivos (`Connecting...`) y falla, debes presionar simultáneamente el pequeño botón **"BOOT"** en tu plaquita mientras salen los puntos para purgar la placa, hasta que detecte conexión y comience a grabar.
4. Cuando marque `Hard resetting via RTS pin` o 100% Finalizado... **No lo toques**.

## 🔌 Verificación Funcional
Al grabar, si todo se ha compilado exitosamente bajo el estándar `USB OTG (TinyUSB)`, cuando ingreses al **Administrador de Dispositivos (Device Manager)** de Windows, verás un nuevo:
- 🐭 `Teclado HID` virtual creado mágicamente.
- ⌨️ `Mouse compatible con HID` virtual creado.
- 🔌 Un Puerto COM Serie USB de dispositivo.

¡El Hardware ahora está inyectado como un dispositivo USB real listo para ser manipulado por el Panel de Control *Phantom Native* sin alertar al PC principal!
