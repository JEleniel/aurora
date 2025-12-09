import React, { useState } from 'react'
import DiagramPreview from './components/DiagramPreview'
import MonacoEditor from './components/MonacoEditor'
import schemaIndex from '../../schemas/card.schema.json'

export default function App(): JSX.Element {
  const [text, setText] = useState<string>('')

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <div className="max-w-7xl mx-auto p-4">
        <header className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-semibold">Aurora Designer</h1>
          <div className="text-sm opacity-80">Dark • Desktop • Tauri</div>
        </header>

        <div className="grid grid-cols-12 gap-4">
          <div className="col-span-5">
            <label className="block text-sm mb-2">Card / Mermaid Source</label>
            <div className="w-full h-[60vh] rounded bg-gray-800 text-gray-100 border border-gray-700">
              <MonacoEditor value={text} onChange={setText} schema={schemaIndex} />
            </div>
          </div>

          <div className="col-span-7">
            <label className="block text-sm mb-2">Diagram Preview</label>
            <div className="p-2 rounded bg-gray-800 border border-gray-700 h-[60vh] overflow-auto">
              <DiagramPreview source={text} />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
