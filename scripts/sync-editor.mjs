import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const sourcePath = path.resolve(root, '..', 'mathtalking', 'csv-editor.html');
const outputDir = path.join(root, 'src');
const outputPath = path.join(outputDir, 'index.html');

const desktopApi = `
  window.MTCsvEditor = {
    loadText(text, name) {
      loadFromText(text || '', name || 'Untitled.csv', null);
      state.dirty = false;
      updateTitle();
    },
    getText() {
      return buildCsv('csv');
    },
    getFilename() {
      return state.filename || 'untitled.csv';
    },
    markSaved(name) {
      if (name) state.filename = name;
      state.dirty = false;
      updateTitle();
    },
    setStatus(message) {
      if (message) $('#stats').textContent = message;
    },
    isDirty() {
      return !!state.dirty;
    }
  };

`;

function replaceOnce(input, from, to) {
  if (!input.includes(from)) {
    throw new Error(`Cannot find sync marker: ${from.slice(0, 80)}`);
  }
  return input.replace(from, to);
}

let html = fs.readFileSync(sourcePath, 'utf8').replace(/\r\n/g, '\n');

html = replaceOnce(
  html,
  '<a class="csv-brand" href="https://mathtalking.com/">MathTalking</a>',
  '<span class="csv-brand">MathTalking</span>'
);

html = replaceOnce(
  html,
  '  async function openFile() {\n',
  "  async function openFile() {\n    if (window.MTCsvDesktop?.openFile) return window.MTCsvDesktop.openFile();\n"
);

html = replaceOnce(
  html,
  "  async function save() {\n    const text = buildCsv('csv');\n",
  "  async function save() {\n    if (window.MTCsvDesktop?.save) return window.MTCsvDesktop.save();\n    const text = buildCsv('csv');\n"
);

html = replaceOnce(
  html,
  "  async function saveAs() {\n    const text = buildCsv('csv');\n",
  "  async function saveAs() {\n    if (window.MTCsvDesktop?.saveAs) return window.MTCsvDesktop.saveAs();\n    const text = buildCsv('csv');\n"
);

html = replaceOnce(
  html,
  '  installImportBridge();\n',
  desktopApi + '  installImportBridge();\n'
);

html = replaceOnce(
  html,
  '</body>',
  '  <script defer src="./desktop-bridge.js"></script>\n</body>'
);

fs.mkdirSync(outputDir, { recursive: true });
fs.writeFileSync(outputPath, html);
console.log(`Synced ${path.relative(root, sourcePath)} -> ${path.relative(root, outputPath)}`);
