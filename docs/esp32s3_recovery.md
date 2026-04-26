# 🛠️ Guía de Recuperación — ESP32-S3 (Phantom Firmware)

Guía para restaurar el ESP32-S3 cuando deja de ser reconocido por Windows o el firmware queda en mal estado.

---

## 🔴 Síntomas Comunes

| Síntoma | Causa probable |
|--------|---------------|
| El puerto COM no aparece en Arduino / Python | USB CDC on Boot desactivado o driver malo |
| Windows muestra "Dispositivo desconocido" | Driver no instalado o modo USB incorrecto |
| El sketch sube pero el dispositivo no responde | USB Mode incorrecto (no es TinyUSB) |
| No se puede subir ningún sketch | Bootloader corrupto o modo de arranque forzado |

---

## 📋 Paso 1 — Verificar Configuración en Arduino IDE

Ir a **Tools (Herramientas)** y verificar cada opción para el ESP32-S3:

| Parámetro | Valor Requerido |
|-----------|----------------|
| **Board** | `ESP32S3 Dev Module` |
| **USB CDC On Boot** | ✅ `Enabled` |
| **USB Mode** | ✅ `TinyUSB` |
| **USB DFU On Boot** | `Disabled` |
| **Upload Mode** | `UART0 / Hardware CDC` |
| **Flash Size** | `4MB` (ajustar según tu módulo) |
| **Partition Scheme** | `Default 4MB` (o la que corresponda) |
| **PSRAM** | `OPI PSRAM` (si el módulo lo tiene) |

> [!IMPORTANT]
> Si **USB CDC On Boot** está en `Disabled`, el ESP32-S3 no creará un puerto COM virtual sobre USB. El PC no verá nada.

> [!WARNING]
> Si **USB Mode** no es `TinyUSB`, la comunicación HID y serial puede fallar silenciosamente. Esta es la causa más común de que el Phantom no envíe comandos al PC.

---

## 🖥️ Paso 2 — Verificar Drivers en Windows

### 2.1 Abrir el Administrador de Dispositivos

```
Win + X  →  Administrador de Dispositivos
```
O desde ejecutar (`Win + R`):
```
devmgmt.msc
```

### 2.2 Qué buscar

Dependiendo del estado del ESP32-S3, aparecerá en distintas secciones:

| Dónde aparece | Qué significa |
|--------------|--------------|
| **Puertos (COM y LPT)** → `USB Serial Device (COMx)` | ✅ Todo bien, usa este puerto |
| **Puertos (COM y LPT)** → `Silicon Labs CP210x` | Módulo con chip UART externo — instalar driver CP210x |
| **Dispositivos desconocidos** → ⚠️ `USB Serial (desconocido)` | Driver de CDC no instalado |
| **Controladores de bus serie universal** → `Espressif USB JTAG/serial debug unit` | Modo JTAG — normal para flasheo |
| **Dispositivos de interfaz humana (HID)** | El ESP32-S3 está en modo HID (correcto para Phantom) |

### 2.3 Instalar/reinstalar driver CDC

Si aparece como dispositivo desconocido:

1. Clic derecho sobre el dispositivo desconocido → **Actualizar controlador**
2. Seleccionar **"Buscar controladores en mi equipo"**
3. Ir a la carpeta de controladores de Arduino:
   ```
   C:\Users\<Usuario>\AppData\Local\Arduino15\packages\esp32\tools\
   ```
   O instalar manualmente desde el sitio de Espressif:
   - [Zadig (recomendado para WinUSB/CDC)](https://zadig.akeo.ie/)

### 2.4 Usar Zadig para forzar driver correcto

1. Descargar y abrir **Zadig**
2. Ver el menú **Options → List All Devices**
3. Seleccionar `ESP32-S3 (Interface 0)` o `USB Serial (CDC)`
4. En el selector de driver, elegir **`usbser` (USB Serial)** o **`WinUSB`**
5. Clic en **"Install Driver"** o **"Reinstall Driver"**

> [!TIP]
> Para el Phantom: el ESP32-S3 aparecerá como **HID** (teclado/mouse virtual). No necesita driver adicional para HID — Windows lo reconoce nativamente. Solo necesita driver para el puerto COM Serial.

---

## 🔧 Paso 3 — Entrar a Modo Bootloader (si no se puede subir código)

Si el ESP32-S3 está completamente bloqueado y no acepta uploads:

### Método A — Botones físicos (modo DFU manual)

1. Mantener presionado el botón **BOOT** (GPIO0)
2. Presionar y soltar el botón **RESET** (EN)
3. Soltar el botón **BOOT**

El dispositivo entrará en modo **USB DFU Bootloader** y aparecerá en el Administrador de Dispositivos como:
```
Espressif USB JTAG/serial debug unit
```

### Método B — Forzar upload desde Arduino

Con el ESP32-S3 en modo bootloader (paso anterior):

1. Seleccionar el puerto correcto en Arduino IDE
2. Presionar **Upload**
3. Si no inicia automáticamente: volver a hacer el proceso de botones justo cuando Arduino muestre `"Connecting..."`

### Método C — esptool.py (línea de comandos)

```bash
# Borrar flash completamente
esptool.py --chip esp32s3 --port COMx erase_flash

# Re-flashear el firmware compilado
esptool.py --chip esp32s3 --port COMx write_flash 0x0 phantom_firmware.bin
```

Para compilar el `.bin` desde Arduino IDE:
```
Sketch → Export Compiled Binary
```

---

## ⚡ Paso 4 — Restaurar Firmware de Phantom

Después de recuperar el dispositivo, subir el firmware con la configuración correcta:

### Checklist antes de subir

- [ ] **USB CDC on Boot** → `Enabled`
- [ ] **USB Mode** → `TinyUSB`
- [ ] Puerto COM correcto seleccionado
- [ ] Board: `ESP32S3 Dev Module`
- [ ] Sketch: `phantom_firmware.ino` abierto

### Subir el firmware

1. `Sketch → Upload` (Ctrl+U)
2. Esperar a que termine: `"Hard resetting via RTS pin..."`
3. Verificar en el Monitor Serial (`Tools → Serial Monitor`, baud: **115200**) que aparezca:
   ```
   [PHANTOM] Ready. Waiting for commands...
   ```

---

## 🔁 Paso 5 — Probar Conexión desde Python (Phantom Host)

```python
import serial
import serial.tools.list_ports

# Listar puertos disponibles
ports = serial.tools.list_ports.comports()
for p in ports:
    print(p.device, p.description)

# Conectar al ESP32-S3
ser = serial.Serial('COMx', 115200, timeout=1)
ser.write(b'PING\n')
resp = ser.readline()
print("Respuesta:", resp)
```

Si el puerto aparece y responde `PONG` (o el equivalente del firmware), todo está funcional.

---

## 🧯 Solución Rápida (Resumen de Emergencia)

```
1. Conectar ESP32-S3 por USB
2. Arduino IDE → Tools:
   - USB CDC on Boot = Enabled
   - USB Mode = TinyUSB
3. Botón BOOT + RESET para entrar a bootloader (si no acepta uploads)
4. Subir firmware
5. Verificar en Administrador de Dispositivos → Puertos (COM y LPT)
6. Si falta driver → usar Zadig → driver usbser
7. Probar conexión serial desde Python
```

---

## 📎 Referencias

- [Espressif ESP32-S3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf)
- [Arduino ESP32 Core — Board Options](https://docs.espressif.com/projects/arduino-esp32/en/latest/boards/ESP32-S3-DevKitC-1.html)
- [Zadig — USB Driver Tool](https://zadig.akeo.ie/)
- [esptool.py Docs](https://docs.espressif.com/projects/esptool/en/latest/)
