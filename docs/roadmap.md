# 🔮 Phantom Native — Roadmap de Mejoras Sugeridas

> **Versión actual:** 1.0.0  
> **Stack:** Rust + egui/eframe + xcap + rdev + serialport  
> **Fecha:** Abril 2026

---

## 🎯 Mejoras de Alta Prioridad

### 1. Auto-guardado de Configuración
**Estado:** Completado
Actualmente la configuración se guarda automáticamente al suceder modificaciones en la Interfaz (Dirty Flags).

```rust
// Ejemplo: comparar hash de config anterior vs actual
if self.config_hash != self.config.calculate_hash() {
    self.config.save();
    self.config_hash = self.config.calculate_hash();
}
```

---

### 2. Captura en Tiempo Real del Monitor
**Estado:** Parcialmente implementado  
La vista previa actualmente se captura una sola vez. Implementar un hilo de fondo que actualice la textura cada ~500ms para dar feedback visual en tiempo real.

**Beneficios:**
- El usuario puede ver exactamente qué está monitoreando
- Facilita la selección precisa de regiones y píxeles
- Confirma visualmente que la ventana anclada sigue visible

---

### 3. Perfiles de Configuración
**Descripción:** Permitir guardar y cargar múltiples perfiles de configuración (ej: "Perfil Gaming", "Perfil AFK", "Perfil Farming").

**Implementación sugerida:**
- Dropdown en la barra superior para seleccionar perfil activo
- Botones para crear, renombrar y eliminar perfiles
- Cada perfil se guarda como `phantom_profile_{nombre}.json`

---

### 4. Indicadores Visuales del Estado de Hotkeys
**Descripción:** Mostrar en la UI qué hotkeys están activamente registrados y cuáles fallaron (por ejemplo, si se ingresó una tecla inválida el campo debería ponerse rojo).

---

## 🚀 Funcionalidades Nuevas

### 5. Sistema de Condiciones para el Monitor
**Estado:** Completado
**Descripción:** En lugar de solo detectar "cambio", permitir condiciones más sofisticadas:
- **Color específico aparece:** Reaccionar cuando un píxel cambia a un color específico
- **Color específico desaparece:** Reaccionar cuando un color deja de estar presente
- **Umbral de similitud:** Usar % de similitud en vez de diferencia absoluta

```
Si [Píxel (400,300)] cambia a [RGB(255,0,0) ± 20] → Ejecutar [Clic Izquierdo]
```

---

### 6. Cadenas de Acciones (Action Chains)
**Descripción:** En lugar de solo hacer "un clic" cuando el monitor detecta algo, permitir ejecutar una secuencia completa:
1. Detecta cambio → Espera 200ms → Clic en (X,Y) → Espera 100ms → Presiona tecla "E"

**Implementación:** Reutilizar el sistema de `MacroEvent` del módulo Sequence como acción del Monitor.

---

### 7. Modo "Multi-Monitor"
**Descripción:** Permitir vigilar múltiples regiones o píxeles simultáneamente, cada uno con su propia acción.

**Ejemplo de uso:**
- Vigilar barra de vida (región 1) → Si baja, presionar tecla de curación
- Vigilar cooldown de habilidad (región 2) → Si está lista, activarla
- Vigilar chat (región 3) → Si aparece texto nuevo, hacer screenshot

---

### 8. Exportar/Importar Configuraciones
**Estado:** Completado
**Descripción:** Botón para exportar toda la configuración como archivo `.json` portable, y otro para importarla. Útil para compartir configuraciones entre máquinas.

---

### 9. Notificaciones del Sistema
**Descripción:** Usar notificaciones nativas de Windows (toast notifications) para alertar cuando:
- Un módulo se activa/desactiva
- El Panic Switch detecta un cambio
- La conexión ESP32 se pierde

**Crate sugerido:** `notify-rust` o `winrt-notification`

---

### 10. Sistema de Plugins/Scripts
**Descripción:** Permitir cargar scripts Lua o Python que definan comportamientos personalizados.

```lua
-- script: auto_fish.lua
function on_pixel_change(x, y, old_color, new_color)
    if new_color.r > 200 and new_color.g < 50 then
        phantom.click("left", 100)
        phantom.wait(500)
        phantom.press_key("e")
    end
end
```

**Crate sugerido:** `rlua` o `mlua`

---

## 🎨 Mejoras de UI/UX

### 11. Tema Personalizable
**Descripción:** Permitir al usuario cambiar colores del tema (dark/light/custom) y guardarlos en la configuración.

---

### 12. Gráfico de Actividad
**Descripción:** Un mini-gráfico en la sección de logs que muestre la actividad reciente de cada módulo (clics/minuto, teclas/minuto, detecciones del monitor).

**Crate sugerido:** `egui_plot`

---

### 13. Indicador de Uso de CPU
**Estado:** Completado
**Descripción:** Mostrar el uso de CPU y RAM del proceso Phantom en una barra lateral. Especialmente útil para el Monitor, donde el paso de muestreo afecta directamente el rendimiento.

**Crate sugerido:** `sysinfo`

---

### 14. Atajos de Teclado Globales con Modificadores
**Estado:** Completado
Permitir combinaciones como `Ctrl+F6`, `Alt+F7` en lugar de solo teclas individuales.

---

### 15. Soporte para Gamepad
**Descripción:** Permitir usar botones del gamepad/joystick como hotkeys.

**Crate sugerido:** `gilrs`

---

## 🔧 Mejoras Técnicas

### 16. Compilación Optimizada
**Estado:** Completado
Crear un script `build_release.ps1` que compile en modo release con optimizaciones Nativas:
```powershell
$env:RUSTFLAGS="-C target-cpu=native"
cargo build --release
```

**Reducción esperada:** ~80% menos uso de CPU, ~60% menos RAM, inicio más rápido.

---

### 17. Instalador / Distribuible
**Estado:** Completado
**Descripción:** Creado `build_portable.ps1` para generar paquete `.zip` portable que incluye config, dependencias, assets y el ejecutable principal.

---

### 18. Tests Automatizados
**Descripción:** Agregar tests unitarios para:
- Parseo de hotkeys (`parse_key`)
- Configuración (serialización/deserialización)
- Lógica de diff de píxeles
- Eventos de secuencia

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_key() {
        assert_eq!(parse_key("F6"), Some(Key::F6));
        assert_eq!(parse_key("escape"), Some(Key::Escape));
        assert_eq!(parse_key("invalid"), None);
    }
}
```

---

### 19. Logging a Archivo
**Estado:** Completado
**Descripción:** Logs persistentes diarios ubicados en `/phantom_logs/` mediante crates `tracing` + `tracing-appender`.

---

### 20. Detección de Anti-Cheat
**Estado:** Completado
**Descripción:** Sistema de alerta que detecta si ciertos procesos de anti-cheat están corriendo (EasyAntiCheat, BattlEye, Vanguard) utilizando escaneos silenciosos del crate `sysinfo`.

---

## 📊 Priorización Recomendada

| Prioridad | # | Feature | Esfuerzo |
|-----------|---|---------|----------|
| 🔴 Alta | 1 | Auto-guardado | Bajo |
| 🔴 Alta | 2 | Captura en Tiempo Real | Medio |
| 🔴 Alta | 16 | Build Release | Bajo |
| 🟡 Media | 3 | Perfiles | Medio |
| 🟡 Media | 5 | Condiciones Monitor | Alto |
| 🟡 Media | 14 | Hotkeys con Modifiers | Bajo |
| 🟡 Media | 18 | Tests | Medio |
| 🟢 Baja | 6 | Action Chains | Alto |
| 🟢 Baja | 7 | Multi-Monitor | Alto |
| 🟢 Baja | 10 | Plugins/Scripts | Muy Alto |
| 🟢 Baja | 12 | Gráfico de Actividad | Medio |

---

*Documento generado por Phantom Development Team — Abril 2026*
