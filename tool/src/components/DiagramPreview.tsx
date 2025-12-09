import React, { useEffect, useRef } from 'react'
import mermaid from 'mermaid'

mermaid.initialize({ startOnLoad: false, theme: 'dark' })

export default function DiagramPreview({ source }: { source: string }) {
  const ref = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const el = ref.current
    if (!el) return
    // detect mermaid code block or treat all input as mermaid
    const code = source && source.trim().length > 0 ? source : 'flowchart TD\n  A[Empty]'
    try {
      mermaid.parse(code) // validate
      mermaid.render('mermaid-' + Math.random().toString(36).slice(2), code, (svg) => {
        if (el) el.innerHTML = svg
      })
    } catch (err) {
      el.innerHTML = `<pre class="text-sm text-red-400">Invalid mermaid or empty input</pre>`
    }
  }, [source])

  return <div ref={ref} className="w-full h-full" />
}
