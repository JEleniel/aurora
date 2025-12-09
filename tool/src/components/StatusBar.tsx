import React, { useEffect, useState } from 'react'
import { getRecentFiles } from '../lib/io'

export default function StatusBar({ message }: { message?: string }) {
  const [recent, setRecent] = useState<string[]>([])

  useEffect(() => {
    let mounted = true
    getRecentFiles().then((r) => {
      if (mounted) setRecent(r)
    })
    return () => {
      mounted = false
    }
  }, [])

  return (
    <div className="mt-3 p-2 text-sm text-gray-300">
      <div>{message || 'Ready'}</div>
      {recent.length > 0 && (
        <div className="mt-1">Recent: {recent.slice(0, 5).join(' • ')}</div>
      )}
    </div>
  )
}
