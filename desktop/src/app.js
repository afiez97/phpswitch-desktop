const { invoke } = window.__TAURI__.core;

const ACCENT = "oklch(0.55 0.17 280)";
const GREEN = "oklch(0.6 0.15 150)";
const AMBER = "oklch(0.68 0.15 75)";
const GRAY = "oklch(0.7 0.01 265)";

let status = { cli: "?", apache: null, activeFpm: [], apacheRunning: false, nginxRunning: false, versions: [] };
let dirty = false;
let busyRescan = false;
let busyRestart = false;
let busyAction = false;
let busyTarget = null;
const anyBusy = () => busyRescan || busyRestart || busyAction;

function actionButton({ label, active, hasCapability = true, command, version }) {
  const btn = document.createElement('button');
  const key = command + ':' + version;
  const isThisBusy = busyTarget === key;
  btn.style.cssText = segBtn(active);
  btn.disabled = !hasCapability || anyBusy();
  if (isThisBusy) {
    btn.innerHTML = '<span style="display:inline-block; animation: spin .8s linear infinite;">⟳</span>';
  } else {
    btn.textContent = label;
    if (!hasCapability || anyBusy()) btn.style.opacity = '0.4';
  }
  btn.onclick = () => runAction(command, version);
  return btn;
}

function segBtn(active) {
  const base = "padding: 6px 11px; border-radius: 7px; font-family: 'Inter', sans-serif; font-size: 12px; font-weight: 600; cursor: pointer; transition: all .12s; min-width: 52px; text-align: center;";
  if (active) return base + "background: " + ACCENT + "; color: white; border: 1px solid " + ACCENT + ";";
  return base + "background: oklch(0.98 0.002 265); color: oklch(0.42 0.02 265); border: 1px solid oklch(0.86 0.008 265);";
}

function render() {
  const el = (id) => document.getElementById(id);

  el('active-version').textContent = 'PHP ' + status.cli;
  el('active-path').textContent = status.versions.find(v => v.version === status.cli)?.path ?? '';

  const apacheSupported = status.apache !== null && status.apache !== undefined;
  el('server-targets').style.gridTemplateColumns = apacheSupported ? '1fr 1fr' : '1fr';
  el('apache-card').style.display = apacheSupported ? '' : 'none';

  if (apacheSupported) {
    const apacheColor = dirty ? AMBER : (status.apacheRunning ? GREEN : GRAY);
    el('apache-version').textContent = 'PHP ' + status.apache;
    el('apache-pkg').textContent = 'libapache2-mod-php' + status.apache;
    el('apache-status-dot').style.background = apacheColor;
    el('apache-status-label').style.color = apacheColor;
    el('apache-status-label').textContent = dirty ? 'pending' : (status.apacheRunning ? 'running' : 'stopped');
  }

  const nginxVer = status.activeFpm && status.activeFpm.length ? status.activeFpm[0] : status.cli;
  const nginxColor = dirty ? AMBER : (status.nginxRunning ? GREEN : GRAY);
  el('nginx-version').textContent = 'PHP ' + nginxVer;
  el('nginx-pkg').textContent = 'php' + nginxVer + '-fpm';
  el('nginx-status-dot').style.background = nginxColor;
  el('nginx-status-label').style.color = nginxColor;
  el('nginx-status-label').textContent = dirty ? 'pending' : (status.nginxRunning ? 'running' : 'stopped');

  el('installed-count').textContent = status.versions.length + ' installed';

  const list = el('versions-list');
  list.innerHTML = '';
  status.versions.forEach((v) => {
    const isCli = v.version === status.cli;
    const isApache = apacheSupported && v.version === status.apache;
    const isNginx = status.activeFpm.includes(v.version);
    const anyActive = isCli || isApache || isNginx;

    const row = document.createElement('div');
    row.style.cssText = "display: flex; align-items: center; gap: 14px; padding: 12px 14px; border-radius: 10px; border: 1px solid " +
      (anyActive ? "oklch(0.8 0.06 280)" : "oklch(0.91 0.006 265)") + "; background: " +
      (anyActive ? "oklch(0.975 0.012 280)" : "oklch(0.99 0.002 265)") + ";";

    const badgeStyle = isCli
      ? "font-size: 10px; font-weight: 700; letter-spacing: 0.05em; padding: 3px 8px; border-radius: 20px; background: " + ACCENT + "; color: white;"
      : "font-size: 10px; font-weight: 600; letter-spacing: 0.05em; padding: 3px 8px; border-radius: 20px; background: oklch(0.94 0.004 265); color: oklch(0.5 0.01 265);";

    row.innerHTML = `
      <span style="font-family: 'JetBrains Mono', monospace; font-size: 15px; font-weight: 600; color: oklch(0.28 0.02 265); min-width: 64px;">PHP ${v.version}</span>
      <span style="${badgeStyle}">${isCli ? 'CLI DEFAULT' : 'installed'}</span>
      <span style="flex: 1; font-family: 'JetBrains Mono', monospace; font-size: 12px; color: oklch(0.58 0.01 265);">${v.path}</span>
      <div style="display: flex; gap: 7px;" data-actions></div>
    `;

    const actions = row.querySelector('[data-actions]');

    if (apacheSupported) {
      actions.appendChild(actionButton({
        label: 'Apache', active: isApache, hasCapability: v.hasApache,
        command: 'set_apache', version: v.version,
      }));
    }

    actions.appendChild(actionButton({
      label: 'Nginx', active: isNginx, hasCapability: v.hasFpm,
      command: 'set_fpm', version: v.version,
    }));

    actions.appendChild(actionButton({
      label: isCli ? '✓ CLI' : 'CLI', active: isCli,
      command: 'set_cli', version: v.version,
    }));

    list.appendChild(row);
  });

  el('rescan-label').textContent = busyRescan ? 'Scanning…' : 'Rescan';
  el('rescan-icon').style.animation = busyRescan ? 'spin 0.9s linear infinite' : '';
  el('btn-rescan').disabled = anyBusy();

  el('restart-label').textContent = busyRestart ? 'Restarting…' : 'Restart web servers';
  el('restart-icon').style.animation = busyRestart ? 'spin 0.9s linear infinite' : '';
  el('btn-restart').style.background = dirty ? ACCENT : GRAY;
  el('btn-restart').disabled = anyBusy();
}

function setLog(text, kind) {
  const colors = { ok: GREEN, warn: AMBER, busy: ACCENT, idle: 'oklch(0.55 0.01 265)' };
  const el = document.getElementById('log-text');
  el.textContent = text;
  el.style.color = colors[kind] || colors.idle;
}

async function loadStatus() {
  try {
    status = await invoke('get_status');
    setLog('Ready.', 'idle');
  } catch (e) {
    setLog(String(e), 'warn');
  }
  render();
}

function isDryRun() {
  return document.getElementById('chk-dry-run').checked;
}

async function runAction(command, version) {
  if (anyBusy()) return;
  const dryRun = isDryRun();
  busyAction = true; busyTarget = command + ':' + version; render();
  const verb = command === 'set_cli' ? 'Switching CLI to PHP ' + version : 'Applying change';
  setLog((dryRun ? '[dry-run] Previewing: ' : '') + verb + '…', 'busy');
  try {
    const data = await invoke(command, { version, dryRun });
    if (data.status) status = data.status;
    if (!dryRun && (command === 'set_apache' || command === 'set_fpm')) dirty = true;
    setLog((dryRun && data.ok ? '[dry-run] ' : '') + data.log, data.logKind);
  } catch (e) {
    setLog(String(e), 'warn');
  } finally {
    busyAction = false;
    busyTarget = null;
    render();
  }
}

document.getElementById('btn-rescan').addEventListener('click', async () => {
  if (anyBusy()) return;
  busyRescan = true; render();
  setLog('Scanning installed PHP versions…', 'busy');
  try {
    const data = await invoke('rescan');
    if (data.status) status = data.status;
    setLog(data.log, data.logKind);
  } catch (e) {
    setLog(String(e), 'warn');
  } finally {
    busyRescan = false;
    render();
  }
});

document.getElementById('btn-restart').addEventListener('click', async () => {
  if (anyBusy()) return;
  const dryRun = isDryRun();
  busyRestart = true; render();
  setLog((dryRun ? '[dry-run] Previewing: r' : 'R') + 'estarting web servers…', 'busy');
  try {
    const data = await invoke('restart_services', { dryRun });
    if (data.status) status = data.status;
    if (!dryRun) dirty = false;
    setLog((dryRun && data.ok ? '[dry-run] ' : '') + data.log, data.logKind);
  } catch (e) {
    setLog(String(e), 'warn');
  } finally {
    busyRestart = false;
    render();
  }
});

render();
loadStatus();
