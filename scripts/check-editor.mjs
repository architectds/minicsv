import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const indexPath = path.join(root, 'src', 'index.html');
const bridgePath = path.join(root, 'src', 'desktop-bridge.js');

const html = fs.readFileSync(indexPath, 'utf8');
const bridge = fs.readFileSync(bridgePath, 'utf8');

const checks = [
  ['standalone desktop title', '<title>CSV Editor — MathTalking</title>'],
  ['desktop editor API', 'window.MTCsvEditor'],
  ['desktop open bridge', 'window.MTCsvDesktop?.openFile'],
  ['desktop save bridge', 'window.MTCsvDesktop?.save'],
  ['desktop save-as bridge', 'window.MTCsvDesktop?.saveAs'],
  ['encoding selector', 'id="sel-encoding"'],
  ['desktop encoding API', 'getEncoding()'],
  ['encoding reload notice', 'encodingSaveOnlyDirty'],
  ['desktop bridge script', './desktop-bridge.js'],
  ['latest lazy cell editor', 'input.cell-edit'],
  ['latest stable cell ghost', 'cell-ghost'],
  ['latest virtual row renderer', 'renderVisibleRows'],
  ['latest virtual row window', '_firstRendered'],
];

const bridgeChecks = [
  ['native encoding bridge', 'encoding: currentEncoding()'],
  ['desktop re-decode bridge', 'reopenWithEncoding'],
];

const missing = checks.filter(([, marker]) => !html.includes(marker));
missing.push(...bridgeChecks.filter(([, marker]) => !bridge.includes(marker)));
if (missing.length) {
  for (const [label, marker] of missing) {
    console.error(`Missing ${label}: ${marker}`);
  }
  process.exit(1);
}

console.log('Standalone editor check passed.');
