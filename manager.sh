#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# manager.sh — SysUtils installer for Linux
# ═══════════════════════════════════════════════════════════════════════════════

set -e

APP_NAME="SysUtils"
BINARY_NAME="sysutils_native"
INSTALL_DIR="$HOME/.local/share/sysutils"
BIN_LINK="$HOME/.local/bin/sysutils"
DESKTOP_FILE="$HOME/.local/share/applications/sysutils.desktop"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Colors ────────────────────────────────────────────────────────────────────
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

show_menu() {
    clear
    echo -e "${CYAN}=========================================${NC}"
    echo -e "${CYAN}              SYSUTILS                   ${NC}"
    echo -e "${CYAN}=========================================${NC}"
    echo ""
    echo "[1] Compilar y Probar (Modo Local)"
    echo "[2] Instalar SysUtils"
    echo "[3] Desinstalar SysUtils"
    echo "[4] Salir"
    echo ""
}

run_local() {
    echo -e "\n${YELLOW}[*] Compilando y ejecutando SysUtils...${NC}"
    cd "$SCRIPT_DIR"
    RUSTFLAGS="-C target-cpu=native" cargo run --release
}

install_app() {
    if ! command -v cargo &>/dev/null; then
        echo -e "${RED}Error: No se encontró 'cargo'. Instala Rust primero:${NC}"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        read -rp "Presiona ENTER para continuar..."
        return
    fi

    echo -e "\n${YELLOW}[*] Deteniendo SysUtils si está abierto...${NC}"
    pkill -f "$BINARY_NAME" 2>/dev/null || true

    echo -e "${YELLOW}[*] Compilando con optimizaciones máximas...${NC}"
    cd "$SCRIPT_DIR"
    RUSTFLAGS="-C target-cpu=native" cargo build --release

    echo -e "${YELLOW}[*] Instalando en $INSTALL_DIR ...${NC}"
    mkdir -p "$INSTALL_DIR"
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    if [ -f "assets/icon.png" ]; then
        cp "assets/icon.png" "$INSTALL_DIR/icon.png"
    fi

    mkdir -p "$HOME/.local/bin"
    ln -sf "$INSTALL_DIR/$BINARY_NAME" "$BIN_LINK"

    mkdir -p "$(dirname "$DESKTOP_FILE")"
    cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=$APP_NAME
Comment=Hardware Automation & Diagnostics Suite
Exec=$INSTALL_DIR/$BINARY_NAME
Icon=$INSTALL_DIR/icon.png
Terminal=false
Categories=Utility;
StartupNotify=true
EOF
    chmod +x "$DESKTOP_FILE"

    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi

    echo ""
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN} SysUtils instalado correctamente        ${NC}"
    echo -e "${GREEN} Binario: $INSTALL_DIR/$BINARY_NAME      ${NC}"
    echo -e "${GREEN} Comando: sysutils                       ${NC}"
    echo -e "${GREEN}=========================================${NC}"
    echo ""
    echo "Nota: asegúrate de que ~/.local/bin esté en tu PATH."
    echo "  Agrega esto a tu ~/.bashrc o ~/.zshrc si no está:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    read -rp "Presiona ENTER para volver al menú..."
}

uninstall_app() {
    echo -e "\n${YELLOW}[*] Deteniendo SysUtils...${NC}"
    pkill -f "$BINARY_NAME" 2>/dev/null || true
    sleep 1

    echo -e "${YELLOW}[*] Eliminando archivos ($INSTALL_DIR)...${NC}"
    rm -rf "$INSTALL_DIR"

    echo -e "${YELLOW}[*] Eliminando symlink y entrada de escritorio...${NC}"
    rm -f "$BIN_LINK"
    rm -f "$DESKTOP_FILE"

    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi

    echo ""
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN} SysUtils desinstalado.                  ${NC}"
    echo -e "${GREEN}=========================================${NC}"
    read -rp "Presiona ENTER para volver al menú..."
}

# ── Main loop ─────────────────────────────────────────────────────────────────
while true; do
    show_menu
    read -rp "Elige una opción: " choice
    case "$choice" in
        1) run_local ;;
        2) install_app ;;
        3) uninstall_app ;;
        4) exit 0 ;;
        *) echo -e "${RED}Opción inválida.${NC}"; sleep 1 ;;
    esac
done
