import React from 'react'
import { saveText, openText, saveSvg } from '../lib/io'
import mermaid from 'mermaid'

export default function Toolbar({
  source,
  onOpen,
  onStatus,
}: {
  source: string
  onOpen: (s: string) => void
  onStatus?: (msg: string) => void
}) {
  const onSave = async () => {
    const name = 'aurora-card.json'
    const path = await saveText(name, source)
    if (path && typeof onStatus === 'function') onStatus(`Saved ${path}`)
    else if (typeof onStatus === 'function') onStatus('Saved (download)')
  }

  const onOpenClick = async () => {
    const res = await openText()
    if (res?.contents) {
      onOpen(res.contents)
      if (res.path && typeof onStatus === 'function') onStatus(`Opened ${res.path}`)
      else if (typeof onStatus === 'function') onStatus('Opened file')
    }
  }

  const onExportSvg = async () => {
    try {
      const code = source && source.trim().length > 0 ? source : 'flowchart TD\n A[Empty]'
      mermaid.initialize({ startOnLoad: false, theme: 'dark' })
      const id = 'm' + Math.random().toString(36).slice(2)
      mermaid.render(id, code, (svg) => {
        saveSvg('diagram.svg', svg)
        if (typeof onStatus === 'function') onStatus('Exported SVG')
      })
    } catch (e) {
      if (typeof onStatus === 'function') onStatus('Export failed')
    }
  }

  return (
    <div className="flex gap-2 mb-3">
      <button onClick={onOpenClick} className="px-3 py-1 rounded bg-gray-700">Open</button>
      <button onClick={onSave} className="px-3 py-1 rounded bg-gray-700">Save</button>
      <button onClick={onExportSvg} className="px-3 py-1 rounded bg-gray-700">Export SVG</button>
    </div>
  )
}
