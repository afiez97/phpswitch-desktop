#!/bin/bash
# uninstall.sh — Remove phpswitch from /usr/local/bin
set -e

INSTALL_PATH="/usr/local/bin/phpswitch"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

if [[ $EUID -ne 0 ]]; then
    echo -e "${RED}✗ Please run as root: sudo bash uninstall.sh${NC}"
    exit 1
fi

echo ""
echo -e "${BOLD}Uninstalling phpswitch...${NC}"

if [[ -f "${INSTALL_PATH}" ]]; then
    rm -f "${INSTALL_PATH}"
    echo -e "  ${GREEN}✓${NC} Removed ${INSTALL_PATH}"
else
    echo -e "  ${YELLOW}⚠${NC}  ${INSTALL_PATH} not found (already removed?)"
fi

echo ""
echo -e "${GREEN}Done!${NC}"
