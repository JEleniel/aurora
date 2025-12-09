import type { SaveDialogOptions, OpenDialogOptions } from '@tauri-apps/api/dialog'

// Tauri-aware file helpers with graceful web fallbacks.
export async function saveText(filename: string, text: string) {
  // Try Tauri fs first
  try {
    // dynamic import to avoid throwing in non-tauri environments
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const { dialog } = await import('@tauri-apps/api/dialog')
    const { fs } = await import('@tauri-apps/api')
    const path = await dialog.save({ defaultPath: filename } as SaveDialogOptions)
    if (path) {
      await fs.writeFile({ path, contents: text })
      return path
    }
  } catch (e) {
    // fallback to browser download
  }

  // Browser fallback: create blob and trigger download
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  URL.revokeObjectURL(url)
  return null
}

export async function openText(): Promise<{ path?: string; contents?: string } | null> {
  try {
    const { dialog } = await import('@tauri-apps/api/dialog')
    const { fs } = await import('@tauri-apps/api')
    const selected = await dialog.open({ multiple: false }) as string | null
    if (selected) {
      const contents = await fs.readText(selected)
      return { path: selected, contents }
    }
  } catch (e) {
    // fallback to file input
  }

  return await browserOpenFallback()
}

async function browserOpenFallback(): Promise<{ path?: string; contents?: string } | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.json,.md,.mmd,.txt'
    input.onchange = async () => {
      const file = input.files?.[0]
      if (!file) return resolve(null)
      const contents = await file.text()
      resolve({ path: file.name, contents })
    }
    input.click()
  })
}

export async function saveSvg(filename: string, svg: string) {
  // write as .svg
  return saveText(filename.endsWith('.svg') ? filename : `${filename}.svg`, svg)
}

// Recent files: prefer Tauri app-local storage, fallback to localStorage
export async function getRecentFiles(): Promise<string[]> {
  try {
    const { appLocalDataDir } = await import('@tauri-apps/api/path')
    const { fs } = await import('@tauri-apps/api')
    const dir = await appLocalDataDir()
    const path = `${dir}/aurora_recent.json`
    try {
      const contents = await fs.readText(path)
      const parsed = JSON.parse(contents)
      return Array.isArray(parsed) ? parsed : []
    } catch (e) {
      return []
    }
  } catch (e) {
    // fallback to localStorage
    try {
      const raw = localStorage.getItem('aurora_recent')
      const parsed = raw ? JSON.parse(raw) : []
      return Array.isArray(parsed) ? parsed : []
    } catch (e2) {
      return []
    }
  }
}

export async function addRecentFile(p: string) {
  if (!p) return
  try {
    const { appLocalDataDir } = await import('@tauri-apps/api/path')
    const { fs } = await import('@tauri-apps/api')
    const dir = await appLocalDataDir()
    const path = `${dir}/aurora_recent.json`
    let items: string[] = []
    try {
      const contents = await fs.readText(path)
      items = JSON.parse(contents)
      if (!Array.isArray(items)) items = []
    } catch (e) {
      items = []
    }
    // dedupe and unshift
    items = [p].concat(items.filter((x) => x !== p)).slice(0, 10)
    await fs.writeFile({ path, contents: JSON.stringify(items) })
    return
  } catch (e) {
    // fallback
    try {
      const raw = localStorage.getItem('aurora_recent')
      const items = raw ? JSON.parse(raw) : []
      const arr = [p].concat((Array.isArray(items) ? items : []).filter((x) => x !== p)).slice(0, 10)
      localStorage.setItem('aurora_recent', JSON.stringify(arr))
    } catch (e2) {
      // ignore
    }
  }
}
