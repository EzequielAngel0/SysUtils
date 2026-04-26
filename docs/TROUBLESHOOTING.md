# 🩺 Troubleshooting Guide

A continuación se listan las correcciones para todos los problemas técnicos y comportamientos inesperados durante el uso y desarrollo de Phantom Native.

---

### 1. `Acceso denegado (os error 5)` al ejecutar o compilar
**Problema:** Al intentar borrar una versión previa a través del Manager, instalar la App, o realizar `cargo build --release`, la terminal arroja un error 5 de Permiso/Bloqueo de archivos.
**Causa:** Se ha quedado abierta la interface antigua o algún hilo secundario (Phantom_native.exe) está secuestrando recursos en segundo plano, impidiendo a Rust reemplazar su ejecutable.
**Solución:** Ve al administrador de tareas de Windows y fuerza el cierre de `Phantom.exe`. O si prefieres consola, ejecuta esto como Administrador en Powershell:
```powershell
Stop-Process -Name "phantom_native" -Force
Stop-Process -Name "Phantom" -Force
```
*Tip: El Phantom Manager ya incorpora una herramienta anti-colisiones limpia por defecto.*

---

### 2. Phantom inicia pero el Mouse/Teclado no respoden (Clicas en aplicar y no ejecuta pulsos)
**Problema:** Conectas la tarjeta pero el software principal dice "Módulos listos" e ignoran tus comandos (el log puede marcar errores de puerto o de formato no parseable).
**Causa:** Dos problemas habituales: o tienes el Target Board en modo Genérico y el firmware ESP32 se cargó sin bandera OTG, O lo conectaste en el conector "UART".
**Solución:**
- Dirígete hacia las intrucciones del Firmware en [HARDWARE_SETUP.md](HARDWARE_SETUP.md) y asegúrate de verificar la opción `USB Mode: USB OTG (TinyUSB) / Hardware CDC` antes de quemar.
- Reconecta el ESP32 **a través del puerto marcado como USB**, no del puerto marcado como UART/COM lateral de la placa.

---

### 3. Falsos Positivos de Criptografía / "Vanguard/EAC detected my Rust app without running"
**Problema:** El módulo de sistema detiene Phantom de abrir sus ganchos globales porque dice detectar "vgk.sys" / `Anti-Cheat`. Pero es un simple bloc de notas.
**Solución:** Por el momento el escáner de `sysinfo` es informativo. Si observas la barra lateral y ves el Panel "Anti-Cheat Running = YES", significará siempre que hay servicios de núcleo en este ambiente. Usa Phantom con normalidad sabiendo que tus *flags de hardware están limpios* ya que el ratón es un hardware físico.

---

### 4. "No module named 'cargo'"
**Problema:** El instalador indica error fatal porque Cargo no se reconoce al iniciar.
**Causa:** Rust no ha sido instanciado correctamente en el `$PATH` de la máquina de Windows.
**Solución:** Dirígete a [Rustup Windows](https://rustup.rs/), descarga e instala el IDE Toolchain de Default. Reinicia por completo la consola de comandos o Visual Studio Code para que las variables de entorno refresquen el path al Rust Backend.

---

### 5. Script Is Not Digitally Signed / ExecutionPolicy Error
**Problema:** Al dar doble clic en el Instalador / Manager de compilación, arroja letras rojas bloqueantes de Seguridad de Powershell.
**Causa:** Windows Defender previene correr scripts locales aleatorios (.ps1).
**Solución:**
Inicia PowerShell como administrador y lanza:
```powershell
Set-ExecutionPolicy RemoteSigned
```
Selecciona *S (Sí)* y posteriormente cierra esta consola; ya podrás hacer doble clic en `Phantom Manager.bat` infinitas veces sin advertencias.

---

### 6. Sequence Recorder: Tiempos extremadamente largos sin razón aparente
**Problema:** Grabas una pequeña macro para minar o correr una ruta y al repetirla observas el log lanzando bloques de `Delay` inmensos y el personaje se traba esperando segundos.
**Solución:** Asegúrate de presionar el "Atajo de Parada" Inmediatamente al terminar tu macro, o usa un Editor temporal JSON para borrar manualnente el úlimo array event "Delay" (se ubica visualmente muy identificable).
