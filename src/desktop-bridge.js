(function () {
  const tauri = window.__TAURI__;
  if (!tauri?.core?.invoke) return;

  const invoke = tauri.core.invoke;
  const listen = tauri.event?.listen;
  let currentPath = '';

  function editor() {
    return window.MTCsvEditor || null;
  }

  function filenameFromPath(filePath) {
    return String(filePath || '').split(/[\\/]/).pop() || 'untitled.csv';
  }

  function currentEncoding() {
    const api = editor();
    return api?.getEncoding?.() || 'utf-8';
  }

  async function loadRecord(record) {
    if (!record || typeof record.contents !== 'string') return;
    currentPath = record.path || '';
    const api = editor();
    if (api) {
      api.loadText(record.contents, record.name || filenameFromPath(record.path), {
        encoding: record.encoding || 'utf-8'
      });
      api.setStatus(record.name ? `Opened ${record.name}` : 'Opened file');
    }
  }

  async function openUrls(urls) {
    const list = Array.isArray(urls) ? urls : [];
    for (const url of list) {
      try {
        const record = await invoke('read_opened_file', { url, encoding: currentEncoding() });
        await loadRecord(record);
        return;
      } catch (error) {
        console.error('Failed to open file', error);
      }
    }
  }

  window.MTCsvDesktop = {
    async openFile() {
      const record = await invoke('open_file_dialog', { encoding: currentEncoding() });
      await loadRecord(record);
    },

    async reopenWithEncoding(encoding) {
      const api = editor();
      if (!currentPath || api?.isDirty?.()) return false;
      const record = await invoke('read_opened_file', {
        url: currentPath,
        encoding: encoding || currentEncoding()
      });
      await loadRecord(record);
      return true;
    },

    async save() {
      const api = editor();
      if (!api) return;
      if (!currentPath) {
        await this.saveAs();
        return;
      }
      const saved = await invoke('save_file', {
        path: currentPath,
        contents: api.getText(),
        encoding: currentEncoding()
      });
      currentPath = saved.path || currentPath;
      api.setEncoding?.(saved.encoding || currentEncoding());
      api.markSaved(saved.name || filenameFromPath(currentPath));
      api.setStatus(saved.name ? `Saved ${saved.name}` : 'Saved file');
    },

    async saveAs() {
      const api = editor();
      if (!api) return;
      const saved = await invoke('save_file_dialog', {
        suggestedName: api.getFilename(),
        contents: api.getText(),
        encoding: currentEncoding()
      });
      if (!saved) return;
      currentPath = saved.path || '';
      api.setEncoding?.(saved.encoding || currentEncoding());
      api.markSaved(saved.name || filenameFromPath(currentPath));
      api.setStatus(saved.name ? `Saved ${saved.name}` : 'Saved file');
    }
  };

  async function init() {
    try {
      await openUrls(await invoke('opened_urls'));
      if (listen) {
        await listen('opened', (event) => openUrls(event.payload));
      }
    } catch (error) {
      console.error('Desktop bridge initialization failed', error);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init, { once: true });
  } else {
    init();
  }
})();
