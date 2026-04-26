// ═══════════════════════════════════════════════════════════════════════════════
// Phantom Firmware — ESP32-S3 HID Controller
// Non-blocking architecture: millis() state machine + zero-allocation parsing
// ═══════════════════════════════════════════════════════════════════════════════

#include "USB.h"
#include "USBHIDMouse.h"
#include "USBHIDKeyboard.h"

USBHIDMouse Mouse;
USBHIDKeyboard Keyboard;

// ── Estado global ───────────────────────────────────────────────────────────────
bool isRunning   = false;
bool isMouseMode = true;
bool holdMode    = false;

char keyToPress  = 'w';
char targetBtn   = 'L';
int  minDelay    = 50;
int  maxDelay    = 100;

// ── Máquina de estados no-bloqueante ────────────────────────────────────────────
enum PulseState { IDLE, PRESSING, RELEASING };
PulseState pulseState       = IDLE;
unsigned long pulseTimer    = 0;
unsigned long pressDuration = 0;
unsigned long pauseDuration = 0;

// ── Buffer serial estático (cero allocaciones de heap) ──────────────────────────
static char cmdBuf[64];
static uint8_t cmdIdx = 0;

// ── Helpers ─────────────────────────────────────────────────────────────────────

void pressTarget() {
  if (isMouseMode) {
    switch (targetBtn) {
      case 'R': Mouse.press(MOUSE_RIGHT);  break;
      case 'M': Mouse.press(MOUSE_MIDDLE); break;
      default:  Mouse.press(MOUSE_LEFT);   break;
    }
  } else {
    Keyboard.press(keyToPress);
  }
}

void releaseAll() {
  Mouse.release(MOUSE_LEFT);
  Mouse.release(MOUSE_RIGHT);
  Mouse.release(MOUSE_MIDDLE);
  Keyboard.releaseAll();
}

// ── Comparador rápido de prefijos sin heap ──────────────────────────────────────
static bool startsWith(const char* str, const char* prefix) {
  while (*prefix) {
    if (*str++ != *prefix++) return false;
  }
  return true;
}

// ── Parseo de enteros inline ────────────────────────────────────────────────────
static int fastAtoi(const char* s) {
  int val = 0;
  while (*s >= '0' && *s <= '9') {
    val = val * 10 + (*s - '0');
    s++;
  }
  return val;
}

// ── Procesamiento de un comando completo ────────────────────────────────────────
void processCommand(const char* cmd) {

  // ── Control de flujo ──────────────────────────────────────────────────────────
  if (cmd[0] == 'S' && cmd[1] == 'T') {
    if (cmd[2] == 'A') {                          // START
      isRunning = true;
      if (holdMode) {
        pressTarget();
      } else {
        pulseState = PRESSING;
        pressTarget();
        pressDuration = random(15, 30);
        pulseTimer = millis();
      }
      Serial.println("ACK:START");
      return;
    }
    if (cmd[2] == 'O') {                          // STOP
      isRunning = false;
      pulseState = IDLE;
      releaseAll();
      Serial.println("ACK:STOP");
      return;
    }
  }

  // ── PING (heartbeat) ─────────────────────────────────────────────────────────
  if (cmd[0] == 'P' && cmd[1] == 'I') {           // PING
    Serial.println("PONG");
    return;
  }

  // ── Modos ─────────────────────────────────────────────────────────────────────
  if (startsWith(cmd, "MODE:")) {
    const char* sub = cmd + 5;
    if (sub[0] == 'H')       holdMode = true;      // MODE:HOLD
    else if (sub[0] == 'P')  holdMode = false;     // MODE:PULSE
    else if (sub[0] == 'M')  isMouseMode = true;   // MODE:MOUSE
    else if (startsWith(sub, "KEY:")) {            // MODE:KEY:x
      isMouseMode = false;
      keyToPress  = sub[4];
    }
    Serial.println("ACK:MODE");
    return;
  }

  // ── Botón objetivo ────────────────────────────────────────────────────────────
  if (startsWith(cmd, "TARGET_BTN:")) {            // TARGET_BTN:L/R/M
    targetBtn = cmd[11];
    Serial.println("ACK:TARGET");
    return;
  }

  // ── Delays ────────────────────────────────────────────────────────────────────
  if (startsWith(cmd, "DELAY:")) {                 // DELAY:50:100
    const char* p = cmd + 6;
    minDelay = fastAtoi(p);
    while (*p && *p != ':') p++;
    if (*p == ':') {
      p++;
      maxDelay = fastAtoi(p);
    }
    Serial.println("ACK:DELAY");
    return;
  }

  // ── Comandos directos de ratón ────────────────────────────────────────────────
  if (startsWith(cmd, "CLK_DOWN:")) {
    char b = cmd[9];
    if      (b == 'R') Mouse.press(MOUSE_RIGHT);
    else if (b == 'M') Mouse.press(MOUSE_MIDDLE);
    else               Mouse.press(MOUSE_LEFT);
    return;
  }
  if (startsWith(cmd, "CLK_UP:")) {
    char b = cmd[7];
    if      (b == 'R') Mouse.release(MOUSE_RIGHT);
    else if (b == 'M') Mouse.release(MOUSE_MIDDLE);
    else               Mouse.release(MOUSE_LEFT);
    return;
  }

  // ── Comandos directos de teclado ──────────────────────────────────────────────
  if (startsWith(cmd, "KEY_DOWN:")) {
    Keyboard.press(cmd[9]);
    return;
  }
  if (startsWith(cmd, "KEY_UP:")) {
    Keyboard.release(cmd[7]);
    return;
  }
  if (startsWith(cmd, "TAP_KEY:")) {
    Keyboard.write(cmd[8]);
    return;
  }

  // ── Movimiento de ratón (nuevo) ───────────────────────────────────────────────
  if (startsWith(cmd, "MOUSE_MOVE:")) {            // MOUSE_MOVE:dx:dy
    const char* p = cmd + 11;
    bool negX = false, negY = false;
    if (*p == '-') { negX = true; p++; }
    int dx = fastAtoi(p);
    if (negX) dx = -dx;
    while (*p && *p != ':') p++;
    if (*p == ':') {
      p++;
      if (*p == '-') { negY = true; p++; }
      int dy = fastAtoi(p);
      if (negY) dy = -dy;
      Mouse.move(dx, dy, 0);
    }
    return;
  }
}

// ── Setup ───────────────────────────────────────────────────────────────────────

void setup() {
  Serial.begin(115200);
  Mouse.begin();
  Keyboard.begin();
  USB.begin();
}

// ── Loop (100% no-bloqueante) ───────────────────────────────────────────────────

void loop() {

  // ── 1. Leer serial byte a byte (nunca bloquea) ────────────────────────────────
  while (Serial.available() > 0) {
    char c = Serial.read();
    if (c == '\n' || c == '\r') {
      if (cmdIdx > 0) {
        cmdBuf[cmdIdx] = '\0';
        processCommand(cmdBuf);
        cmdIdx = 0;
      }
    } else if (cmdIdx < sizeof(cmdBuf) - 1) {
      cmdBuf[cmdIdx++] = c;
    }
  }

  // ── 2. Máquina de estados del Pulse (no-bloqueante) ───────────────────────────
  if (isRunning && !holdMode) {
    unsigned long now = millis();

    switch (pulseState) {
      case IDLE:
        pulseState    = PRESSING;
        pressTarget();
        pressDuration = random(15, 30);
        pulseTimer    = now;
        break;

      case PRESSING:
        if (now - pulseTimer >= pressDuration) {
          releaseAll();
          pauseDuration = random(minDelay, maxDelay);
          pulseTimer    = now;
          pulseState    = RELEASING;
        }
        break;

      case RELEASING:
        if (now - pulseTimer >= pauseDuration) {
          pulseState = IDLE;
        }
        break;
    }
  }
}
