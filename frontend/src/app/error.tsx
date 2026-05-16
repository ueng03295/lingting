'use client'

import { useEffect } from 'react'

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  useEffect(() => {
    // Log the error to console so we can see it
    console.error('[App Error Boundary]', error)
  }, [error])

  return (
    <div style={{ padding: '2rem', fontFamily: 'monospace' }}>
      <h2>Something went wrong!</h2>
      <pre style={{ whiteSpace: 'pre-wrap', background: '#f5f5f5', padding: '1rem', borderRadius: '8px', overflow: 'auto', maxHeight: '60vh' }}>
        {error?.message || 'Unknown error'}
        {'\n\n'}
        {error?.stack || ''}
      </pre>
      <button
        onClick={() => reset()}
        style={{ marginTop: '1rem', padding: '0.5rem 1rem', cursor: 'pointer' }}
      >
        Try again
      </button>
    </div>
  )
}