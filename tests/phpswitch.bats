#!/usr/bin/env bats
# Tests for the phpswitch bash CLI. All system-mutating commands
# (update-alternatives, a2enmod/a2dismod, systemctl, nginx) are mocked under
# tests/mock-bin/ — see tests/test_helper.bash for the fake-root/mock setup.
#
# Footgun: functions reachable from main()'s dispatch (check_root,
# validate_version_arg, the unknown-argument branch) and switch_version call
# a bare `exit`. Any test invoking one of those — or anything that can return
# nonzero, since bats treats a nonzero non-`run` command as a test failure —
# MUST use bats' `run` wrapper (which subshells), never plain invocation or
# command substitution.

load 'test_helper'

setup() {
    setup_fake_root
    # shellcheck source=/dev/null
    source "$PHPSWITCH_SCRIPT"
    # main()'s --set-*/version dispatch arms call check_root first; stub it
    # so dispatch-layer tests can run as a non-root bats process. Tests that
    # call switch_cli/switch_apache/switch_fpm/switch_version directly never
    # reach check_root at all.
    check_root() { :; }
}

# ── Discovery ────────────────────────────────────────────

@test "get_installed_versions sorts detected php binaries, single and double digit minors" {
    make_php_version 8.0
    make_php_version 8.10
    make_php_version 8.3
    run get_installed_versions
    [ "$status" -eq 0 ]
    [[ "$output" == *"8.0"*"8.3"*"8.10"* ]]
}

@test "has_apache_module true only when the mods-available conf exists" {
    seed_apache_module 8.3
    run has_apache_module 8.3
    [ "$status" -eq 0 ]
    run has_apache_module 8.4
    [ "$status" -ne 0 ]
}

@test "has_fpm true only when the systemd unit file is registered" {
    seed_unit "php8.3-fpm"
    run has_fpm 8.3
    [ "$status" -eq 0 ]
    run has_fpm 8.4
    [ "$status" -ne 0 ]
}

# ── switch_cli ───────────────────────────────────────────

@test "switch_cli points PHP_BIN_DIR/php at the target version and returns 0" {
    make_php_version 8.1
    make_php_version 8.3
    set_current_cli 8.1

    run switch_cli 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"CLI"* ]]
    run get_current_cli
    [[ "$output" == "8.3" ]]
}

@test "switch_cli only switches companion tools that are registered alternatives" {
    make_php_version 8.3
    register_alternative phpize 8.3
    run switch_cli 8.3
    [ "$status" -eq 0 ]
    grep -q "update-alternatives --set phpize" "$MOCK_LOG"
    ! grep -q "update-alternatives --set php-config" "$MOCK_LOG"
}

@test "switch_cli fails (returns 1) when update-alternatives --set fails" {
    make_php_version 8.3
    fail_alternative_set php
    run switch_cli 8.3
    [ "$status" -eq 1 ]
}

# ── switch_apache ────────────────────────────────────────

@test "switch_apache skips (2) when apache2ctl isn't installed" {
    remove_mock apache2ctl
    run switch_apache 8.3
    [ "$status" -eq 2 ]
    [[ "$output" == *"Apache not found"* ]]
}

@test "switch_apache skips (2) when the module for that version isn't installed" {
    run switch_apache 8.9
    [ "$status" -eq 2 ]
    [[ "$output" == *"not installed"* ]]
}

@test "switch_apache disables the old module, enables the new one, and restarts" {
    enable_apache_module 8.1
    seed_apache_module 8.3
    run switch_apache 8.3
    [ "$status" -eq 0 ]
    [[ ! -e "${APACHE_MODS_ENABLED}/php8.1.conf" ]]
    [[ -e "${APACHE_MODS_ENABLED}/php8.3.conf" ]]
    grep -q "systemctl restart apache2" "$MOCK_LOG"
}

@test "switch_apache fails (returns 1) when apache2 restart fails" {
    seed_apache_module 8.3
    fail_unit restart apache2
    run switch_apache 8.3
    [ "$status" -eq 1 ]
}

# ── switch_fpm ───────────────────────────────────────────

@test "switch_fpm skips (2) when no fpm units exist at all" {
    run switch_fpm 8.3
    [ "$status" -eq 2 ]
}

@test "switch_fpm skips (2) when the target version's unit isn't installed" {
    seed_unit "php8.1-fpm"
    run switch_fpm 8.3
    [ "$status" -eq 2 ]
    [[ "$output" == *"not installed"* ]]
}

@test "switch_fpm stops the stale active version and starts the target" {
    seed_unit "php8.1-fpm"
    activate_unit "php8.1-fpm"
    seed_unit "php8.3-fpm"

    run switch_fpm 8.3
    [ "$status" -eq 0 ]

    run get_active_fpm_versions
    [[ "$output" == *"8.3"* ]]
    [[ "$output" != *"8.1"* ]]
}

@test "switch_fpm fails (returns 1) when the target fpm restart fails" {
    seed_unit "php8.3-fpm"
    fail_unit restart php8.3-fpm
    run switch_fpm 8.3
    [ "$status" -eq 1 ]
}

@test "switch_fpm reports NGINX_STATUS=skipped when nginx isn't running" {
    seed_unit "php8.3-fpm"
    switch_fpm 8.3 >/dev/null
    [ "$NGINX_STATUS" = "skipped" ]
}

@test "switch_fpm reports NGINX_STATUS=ok and reloads nginx when it's running" {
    seed_unit "php8.3-fpm"
    activate_unit "nginx"
    switch_fpm 8.3 >/dev/null
    [ "$NGINX_STATUS" = "ok" ]
    grep -q "systemctl reload nginx" "$MOCK_LOG"
}

# ── update_nginx_fpm_socket ──────────────────────────────

@test "update_nginx_fpm_socket rewrites a stale fastcgi_pass socket in place" {
    seed_nginx_site default.conf 8.1
    run update_nginx_fpm_socket 8.3
    [ "$status" -eq 0 ]
    grep -q "php8.3-fpm.sock" "${NGINX_SITES_ENABLED}/default.conf"
    ! grep -q "php8.1-fpm.sock" "${NGINX_SITES_ENABLED}/default.conf"
}

# ── JSON status ──────────────────────────────────────────

@test "--json-status path field reflects the overridden PHP_BIN_DIR" {
    make_php_version 8.3
    set_current_cli 8.3
    run print_json_status
    [ "$status" -eq 0 ]
    [[ "$output" == *"\"path\":\"${PHP_BIN_DIR}/php8.3\""* ]]
}

# ── validate_version_arg ─────────────────────────────────

@test "validate_version_arg rejects a malformed version string" {
    run validate_version_arg "8.x"
    [ "$status" -eq 1 ]
}

@test "validate_version_arg rejects a version with no installed binary" {
    run validate_version_arg "8.9"
    [ "$status" -eq 1 ]
}

@test "validate_version_arg accepts an installed version" {
    make_php_version 8.3
    run validate_version_arg "8.3"
    [ "$status" -eq 0 ]
}

# ── main() dispatch — the critical skip-vs-fail translation fix ─────────

@test "main --set-apache exits 0 (not 2) when Apache is intentionally skipped" {
    make_php_version 8.3
    remove_mock apache2ctl
    run main --set-apache 8.3
    [ "$status" -eq 0 ]
}

@test "main --set-fpm exits 0 (not 2) when FPM is intentionally skipped" {
    make_php_version 8.3
    run main --set-fpm 8.3
    [ "$status" -eq 0 ]
}

@test "main --set-apache exits 1 on a real failure" {
    make_php_version 8.3
    seed_apache_module 8.3
    fail_unit restart apache2
    run main --set-apache 8.3
    [ "$status" -eq 1 ]
}

@test "main --set-fpm exits 1 on a real failure" {
    make_php_version 8.3
    seed_unit "php8.3-fpm"
    fail_unit restart php8.3-fpm
    run main --set-fpm 8.3
    [ "$status" -eq 1 ]
}

# ── switch_version aggregate summary ─────────────────────

@test "switch_version exits 0 and reports skipped components when nothing is installed but CLI" {
    make_php_version 8.3
    run switch_version 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"Summary"* ]]
    [[ "$output" == *"Done!"* ]]
}

@test "switch_version exits 1 and reports a failed component in the summary" {
    make_php_version 8.3
    seed_apache_module 8.3
    fail_unit restart apache2
    run switch_version 8.3
    [ "$status" -eq 1 ]
    [[ "$output" == *"Completed with errors"* ]]
}

# ── Dry-run mode ─────────────────────────────────────────

@test "--dry-run makes switch_cli a no-op" {
    make_php_version 8.1
    make_php_version 8.3
    set_current_cli 8.1
    DRY_RUN=1

    run switch_cli 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"[dry-run]"* ]]
    ! grep -q "update-alternatives --set" "$MOCK_LOG"
    run get_current_cli
    [[ "$output" == "8.1" ]]
}

@test "--dry-run makes switch_apache a no-op" {
    enable_apache_module 8.1
    seed_apache_module 8.3
    DRY_RUN=1

    run switch_apache 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"[dry-run]"* ]]
    [[ -e "${APACHE_MODS_ENABLED}/php8.1.conf" ]]
    [[ ! -e "${APACHE_MODS_ENABLED}/php8.3.conf" ]]
    ! grep -q "a2enmod\|a2dismod\|systemctl restart apache2" "$MOCK_LOG"
}

@test "--dry-run makes switch_fpm and the nginx socket rewrite a no-op" {
    seed_unit "php8.1-fpm"
    activate_unit "php8.1-fpm"
    seed_unit "php8.3-fpm"
    seed_nginx_site default.conf 8.1
    DRY_RUN=1

    run switch_fpm 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"[dry-run]"* ]]

    run get_active_fpm_versions
    [[ "$output" == *"8.1"* ]]
    [[ "$output" != *"8.3"* ]]
    grep -q "php8.1-fpm.sock" "${NGINX_SITES_ENABLED}/default.conf"
    ! grep -q "systemctl restart php8.3-fpm\|systemctl stop php8.1-fpm" "$MOCK_LOG"
}

@test "check_root is a no-op under --dry-run, regardless of EUID" {
    # setup() stubs check_root() for the dispatch tests above — re-source to
    # get the real implementation back before testing it directly.
    source "$PHPSWITCH_SCRIPT"
    DRY_RUN=1
    run check_root
    [ "$status" -eq 0 ]
}

@test "check_root still requires root when DRY_RUN=0 (this test process is non-root)" {
    source "$PHPSWITCH_SCRIPT"
    DRY_RUN=0
    run check_root
    [ "$status" -eq 1 ]
}

@test "--dry-run is recognized in any argv position" {
    make_php_version 8.3
    run main --dry-run 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"Dry run only"* ]]

    run main 8.3 --dry-run
    [ "$status" -eq 0 ]
    [[ "$output" == *"Dry run only"* ]]
}

@test "switch_version summary says 'Dry run only' and never 'Done!' under --dry-run" {
    make_php_version 8.3
    DRY_RUN=1
    run switch_version 8.3
    [ "$status" -eq 0 ]
    [[ "$output" == *"Dry run only"* ]]
    [[ "$output" != *"Done!"* ]]
}
