import React from 'react'
import { saveText, openText, saveSvg } from '../lib/io'
import mermaid from 'mermaid'

export default function Toolbar({
  source,
  onOpen,
}: {
  source: string
  onOpen: (s: string) => void
}) {
  const onSave = async () => {
    const name = 'aurora-card.json'
    await saveText(name, source)
  }

  const onOpenClick = async () => {
    const res = await openText()
    if (res?.contents) onOpen(res.contents)
  }

  const onExportSvg = async () => {
    try {
      const code = source && source.trim().length > 0 ? source : 'flowchart TD\n A[Empty]'
      // ensure mermaid is initialized
      mermaid.initialize({ startOnLoad: false, theme: 'dark' })
      const id = 'm' + Math.random().toString(36).slice(2)
      // mermaid.render returns a promise in newer API; using callback form
      mermaid.render(id, code, (svg) => {
        saveSvg('diagram.svg', svg)
      })
    } catch (e) {
      // fallback: alert
      // eslint-disable-next-line no-alert
      alert('Failed to export SVG: ' + (e as Error).message)
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
