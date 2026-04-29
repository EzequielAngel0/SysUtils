# 🔮 SysUtils — Roadmap de Mejoras

> **Versión actual:** 1.1.0  
> **Stack:** Rust + egui/eframe + xcap + rdev + serialport  
> **Última actualización:** Abril 2026  
> **Spec activo:** `.kiro/specs/bugfix-and-v11-sprint/`

---

## ✅ Completado (v1.0 — Sprint 1)

| # | Feature | Notas |
|---|---------|-------|
| ✓ | Auto-guardado de configuración | Dirty flags + tick cada 5s |
| ✓ | Condiciones del Monitor | `change` / `color_appear` / `color_disappear` |
| ✓ | Exportar / Importar configuración | JSON portable |
| ✓ | System Overview (CPU/RAM) | Sidebar derecho, refresco cada 2s |
| ✓ | Hotkeys con modificadores | `Ctrl+`, `Alt+`, `Shift+` + alfanumérico |
| ✓ | Scripts de compilación Windows | `Manager.ps1` + `SysUtils.bat` |
| ✓ | Logging a archivo | `phantom_logs/` diario con `tracing-appender` |
| ✓ | Pulse con teclas de teclado | `e`, `f`, `Space` además de clics |
| ✓ | Filtro de ruido en Sequence | Filtro euclidiano 10px + coalescencia de MouseMove |
| ✓ | Detección de Anti-Cheat | EasyAntiCheat, BattlEye, Vanguard via `sysinfo` |

---

## ✅ Completado (v1.1 — Sprint 2)

| # | Feature / Bug | Notas |
|---|--------------|-------|
| ✓ B1 | Fix MouseMove en reproducción | Deltas relativos `MOUSE_MOVE_REL:dx:dy` |
| ✓ B2 | Fix `color_disappear` referencia deslizante | Baseline se actualiza cuando el color está presente (REGION/FULLSCREEN) |
| ✓ B3 | Fix `LogBuffer` O(n) → O(1) | Migrado de `Vec` a `VecDeque` |
| ✓ B4 | Feedback visual de hotkeys inválidas | Frame rojo + tooltip en todos los módulos |
| ✓ F1 | Limpiar hotkeys con botón ✕ | En todos los módulos, hotkey vacía es estado válido |
| ✓ F2 | Preview en tiempo real del Monitor | Hilo de fondo a 500ms, textura actualizada por generación |
| ✓ F3 | Perfiles de configuración | Guardar / cargar / eliminar / sobreescribir perfiles |
| ✓ F4 | Color picker visual en Monitor | `egui::color_picker::color_edit_button_rgb` |
| ✓ F5 | Contador de loops en vivo (Sequence) | `Arc<Mutex<u32>>` actualizado por hilo de reproducción |
| ✓ | Rebranding a "SysUtils" | Título, sidebar, Cargo.toml, scripts, docs |
| ✓ | Fix captura de clic al asignar hotkey | Cooldown de 200ms + drain del canal `raw_rx` |
| ✓ | Script de instalación para Linux | `manager.sh` con install/uninstall + `.desktop` entry |
| ✓ | Sobreescribir perfil activo (botón ↺) | Actualiza el archivo de perfil con la config actual |
| ✓ | Config en tiempo real + bloqueo durante ejecución | Cambios se envían al ESP32 al instante; controles bloqueados mientras el módulo corre |
| ✓ | Botón "Aplicar Config" en todos los módulos | Siempre visible para forzar aplicación manual |

---

## 🐛 Bugs Conocidos / Deuda Técnica

*(Todos los bugs del sprint anterior han sido resueltos)*

---

## 🔴 Alta Prioridad (v1.2)

### 1. Cadenas de Acciones en el Monitor (Action Chains)
**Estado:** Pendiente  
En lugar de ejecutar un solo clic o tecla al detectar un cambio, ejecutar una secuencia completa de `MacroEvent`.

**Ejemplo:**
```
Detecta cambio → Espera 200ms → KEY_DOWN:e → Espera 100ms → KEY_UP:e → CLK_DOWN:L → CLK_UP:L
```

**Implementación:** Reutilizar `Vec<MacroEvent>` del módulo Sequence. Añadir al `AppConfig` un campo `monitor_action_chain: Vec<MacroEvent>` y en la UI un mini-editor de la cadena.

---

### 2. Modo Multi-Monitor (Múltiples Regiones)
**Estado:** Pendiente  
Vigilar múltiples regiones o píxeles simultáneamente, cada uno con su propia condición y acción independiente.

**Ejemplo de uso:**
- Región 1: barra de vida baja → presionar tecla de curación
- Región 2: cooldown de habilidad listo → activarla
- Región 3: aparece texto en chat → hacer screenshot

```rust
pub struct MonitorRule {
    pub id: usize,
    pub name: String,
    pub mode: String,
    pub condition: String,
    pub action_chain: Vec<MacroEvent>,
    pub enabled: bool,
}
```

---

### 3. Notificaciones del Sistema
**Estado:** Pendiente  
Toast notifications nativas cuando:
- El Panic Switch se dispara
- Se pierde la conexión con el ESP32
- Un módulo se activa/desactiva por hotkey (configurable)

**Crate sugerido:** `notify-rust` (cross-platform, funciona en Windows y Linux)

---

### 4. Tests Automatizados
**Estado:** Pendiente  
Tests unitarios para la lógica core sin dependencias de hardware.

**Cobertura mínima:**
- `extract_key_char` / `extract_mouse_btn`
- Serialización/deserialización de `AppConfig`
- `PixelColor::distance`
- Rotación del `LogBuffer`

---

## 🟡 Prioridad Media (v1.3)

### 5. Gráfico de Actividad
**Estado:** Pendiente  
Mini-gráfico en la pestaña de Logs mostrando actividad reciente por módulo: clics/min, teclas/min, detecciones del monitor.

**Crate sugerido:** `egui_plot`

---

### 6. Tema Personalizable
**Estado:** Pendiente  
Cambiar entre tema oscuro, claro y personalizado. Los colores se guardan en `AppConfig`.

---

### 7. Reconexión Automática al ESP32
**Estado:** Pendiente  
Si se pierde la conexión serial, intentar reconectar automáticamente cada N segundos.

---

### 8. Secuencias con Nombre y Biblioteca
**Estado:** Pendiente  
Mantener una biblioteca de secuencias nombradas en lugar de una sola activa. Cada una se guarda como `sysutils_seq_{nombre}.json`.

---

## 🖥️ Soporte Multiplataforma (Windows + Linux)

### Estrategia General
**Estado:** En progreso — script Linux completado  
El stack es mayormente cross-platform. `egui/eframe`, `serialport`, `rdev`, `serde` y `sysinfo` funcionan en ambos sistemas. El script `manager.sh` ya permite compilar e instalar en Linux.

**Pendiente:**

### MP1. Abstracción de Screen Capture por Plataforma
**Estado:** Pendiente  
`screen_capture.rs` usa `xcap` directamente. Hay que abstraerlo detrás de un trait para poder intercambiar el backend según la plataforma.

| Crate | Windows | X11 | Wayland |
|-------|---------|-----|---------|
| `xcap` *(actual)* | ✅ | ✅ | ⚠ parcial |
| `scap` | ✅ | ✅ | ✅ |

---

### MP2. Scan de Anti-Cheat en Linux
**Estado:** Pendiente  
Los nombres de procesos a detectar son diferentes en Linux (sin `.exe`). Añadir listas separadas por plataforma.

```rust
#[cfg(windows)]
const ANTICHEAT_PROCESSES: &[&str] = &["EasyAntiCheat.exe", "BEService.exe", "vgc.exe"];

#[cfg(target_os = "linux")]
const ANTICHEAT_PROCESSES: &[&str] = &["EasyAntiCheat", "wine-preloader", "BEService"];
```

---

## 🟢 Baja Prioridad / Largo Plazo (v2.0+)

### 9. Sistema de Plugins / Scripts Lua
**Estado:** Pendiente  
Cargar scripts `.lua` que definan comportamientos personalizados reaccionando a eventos del monitor.

**Crate sugerido:** `mlua` con feature `lua54`

---

### 10. OCR Básico
**Estado:** Pendiente  
Detectar texto en regiones de pantalla para condiciones más avanzadas.

**Crate sugerido:** `leptess` (bindings de Tesseract) o `rusty-tesseract`

---

### 11. Soporte para Gamepad como Hotkeys
**Estado:** Pendiente  
Usar botones de gamepad/joystick como triggers para activar módulos.

**Crate sugerido:** `gilrs`

---

### 12. Modo Stealth Mejorado
**Estado:** Pendiente  
Ofuscación del nombre del proceso y título de ventana para reducir la superficie de detección.

---

## 📊 Tabla de Priorización

| Prioridad | # | Feature | Estado |
|-----------|---|---------|--------|
| ✅ Hecho | B1-B4 | Todos los bugs del sprint 1 | Completado v1.1 |
| ✅ Hecho | F1-F5 | Todas las features de alta prioridad | Completado v1.1 |
| ✅ Hecho | — | Rebranding, fix hotkey, Linux, perfiles overwrite, config live | Completado v1.1 |
| 🔴 Alta | 1 | Action Chains en Monitor | Pendiente v1.2 |
| 🔴 Alta | 2 | Multi-Monitor | Pendiente v1.2 |
| 🔴 Alta | 3 | Notificaciones del sistema | Pendiente v1.2 |
| 🔴 Alta | 4 | Tests automatizados | Pendiente v1.2 |
| 🟡 Media | 5 | Gráfico de actividad | Pendiente v1.3 |
| 🟡 Media | 6 | Tema personalizable | Pendiente v1.3 |
| 🟡 Media | 7 | Reconexión automática ESP32 | Pendiente v1.3 |
| 🟡 Media | 8 | Biblioteca de secuencias | Pendiente v1.3 |
| 🔵 Plataforma | MP1 | Abstracción screen capture | Pendiente |
| 🔵 Plataforma | MP2 | Anti-cheat scan en Linux | Pendiente |
| 🟢 Baja | 9 | Plugins Lua | Pendiente v2.0 |
| 🟢 Baja | 10 | OCR básico | Pendiente v2.0 |
| 🟢 Baja | 11 | Soporte gamepad | Pendiente v2.0 |
| 🟢 Baja | 12 | Modo stealth mejorado | Pendiente v2.0 |

---

*Última revisión: Abril 2026 — SysUtils Development*
