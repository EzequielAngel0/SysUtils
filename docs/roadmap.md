# 🔮 SysUtils — Roadmap de Mejoras

> **Versión actual:** 1.2.0  
> **Stack:** Rust + egui/eframe + xcap + rdev + serialport  
> **Última actualización:** Abril 2026

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
| ✓ B2 | Fix `color_disappear` referencia deslizante | Baseline se actualiza cuando el color está presente |
| ✓ B3 | Fix `LogBuffer` O(n) → O(1) | Migrado de `Vec` a `VecDeque` |
| ✓ B4 | Feedback visual de hotkeys inválidas | Frame rojo + tooltip en todos los módulos |
| ✓ F1 | Limpiar hotkeys con botón ✕ | En todos los módulos |
| ✓ F2 | Preview en tiempo real del Monitor | Hilo de fondo a 500ms, textura por generación |
| ✓ F3 | Perfiles de configuración | Guardar / cargar / eliminar / sobreescribir |
| ✓ F4 | Color picker visual en Monitor | `egui::color_picker::color_edit_button_rgb` |
| ✓ F5 | Contador de loops en vivo (Sequence) | `Arc<Mutex<u32>>` actualizado por hilo |
| ✓ | Script de instalación para Linux | `manager.sh` con install/uninstall + `.desktop` |
| ✓ | Config en tiempo real + bloqueo durante ejecución | Cambios al ESP32 al instante |

---

## ✅ Completado (v1.2 — Sprint 3)

| # | Feature | Notas |
|---|---------|-------|
| ✓ | Notificaciones del sistema | `notify-rust` — Panic Switch, desconexión ESP32, toggle de módulo |
| ✓ | Abstracción de Screen Capture (MP1) | Trait `ScreenCaptureBackend` + `XcapBackend` + `ScapBackend` (feature flag) |
| ✓ | Scan Anti-Cheat multiplataforma (MP2) | `#[cfg(windows)]` / `#[cfg(target_os = "linux")]` + soporte Wine |
| ✓ | Modo Stealth mejorado | Título de ventana falso + `prctl` en Linux |

---

## 🐛 Deuda Técnica

| Prioridad | Descripción |
|-----------|-------------|
| 🟡 Media | `main.rs` (780 líneas) — extraer sidebar derecho y hotkey dispatch |
| 🟡 Media | `monitor_tab.rs` (559 líneas) — dividir en preview + editor de región |
| 🟢 Baja | `stealth_mode_applied` solo detecta cambio de `enabled`, no de título/nombre |

---

## 🔴 Alta Prioridad (v1.3)

### 1. Cadenas de Acciones en el Monitor (Action Chains)
**Estado:** Pendiente  
En lugar de ejecutar un solo clic o tecla al detectar un cambio, ejecutar una secuencia completa de `MacroEvent`.

**Ejemplo:**
```
Detecta cambio → Espera 200ms → KEY_DOWN:e → Espera 100ms → KEY_UP:e → CLK_DOWN:L → CLK_UP:L
```

**Implementación:** Reutilizar `Vec<MacroEvent>` del módulo Sequence. Añadir `monitor_action_chain: Vec<MacroEvent>` a `AppConfig` y un mini-editor en la UI.

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

### 3. Tests Automatizados
**Estado:** Pendiente  
Tests unitarios para la lógica core sin dependencias de hardware.

**Cobertura mínima:**
- `PixelColor::distance`
- Serialización/deserialización de `AppConfig` (incluyendo campos v1.2)
- Rotación del `LogBuffer`
- `NotificationService` con `enabled = false` → no-op

**Crate:** `proptest` para tests basados en propiedades.

---

### 4. Refactoring de Arquitectura
**Estado:** Pendiente  
Reducir tamaño de archivos críticos sin cambiar comportamiento.

**Plan:**
- Extraer sidebar derecho → `ui/sidebar_right.rs` (~200 líneas)
- Extraer `process_hotkeys()` → método en `app.rs`
- Dividir `monitor_tab.rs` → `ui/monitor_preview.rs` + `ui/monitor_region_editor.rs`

---

## 🟡 Prioridad Media (v1.4)

### 5. Reconexión Automática al ESP32
**Estado:** Pendiente  
Si se pierde la conexión serial, intentar reconectar automáticamente cada N segundos sin intervención del usuario.

**Implementación:** Hilo de watchdog en `hw_link.rs` que detecta `ConnectionState::Disconnected` y reintenta `connect()` con backoff exponencial.

---

### 6. Biblioteca de Secuencias
**Estado:** Pendiente  
Mantener múltiples secuencias nombradas en lugar de una sola activa. Cada una se guarda como `sysutils_seq_{nombre}.json`.

**UI:** Lista con nombre, duración estimada, botones de cargar/eliminar/duplicar.

---

### 7. Gráfico de Actividad en Logs
**Estado:** Pendiente  
Mini-gráfico en la pestaña de Logs mostrando actividad reciente por módulo: clics/min, teclas/min, detecciones del monitor.

**Crate sugerido:** `egui_plot`

---

### 8. Tema Personalizable
**Estado:** Pendiente  
Cambiar entre tema oscuro, claro y personalizado. Los colores se guardan en `AppConfig`.

**Implementación:** Struct `ThemeConfig` con colores primarios, de acento y de fondo. Selector en el sidebar.

---

### 9. Historial de Sesiones
**Estado:** Pendiente  
Guardar un resumen de cada sesión (módulos usados, duración, eventos detectados) en un archivo JSON por día. Visible en la pestaña de Logs.

**Utilidad:** Auditoría de uso, debugging post-mortem.

---

### 10. Modo Turbo para Pulse
**Estado:** Pendiente  
Permitir delays por debajo de 1ms usando `spin_sleep` en lugar de `std::thread::sleep` para mayor precisión en el timing del auto-clicker.

**Crate sugerido:** `spin_sleep`

---

### 11. Condición de Tiempo en Monitor
**Estado:** Pendiente  
Añadir condición `active_for_ms` — solo disparar la acción si la condición se mantiene durante N milisegundos consecutivos. Evita falsos positivos por frames transitorios.

**Ejemplo:** Color rojo presente por más de 500ms → ejecutar acción.

---

## 🟢 Baja Prioridad / Largo Plazo (v2.0+)

### 12. Soporte para Gamepad como Hotkeys
**Estado:** Pendiente  
Usar botones de gamepad/joystick como triggers para activar módulos.

**Crate sugerido:** `gilrs`

---

### 13. OCR Básico en Monitor
**Estado:** Pendiente  
Detectar texto en regiones de pantalla para condiciones más avanzadas. Útil para leer valores numéricos (vida, mana, cooldowns con texto).

**Crate sugerido:** `leptess` (bindings de Tesseract) o `rusty-tesseract`

---

### 14. Sistema de Plugins / Scripts Lua
**Estado:** Pendiente  
Cargar scripts `.lua` que definan comportamientos personalizados reaccionando a eventos del monitor. Permite extender la app sin recompilar.

**Crate sugerido:** `mlua` con feature `lua54`

---

### 15. Exportar Secuencias como Script
**Estado:** Pendiente  
Exportar una secuencia grabada como script ejecutable (AutoHotkey `.ahk`, Python con `pyautogui`, o formato propio `.sysutils`). Permite compartir macros entre usuarios sin el hardware ESP32.

---

### 16. Dashboard de Estadísticas
**Estado:** Pendiente  
Pestaña dedicada con métricas acumuladas: total de clics enviados, teclas presionadas, tiempo activo por módulo, detecciones del monitor. Persistido en JSON entre sesiones.

---

### 17. Modo Headless / CLI
**Estado:** Pendiente  
Ejecutar SysUtils sin GUI, controlado por argumentos de línea de comandos o un archivo de configuración de sesión. Útil para automatización en servidores o scripts.

**Ejemplo:**
```bash
sysutils_native --headless --profile gaming --start pulse,keepalive
```

---

### 18. Integración con Webhooks
**Estado:** Pendiente  
Enviar una petición HTTP POST a una URL configurable cuando ocurren eventos críticos (Panic Switch, desconexión ESP32). Permite integración con Discord, Slack, o sistemas de monitoreo externos.

**Crate sugerido:** `ureq` (HTTP síncrono, sin async)

---

### 19. Grabación de Pantalla al Dispararse el Panic
**Estado:** Pendiente  
Cuando el Panic Switch se activa, guardar automáticamente un screenshot de la pantalla en `phantom_logs/` con timestamp. Útil para debugging post-mortem.

**Implementación:** Reutilizar `ScreenCapture::capture_frame()` + `image::save_buffer()`.

---

### 20. Soporte para Múltiples ESP32
**Estado:** Pendiente  
Conectar y gestionar más de un ESP32 simultáneamente, asignando módulos específicos a cada dispositivo (ej. Pulse en COM3, KeepAlive en COM4).

**Implementación:** `Vec<Arc<HwLink>>` en `SysUtilsApp`, selector de dispositivo por módulo en la UI.

---

## 📊 Tabla de Priorización

| Prioridad | # | Feature | Estado |
|-----------|---|---------|--------|
| ✅ Hecho | — | Sprint v1.0 completo | Completado |
| ✅ Hecho | — | Sprint v1.1 completo | Completado |
| ✅ Hecho | — | Sprint v1.2 completo | Completado |
| 🔴 Alta | 1 | Action Chains en Monitor | Pendiente v1.3 |
| 🔴 Alta | 2 | Multi-Monitor (Múltiples Regiones) | Pendiente v1.3 |
| 🔴 Alta | 3 | Tests automatizados | Pendiente v1.3 |
| 🔴 Alta | 4 | Refactoring de arquitectura | Pendiente v1.3 |
| 🟡 Media | 5 | Reconexión automática ESP32 | Pendiente v1.4 |
| 🟡 Media | 6 | Biblioteca de secuencias | Pendiente v1.4 |
| 🟡 Media | 7 | Gráfico de actividad en Logs | Pendiente v1.4 |
| 🟡 Media | 8 | Tema personalizable | Pendiente v1.4 |
| 🟡 Media | 9 | Historial de sesiones | Pendiente v1.4 |
| 🟡 Media | 10 | Modo Turbo para Pulse | Pendiente v1.4 |
| 🟡 Media | 11 | Condición de tiempo en Monitor | Pendiente v1.4 |
| 🟢 Baja | 12 | Soporte gamepad como hotkeys | Pendiente v2.0 |
| 🟢 Baja | 13 | OCR básico en Monitor | Pendiente v2.0 |
| 🟢 Baja | 14 | Sistema de plugins Lua | Pendiente v2.0 |
| 🟢 Baja | 15 | Exportar secuencias como script | Pendiente v2.0 |
| 🟢 Baja | 16 | Dashboard de estadísticas | Pendiente v2.0 |
| 🟢 Baja | 17 | Modo headless / CLI | Pendiente v2.0 |
| 🟢 Baja | 18 | Integración con webhooks | Pendiente v2.0 |
| 🟢 Baja | 19 | Screenshot automático en Panic | Pendiente v2.0 |
| 🟢 Baja | 20 | Soporte múltiples ESP32 | Pendiente v2.0 |

---

*Última revisión: Abril 2026 — SysUtils v1.2.0*
