import React, { useRef, useEffect } from 'react'
import Editor, { OnMount } from '@monaco-editor/react'
import Ajv from 'ajv'
import addFormats from 'ajv-formats'

const ajv = new Ajv({ allErrors: true, strict: false })
addFormats(ajv)

export default function MonacoEditor({
  value,
  onChange,
  schema,
}: {
  value: string
  onChange: (v: string) => void
  schema?: any
}) {
  const monRef = useRef<any>(null)

  const handleMount: OnMount = (editor, monaco) => {
    monRef.current = editor
    if (schema) {
      monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
        validate: true,
        schemas: [
          {
            uri: 'inmemory://model/aurora-schema.json',
            fileMatch: ['*'],
            schema,
          },
        ],
      })
    }
  }

  useEffect(() => {
    if (!schema) return
    try {
      const validate = ajv.compile(schema)
      const parsed = JSON.parse(value || '{}')
      const valid = validate(parsed)
      if (!valid) {
        // do nothing; Monaco shows diagnostics via json schema mapping
      }
    } catch (e) {
      // ignore parse errors here; Monaco will show syntax errors
    }
  }, [value, schema])

  return (
    <Editor
      height="100%"
      defaultLanguage="json"
      value={value}
      onChange={(v) => onChange(v || '')}
      onMount={handleMount}
      options={{ minimap: { enabled: false } }}
    />
  )
}
