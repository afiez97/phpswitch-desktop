#!/usr/bin/env bash
# Shared setup for phpswitch.bats.
#
# Builds a per-test fake filesystem root (so PHP_BIN_DIR/APACHE_MODS_*/
# NGINX_*/FPM_SOCK_DIR never point at the real system) and a per-test mock-bin
# directory prepended to PATH (so systemctl/apache2ctl/a2enmod/a2dismod/
# update-alternatives/nginx/composer never run for real). The mock-bin copy is
# per-test (not the repo's tests/mock-bin directly) so a test that removes a
# mock (e.g. to simulate "apache2ctl not installed") can't affect other tests
# or the repo itself.

PHPSWITCH_SCRIPT="${BATS_TEST_DIRNAME}/../phpswitch"

setup_fake_root() {
    export FAKE_ROOT="${BATS_TEST_TMPDIR}/root"
    export MOCK_STATE_DIR="${BATS_TEST_TMPDIR}/mock-state"
    export MOCK_BIN_DIR="${BATS_TEST_TMPDIR}/mock-bin"
    export MOCK_LOG="${BATS_TEST_TMPDIR}/mock.log"

    mkdir -p \
        "${FAKE_ROOT}/bin" \
        "${FAKE_ROOT}/apache-available" \
        "${FAKE_ROOT}/apache-enabled" \
        "${FAKE_ROOT}/nginx-sites" \
        "${FAKE_ROOT}/nginx-confd" \
        "${FAKE_ROOT}/fpm-sock" \
        "${MOCK_STATE_DIR}/systemd" \
        "${MOCK_STATE_DIR}/alternatives"

    touch "${MOCK_STATE_DIR}/systemd/units.installed" \
          "${MOCK_STATE_DIR}/systemd/units.active" \
          "${MOCK_STATE_DIR}/systemd/fail" \
          "${MOCK_STATE_DIR}/alternatives/registered" \
          "$MOCK_LOG"

    cp -r "${BATS_TEST_DIRNAME}/mock-bin" "$MOCK_BIN_DIR"

    export PHP_BIN_DIR="${FAKE_ROOT}/bin"
    export APACHE_MODS_AVAILABLE="${FAKE_ROOT}/apache-available"
    export APACHE_MODS_ENABLED="${FAKE_ROOT}/apache-enabled"
    export NGINX_SITES_ENABLED="${FAKE_ROOT}/nginx-sites"
    export NGINX_CONF_D="${FAKE_ROOT}/nginx-confd"
    export FPM_SOCK_DIR="${FAKE_ROOT}/fpm-sock"

    # Deliberately narrow PATH: real apache2ctl/nginx/a2enmod/a2dismod live
    # in /usr/sbin and real composer in /usr/local/bin on a dev box with
    # these installed — if those stay on PATH, a test that removes a mock
    # (e.g. to simulate "apache2ctl not installed") would silently fall
    # through to the real system binary instead of getting "not found".
    # /usr/bin and /bin cover the core utilities (readlink, sed, grep, awk,
    # sort, ln, ...) the script and mocks themselves depend on.
    export PATH="${MOCK_BIN_DIR}:/usr/bin:/bin"
}

# Removes one mock binary from this test's private mock-bin copy, so
# `command -v <name>` fails — simulates "<name> isn't installed."
remove_mock() {
    rm -f "${MOCK_BIN_DIR:?}/$1"
}

# Creates an executable fake PHP binary at $PHP_BIN_DIR/php<ver> that reports
# its own version via --version (resolved through readlink -f so it still
# works when invoked through the "php" alternatives symlink).
make_php_version() {
    local ver="$1"
    cat > "${PHP_BIN_DIR}/php${ver}" <<'STUB'
#!/usr/bin/env bash
self=$(readlink -f "$0")
ver=$(basename "$self" | sed -E 's/^php//')
case "$1" in
    --version|-v) echo "PHP ${ver}.0 (cli) (built: mock)" ;;
    *)            echo "PHP ${ver}.0 (cli)" ;;
esac
STUB
    chmod +x "${PHP_BIN_DIR}/php${ver}"
}

# Points $PHP_BIN_DIR/php at the given (already-created) version, simulating
# a prior `update-alternatives --set`.
set_current_cli() {
    ln -sf "php${1}" "${PHP_BIN_DIR}/php"
}

# Registers a companion tool (phpize, php-config, ...) as a known
# update-alternatives name, and creates its versioned binary.
register_alternative() {
    local name="$1" ver="$2"
    echo "$name" >> "${MOCK_STATE_DIR}/alternatives/registered"
    : > "${PHP_BIN_DIR}/${name}${ver}"
    chmod +x "${PHP_BIN_DIR}/${name}${ver}"
}

seed_apache_module() {
    touch "${APACHE_MODS_AVAILABLE}/php${1}.conf"
}

enable_apache_module() {
    seed_apache_module "$1"
    ln -sf "${APACHE_MODS_AVAILABLE}/php${1}.conf" "${APACHE_MODS_ENABLED}/php${1}.conf"
}

# Registers a systemd unit as "installed" (visible in list-unit-files).
seed_unit() {
    grep -qxF "$1" "${MOCK_STATE_DIR}/systemd/units.installed" 2>/dev/null || \
        echo "$1" >> "${MOCK_STATE_DIR}/systemd/units.installed"
}

# Registers + marks a systemd unit as currently active.
activate_unit() {
    seed_unit "$1"
    grep -qxF "$1" "${MOCK_STATE_DIR}/systemd/units.active" 2>/dev/null || \
        echo "$1" >> "${MOCK_STATE_DIR}/systemd/units.active"
}

# Forces the mock systemctl to fail a specific "<subcommand> <unit>" call,
# e.g. `fail_unit restart apache2`.
fail_unit() {
    echo "$1:$2" >> "${MOCK_STATE_DIR}/systemd/fail"
}

# Forces the mock update-alternatives to fail `--set <name> ...`.
fail_alternative_set() {
    touch "${MOCK_STATE_DIR}/alternatives/fail-set-${1}"
}

seed_nginx_site() {
    local name="$1" ver="$2"
    cat > "${NGINX_SITES_ENABLED}/${name}" <<EOF
server {
    location ~ \.php\$ {
        fastcgi_pass unix:${FPM_SOCK_DIR}/php${ver}-fpm.sock;
    }
}
EOF
}
