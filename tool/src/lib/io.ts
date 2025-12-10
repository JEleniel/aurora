import type {
  SaveDialogOptions,
  OpenDialogOptions,
} from '@tauri-apps/api/dialog';

// Tauri-aware file helpers with graceful web fallbacks.
export async function saveText(filename: string, text: string) {
  // Try Tauri fs first
  try {
    // dynamic import to avoid throwing in non-tauri environments
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const { dialog } = await import('@tauri-apps/api/dialog');
    const { fs } = await import('@tauri-apps/api');
    const path = await dialog.save({
      defaultPath: filename,
    } as SaveDialogOptions);
    if (path) {
      await fs.writeFile({ path, contents: text });
      return path;
    }
  } catch (e) {
    // fallback to browser download
  }

  // Browser fallback: create blob and trigger download
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
  return null;
}

export async function openText(): Promise<{
  path?: string;
  contents?: string;
} | null> {
  try {
    const { dialog } = await import('@tauri-apps/api/dialog');
    const { fs } = await import('@tauri-apps/api');
    const selected = (await dialog.open({ multiple: false })) as string | null;
    if (selected) {
      const contents = await fs.readText(selected);
      return { path: selected, contents };
    }
  } catch (e) {
    // fallback to file input
  }

  return await browserOpenFallback();
}

// Open a ZIP archive and return array of entries (name + contents as string)
export async function openArchive(): Promise<{
  path?: string;
  entries: Array<{ name: string; contents: string }>;
} | null> {
  // Try Tauri first
  try {
    const JSZip = (await import('jszip')).default;
    const { dialog } = await import('@tauri-apps/api/dialog');
    const { fs } = await import('@tauri-apps/api/fs');
    const selected = (await dialog.open({ multiple: false })) as string | null;
    if (selected) {
      // read as binary
      const uint8 = await fs.readBinaryFile(selected);
      const zip = await JSZip.loadAsync(uint8 as Uint8Array);
      const entries: Array<{ name: string; contents: string }> = [];
      await Promise.all(
        Object.keys(zip.files).map(async (f) => {
          if (zip.files[f].dir) return;
          const txt = await zip.files[f].async('string');
          entries.push({ name: f, contents: txt });
        })
      );
      return { path: selected, entries };
    }
  } catch (e) {
    // fallback to browser file input
  }

  return await browserOpenArchiveFallback();
}

async function browserOpenArchiveFallback(): Promise<{
  path?: string;
  entries: Array<{ name: string; contents: string }>;
} | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.zip';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      const JSZip = (await import('jszip')).default;
      const buf = await file.arrayBuffer();
      const zip = await JSZip.loadAsync(buf);
      const entries: Array<{ name: string; contents: string }> = [];
      await Promise.all(
        Object.keys(zip.files).map(async (f) => {
          if (zip.files[f].dir) return;
          const txt = await zip.files[f].async('string');
          entries.push({ name: f, contents: txt });
        })
      );
      resolve({ path: file.name, entries });
    };
    input.click();
  });
}

async function browserOpenFallback(): Promise<{
  path?: string;
  contents?: string;
} | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json,.md,.mmd,.txt';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      const contents = await file.text();
      resolve({ path: file.name, contents });
    };
    input.click();
  });
}

export async function saveSvg(filename: string, svg: string) {
  // write as .svg
  return saveText(
    filename.endsWith('.svg') ? filename : `${filename}.svg`,
    svg
  );
}

// Save or update an entry inside a ZIP archive. If the archive exists (Tauri),
// it will be read, updated, and written back. In the browser fallback we will
// generate a new ZIP and trigger a download.
export async function saveArchiveEntry(
  archivePath: string | undefined | null,
  entryName: string,
  contents: string
) {
  const JSZip = (await import('jszip')).default;
  // Try Tauri write
  try {
    const { fs } = await import('@tauri-apps/api/fs');
    // read binary
    const exists = !!archivePath;
    let zip: any;
    if (exists && archivePath) {
      try {
        const uint8 = await fs.readBinaryFile(archivePath);
        zip = await JSZip.loadAsync(uint8 as Uint8Array);
      } catch (e) {
        zip = new JSZip();
      }
    } else {
      zip = new JSZip();
    }

    zip.file(entryName, contents);
    const out = await zip.generateAsync({ type: 'uint8array' });
    // write back
    if (archivePath) {
      await fs.writeFile({ path: archivePath, contents: out });
      return archivePath;
    }
  } catch (e) {
    // fallback to browser download
  }

  // Browser fallback: offer download of updated zip
  const zip = new JSZip();
  zip.file(entryName, contents);
  const blob = await zip.generateAsync({ type: 'blob' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = archivePath
    ? archivePath.split('/').pop() || 'aurora.zip'
    : 'aurora.zip';
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
  return null;
}

// Save a full archive with multiple entries. `entries` is an array of { name, contents }.
export async function saveArchive(
  archivePath: string | undefined | null,
  entries: Array<{ name: string; contents: string }>
) {
  const JSZip = (await import('jszip')).default;
  try {
    const { fs } = await import('@tauri-apps/api/fs');
    let zip: any;
    if (archivePath) {
      try {
        const uint8 = await fs.readBinaryFile(archivePath);
        zip = await JSZip.loadAsync(uint8 as Uint8Array);
      } catch (e) {
        zip = new JSZip();
      }
    } else {
      zip = new JSZip();
    }

    for (const e of entries) {
      zip.file(e.name, e.contents);
    }

    const out = await zip.generateAsync({ type: 'uint8array' });
    if (archivePath) {
      await fs.writeFile({ path: archivePath, contents: out });
      return archivePath;
    }
  } catch (e) {
    // fallback to browser
  }

  const zip = new JSZip();
  for (const e of entries) zip.file(e.name, e.contents);
  const blob = await zip.generateAsync({ type: 'blob' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = archivePath
    ? archivePath.split('/').pop() || 'aurora.zip'
    : 'aurora.zip';
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
  return null;
}

// Recent files: prefer Tauri app-local storage, fallback to localStorage
export async function getRecentFiles(): Promise<string[]> {
  try {
    const { appLocalDataDir } = await import('@tauri-apps/api/path');
    const { fs } = await import('@tauri-apps/api');
    const dir = await appLocalDataDir();
    const path = `${dir}/aurora_recent.json`;
    try {
      const contents = await fs.readText(path);
      const parsed = JSON.parse(contents);
      return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
      return [];
    }
  } catch (e) {
    // fallback to localStorage
    try {
      const raw = localStorage.getItem('aurora_recent');
      const parsed = raw ? JSON.parse(raw) : [];
      return Array.isArray(parsed) ? parsed : [];
    } catch (e2) {
      return [];
    }
  }
}

export async function addRecentFile(p: string) {
  if (!p) return;
  try {
    const { appLocalDataDir } = await import('@tauri-apps/api/path');
    const { fs } = await import('@tauri-apps/api');
    const dir = await appLocalDataDir();
    const path = `${dir}/aurora_recent.json`;
    let items: string[] = [];
    try {
      const contents = await fs.readText(path);
      items = JSON.parse(contents);
      if (!Array.isArray(items)) items = [];
    } catch (e) {
      items = [];
    }
    // dedupe and unshift
    items = [p].concat(items.filter((x) => x !== p)).slice(0, 10);
    await fs.writeFile({ path, contents: JSON.stringify(items) });
    return;
  } catch (e) {
    // fallback
    try {
      const raw = localStorage.getItem('aurora_recent');
      const items = raw ? JSON.parse(raw) : [];
      const arr = [p]
        .concat((Array.isArray(items) ? items : []).filter((x) => x !== p))
        .slice(0, 10);
      localStorage.setItem('aurora_recent', JSON.stringify(arr));
    } catch (e2) {
      // ignore
    }
  }
}
