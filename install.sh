#!/bin/bash
# install.sh — Install phpswitch to /usr/local/bin
set -e

INSTALL_PATH="/usr/local/bin/phpswitch"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

if [[ $EUID -ne 0 ]]; then
    echo -e "${RED}✗ Please run as root: sudo bash install.sh${NC}"
    exit 1
fi

echo ""
echo -e "${BOLD}Installing phpswitch...${NC}"

# Copy script and make executable
install -m 755 "${SCRIPT_DIR}/phpswitch" "${INSTALL_PATH}"

echo -e "  ${GREEN}✓${NC} Installed $("${INSTALL_PATH}" --version) to ${INSTALL_PATH}"
echo ""
echo -e "${BOLD}Usage:${NC}"
echo -e "  ${CYAN}phpswitch --status${NC}    Show current PHP versions"
echo -e "  ${CYAN}sudo phpswitch${NC}        Interactive menu"
echo -e "  ${CYAN}sudo phpswitch 8.4${NC}    Switch directly to PHP 8.4"
echo ""
echo -e "${GREEN}Done!${NC}"
