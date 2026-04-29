# Requirements Document

## Introduction

Este sprint agrupa 4 correcciones de bugs y 5 features de alta prioridad para Phantom Native (Rust + egui/eframe). El objetivo es cerrar deuda técnica crítica y añadir las mejoras de UX más solicitadas en una sola iteración, manteniendo la arquitectura existente: trait-per-module, `Arc<Mutex<T>>` para estado compartido, dirty flag + auto-save cada 5 s, y comunicación serial con el ESP32.

## Glossary

- **App**: La aplicación `SysUtilsApp` definida en `src/app.rs`.
- **ESP32**: El microcontrolador conectado por puerto serial que recibe comandos HID.
- **HwLink**: El struct `HwLink` en `src/hw_link.rs` que encapsula la comunicación serial.
- **LogBuffer**: El struct `LogBuffer` en `src/models.rs` que almacena entradas de log en memoria.
- **MacroEvent**: El enum `MacroEvent` en `src/models.rs` que representa un evento grabado en una secuencia.
- **Monitor**: El módulo de vigilancia de pantalla implementado en `src/logic/monitor.rs` y `src/ui/monitor_tab.rs`.
- **Sequence**: El módulo de grabación/reproducción de macros en `src/logic/sequence.rs` y `src/ui/sequence_tab.rs`.
- **AppConfig**: El struct `AppConfig` en `src/config.rs` que contiene todos los campos persistentes.
- **HotkeyEngine**: El motor de hotkeys en `src/hotkey_engine.rs`.
- **Profile**: Un archivo JSON con nombre `sysutils_profile_{nombre}.json` que contiene una copia completa de `AppConfig`.
- **Preview_Thread**: Hilo de fondo que captura frames del Monitor periódicamente para actualizar la textura de vista previa.
- **Loop_Counter**: Contador `Arc<Mutex<u32>>` que el hilo de reproducción de Sequence actualiza en cada iteración.

---

## Requirements

### Requirement 1 — B1: Reproducción de MouseMove en Sequence

**User Story:** Como usuario que graba y reproduce secuencias de ratón, quiero que los movimientos del cursor se ejecuten correctamente durante la reproducción, para que las macros que dependen de posicionamiento funcionen de extremo a extremo.

#### Acceptance Criteria

1. WHEN la App reproduce un `MacroEvent::MouseMove(x, y)`, THE Sequence SHALL calcular el delta relativo `dx = x - last_x` y `dy = y - last_y` respecto a la posición absoluta del evento anterior de tipo `MouseMove`.
2. WHEN el delta ha sido calculado, THE Sequence SHALL enviar el comando `MOUSE_MOVE_REL:dx:dy` al ESP32 mediante HwLink.
3. THE Sequence SHALL inicializar `last_x` y `last_y` al valor del primer `MacroEvent::MouseMove` encontrado en la secuencia, enviando un delta `(0, 0)` para ese primer evento.
4. WHEN no existe ningún `MacroEvent::MouseMove` previo en la iteración actual del loop, THE Sequence SHALL tratar la posición anterior como la del último `MouseMove` del loop anterior, o `(0, 0)` si es el primer loop.
5. THE Sequence SHALL preservar el comportamiento existente para `KeyDown`, `KeyUp`, `MouseDown`, `MouseUp` y `Delay` sin modificación.

---

### Requirement 2 — B2: Corrección de `color_disappear` en modo REGION

**User Story:** Como usuario que usa el Monitor con condición `color_disappear` en modo REGION, quiero que la detección sea precisa y no genere falsos negativos, para que las acciones automáticas se disparen en el momento correcto.

#### Acceptance Criteria

1. WHEN el Monitor está activo con condición `color_disappear` y modo `REGION`, THE Monitor SHALL actualizar la imagen de referencia interna (`baseline_img`) con el frame capturado en cada iteración en la que el color objetivo SÍ está presente en la región.
2. WHEN el color objetivo ya no está presente en la región comparado con la referencia actualizada, THE Monitor SHALL disparar la acción configurada.
3. WHEN el Monitor está activo con condición `color_disappear` y modo `PIXEL`, THE Monitor SHALL mantener el comportamiento actual (comparación contra la referencia fija capturada al inicio).
4. WHEN el Monitor está activo con condición `color_disappear` y modo `FULLSCREEN`, THE Monitor SHALL aplicar la misma lógica de referencia deslizante que en modo `REGION`.
5. THE Monitor SHALL registrar en LogBuffer un mensaje de nivel `Info` cada vez que la referencia interna sea actualizada en modo deslizante.

---

### Requirement 3 — B3: LogBuffer con VecDeque

**User Story:** Como desarrollador que mantiene el proyecto, quiero que el LogBuffer use una estructura de datos eficiente para la rotación de entradas, para eliminar micro-stutters en el hilo de UI causados por operaciones O(n).

#### Acceptance Criteria

1. THE LogBuffer SHALL almacenar las entradas de log en un `std::collections::VecDeque<LogEntry>` en lugar de `Vec<LogEntry>`.
2. WHEN el número de entradas alcanza el límite `max_entries`, THE LogBuffer SHALL eliminar la entrada más antigua usando `pop_front()` con complejidad O(1).
3. THE LogBuffer SHALL exponer el método `get_all()` retornando `Vec<LogEntry>` (colección ordenada de más antiguo a más reciente) para mantener compatibilidad con el código de renderizado existente.
4. THE LogBuffer SHALL mantener la firma pública de los métodos `log()`, `get_all()` y `clear()` sin cambios en sus parámetros ni tipos de retorno.
5. WHEN se llama a `clear()`, THE LogBuffer SHALL vaciar el `VecDeque` completamente.

---

### Requirement 4 — B4: Feedback visual de hotkey inválida

**User Story:** Como usuario que configura hotkeys, quiero recibir retroalimentación visual inmediata cuando introduzco una tecla inválida, para saber que debo corregirla antes de guardar.

#### Acceptance Criteria

1. WHEN el usuario introduce un string de hotkey que HotkeyEngine no puede parsear como tecla válida, THE App SHALL colorear el campo de hotkey correspondiente en rojo (`Color32::from_rgb(200, 60, 60)` como fondo o borde).
2. WHEN el campo de hotkey está en estado de error, THE App SHALL mostrar un tooltip con el texto `"Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft"` al pasar el cursor sobre el campo.
3. WHEN el usuario corrige el string a una hotkey válida o lo deja vacío, THE App SHALL restaurar el estilo visual normal del campo.
4. THE App SHALL aplicar este feedback en todos los campos de hotkey de los módulos: Pulse, KeepAlive, Monitor, Panic, Sequence (grabar) y Sequence (reproducir).
5. IF el campo de hotkey contiene un string inválido al intentar iniciar el módulo correspondiente, THEN THE App SHALL mostrar un mensaje de error en la barra de estado y no registrar la hotkey.

---

### Requirement 5 — F1: Botón para limpiar hotkeys

**User Story:** Como usuario que quiere desactivar el trigger por teclado de un módulo, quiero poder limpiar la hotkey asignada con un solo clic, para que el módulo solo sea controlable desde la UI.

#### Acceptance Criteria

1. THE App SHALL mostrar un botón `✕` junto a cada campo de hotkey en los módulos Pulse, KeepAlive, Monitor, Panic, Sequence (grabar) y Sequence (reproducir).
2. WHEN el usuario hace clic en el botón `✕` de un campo de hotkey, THE App SHALL establecer el string de hotkey correspondiente en `AppConfig` a `""` (cadena vacía).
3. WHEN la hotkey es establecida a `""`, THE App SHALL llamar a `apply_hotkeys()` para desregistrar el binding del HotkeyEngine.
4. WHEN la hotkey es establecida a `""`, THE App SHALL llamar a `mark_dirty()` para que el cambio se persista en el siguiente auto-save.
5. WHEN la hotkey de un campo es `""`, THE App SHALL mostrar el texto `"Ninguna"` en el campo y deshabilitar el botón `✕` (o mostrarlo en gris) para indicar que ya está vacío.
6. THE App SHALL mostrar el botón `✕` con un tooltip `"Quitar hotkey"` al pasar el cursor sobre él.

---

### Requirement 6 — F2: Preview en tiempo real del Monitor

**User Story:** Como usuario que configura regiones y píxeles en el Monitor, quiero ver una vista previa actualizada de la pantalla vigilada, para confirmar que la región seleccionada es correcta mientras el monitor está activo.

#### Acceptance Criteria

1. WHEN el usuario está en la pestaña Monitor y hay una referencia capturada, THE Preview_Thread SHALL capturar un nuevo frame del target seleccionado cada 500 ms aproximadamente.
2. THE Preview_Thread SHALL almacenar el frame capturado en un `Arc<Mutex<Option<Vec<u8>>>>` compartido con el hilo de UI.
3. WHEN el hilo de UI renderiza la pestaña Monitor y hay un frame nuevo disponible, THE App SHALL actualizar la `TextureHandle` de vista previa con los nuevos píxeles.
4. WHEN el usuario navega fuera de la pestaña Monitor, THE Preview_Thread SHALL continuar ejecutándose pero la UI no actualizará la textura hasta que el usuario regrese.
5. WHEN el usuario hace clic en "Re-capturar Referencia", THE Preview_Thread SHALL detenerse y reiniciarse con el nuevo target.
6. WHEN el Monitor es detenido (`stop_monitor()`), THE Preview_Thread SHALL detenerse también usando un flag `Arc<Mutex<bool>>`.
7. IF la captura de frame falla en una iteración, THEN THE Preview_Thread SHALL registrar el error en LogBuffer con nivel `Warning` y continuar en la siguiente iteración sin detener el hilo.

---

### Requirement 7 — F3: Perfiles de configuración

**User Story:** Como usuario que usa Phantom Native en distintos contextos (gaming, AFK, farming), quiero guardar y cargar múltiples perfiles de configuración con nombre, para cambiar entre configuraciones completas sin reconfigurar manualmente.

#### Acceptance Criteria

1. THE App SHALL guardar cada perfil como un archivo JSON con nombre `sysutils_profile_{nombre}.json` en el mismo directorio que el ejecutable, donde `{nombre}` es el nombre del perfil sin espacios ni caracteres especiales.
2. WHEN el usuario crea un perfil nuevo, THE App SHALL guardar el `AppConfig` actual como el nuevo perfil y añadirlo a la lista de perfiles disponibles.
3. WHEN el usuario carga un perfil, THE App SHALL reemplazar el `AppConfig` activo con el contenido del archivo del perfil y llamar a `apply_hotkeys()`.
4. WHEN el usuario elimina un perfil, THE App SHALL borrar el archivo `sysutils_profile_{nombre}.json` correspondiente y eliminarlo de la lista de perfiles disponibles.
5. THE App SHALL mostrar un `ComboBox` en la barra superior o en una sección dedicada de la UI con la lista de perfiles disponibles detectados en el directorio del ejecutable.
6. WHEN la App inicia, THE App SHALL escanear el directorio del ejecutable y cargar la lista de archivos `sysutils_profile_*.json` disponibles.
7. IF un archivo de perfil no puede ser parseado como `AppConfig` válido, THEN THE App SHALL registrar un error en LogBuffer con nivel `Error` y omitir ese perfil de la lista.
8. THE App SHALL persistir el nombre del perfil activo en `sysutils_config.json` como campo `active_profile: String` para restaurarlo al reiniciar.

---

### Requirement 8 — F4: Color picker visual en el Monitor

**User Story:** Como usuario que configura condiciones de color en el Monitor, quiero seleccionar el color objetivo con un selector visual, para no tener que introducir valores RGB manualmente.

#### Acceptance Criteria

1. WHEN el usuario está configurando la condición `color_appear` o `color_disappear`, THE App SHALL mostrar un widget `egui::color_picker::color_edit_button_rgb` en lugar de los tres campos `DragValue` separados para R, G y B.
2. WHEN el usuario selecciona un color en el picker, THE App SHALL actualizar `AppConfig.monitor_target_color_r`, `AppConfig.monitor_target_color_g` y `AppConfig.monitor_target_color_b` con los valores correspondientes.
3. WHEN los valores de color cambian a través del picker, THE App SHALL llamar a `mark_dirty()` para persistir el cambio.
4. THE App SHALL mantener el swatch de previsualización del color actual junto al picker.
5. THE App SHALL mantener el campo de tolerancia de color (`monitor_color_tolerance`) como `DragValue` sin cambios.

---

### Requirement 9 — F5: Contador de loops en vivo en Sequence

**User Story:** Como usuario que ejecuta secuencias con múltiples repeticiones, quiero ver en tiempo real cuántos loops se han completado, para saber el progreso de la reproducción sin tener que revisar los logs.

#### Acceptance Criteria

1. THE App SHALL mantener un `Arc<Mutex<u32>>` llamado `sequence_loop_counter` en `SysUtilsApp` que represente el número de loops completados en la reproducción actual.
2. WHEN el hilo de reproducción completa un loop, THE Sequence SHALL incrementar `sequence_loop_counter` en 1.
3. WHEN `play_sequence()` es llamado, THE Sequence SHALL resetear `sequence_loop_counter` a `0` antes de iniciar el hilo de reproducción.
4. WHEN la UI renderiza la pestaña Sequence y `sequence_playing` es `true`, THE App SHALL mostrar el texto `"Loop: {n} / {total}"` donde `{n}` es el valor actual de `sequence_loop_counter` y `{total}` es el número de loops configurado (o `"∞"` si es 0).
5. WHEN la reproducción finaliza o es detenida, THE App SHALL mantener el último valor de `sequence_loop_counter` visible hasta que se inicie una nueva reproducción.
