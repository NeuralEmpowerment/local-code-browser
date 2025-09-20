import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { FixedSizeList as List } from 'react-window'

type Project = {
  id: number
  name: string
  path: string
  project_type?: string
  is_git_repo: boolean
  size_bytes?: number
  files_count?: number
  last_edited_at?: number
  loc?: number
}

type Page = {
  items: Project[]
  page: number
  page_size: number
  total_count: number
}

const DEFAULT_PAGE_SIZE = 500

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

function formatDate(timestamp: number | null): string {
  if (!timestamp) return '-'
  
  const date = new Date(timestamp * 1000) // Convert from Unix timestamp
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) return 'Today'
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays}d ago`
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`
  return `${Math.floor(diffDays / 365)}y ago`
}

export default function App() {
  const [q, setQ] = useState('')
  const [sort, setSort] = useState<'recent'|'size'|'name'|'type'|'loc'>('recent')
  const [sortDirection, setSortDirection] = useState<'asc'|'desc'>('desc')
  const [page, setPage] = useState(0)
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE)
  const [rows, setRows] = useState<Project[]>([])
  const [totalCount, setTotalCount] = useState(0)
  const [loading, setLoading] = useState(false)
  const [scanning, setScanning] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [showOpenMenu, setShowOpenMenu] = useState(false)

  useEffect(() => { 
    fetchPage(0) 
  }, [sort, sortDirection])
  
  useEffect(() => { 
    fetchPage(0) 
  }, []) // Load projects on initial mount

  async function fetchPage(p: number) {
    setLoading(true)
    try {
      const res = await invoke<Page>('projects_query', { q, sort, sortDirection, page: p, pageSize })
      setRows(res.items)
      setPage(p)
      setTotalCount(res.total_count)
      setMessage(`${res.items.length} of ${res.total_count} projects loaded`)
    } catch (e: any) {
      console.error('projects_query failed', e)
      setMessage(`Query failed: ${String(e)}`)
    } finally { setLoading(false) }
  }

  function handleHeaderClick(newSort: 'recent'|'size'|'name'|'type'|'loc') {
    if (sort === newSort) {
      // Toggle direction if clicking the same column
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc')
    } else {
      // Set new sort column with default direction
      setSort(newSort)
      setSortDirection(newSort === 'name' || newSort === 'type' ? 'asc' : 'desc')
    }
  }

  async function triggerScan() {
    setScanning(true)
    setMessage('Scanning projects...')
    try {
      const count = await invoke<number>('scan_start', { roots: undefined, dry_run: false })
      setMessage(`Scanned ${count} project(s)`) 
      await fetchPage(0)
    } catch (e: any) {
      console.error('scan_start failed', e)
      setMessage(`Scan failed: ${String(e)}`)
    } finally { setScanning(false) }
  }

  const Row = ({ index, style }: { index: number, style: any }) => {
    const r = rows[index]
    return (
      <div style={style} className="grid grid-cols-[14rem_5rem_7rem_5rem_7rem_1fr] gap-2 px-2 py-1 border-b border-zinc-800">
        <div className="truncate" title={r.name}>{r.name}</div>
        <div className="text-zinc-400">{r.project_type ?? '-'}</div>
        <div className="text-zinc-400 text-right">{formatBytes(r.size_bytes ?? 0)}</div>
        <div className="text-zinc-400 text-right">{r.loc ?? 0}</div>
        <div className="text-zinc-400 text-right">{formatDate(r.last_edited_at)}</div>
        <div className="truncate text-zinc-300" title={r.path}>{r.path}</div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <header className="p-3 flex items-center gap-2 border-b border-zinc-800">
        <input
          value={q}
          onChange={e => setQ(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') fetchPage(0) }}
          placeholder="Search name or path..."
          className="w-80 px-3 py-2 rounded bg-zinc-800 outline-none"
        />
        <select value={sort} onChange={e => setSort(e.target.value as any)} className="px-2 py-2 rounded bg-zinc-800">
          <option value="recent">Recent</option>
          <option value="size">Size</option>
          <option value="name">Name</option>
          <option value="type">Type</option>
          <option value="loc">LOC</option>
        </select>
        <button onClick={() => fetchPage(0)} className="px-3 py-2 rounded bg-zinc-700">Search</button>
        <button 
          onClick={triggerScan} 
          disabled={scanning}
          className={`px-3 py-2 rounded flex items-center gap-2 ${scanning ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'}`}
        >
          {scanning && (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          )}
          {scanning ? 'Scanning...' : 'Scan'}
        </button>
        {loading && <span className="ml-2 text-sm text-zinc-400">Loading…</span>}
        
        {/* Page Size Selector */}
        <div className="flex items-center gap-2 ml-4">
          <span className="text-sm text-zinc-400">Show:</span>
          <select 
            value={pageSize} 
            onChange={e => {
              setPageSize(Number(e.target.value))
              setPage(0)
              fetchPage(0)
            }}
            className="px-2 py-1 rounded bg-zinc-800 text-sm"
          >
            <option value={100}>100</option>
            <option value={250}>250</option>
            <option value={500}>500</option>
            <option value={1000}>1000</option>
          </select>
        </div>

        {/* Pagination Controls */}
        {totalCount > pageSize && (
          <div className="flex items-center gap-2 ml-4">
            <button 
              onClick={() => fetchPage(page - 1)}
              disabled={page === 0}
              className="px-2 py-1 rounded bg-zinc-700 disabled:bg-zinc-800 disabled:text-zinc-500 text-sm"
            >
              ←
            </button>
            <span className="text-sm text-zinc-400">
              Page {page + 1} of {Math.ceil(totalCount / pageSize)}
            </span>
            <button 
              onClick={() => fetchPage(page + 1)}
              disabled={(page + 1) * pageSize >= totalCount}
              className="px-2 py-1 rounded bg-zinc-700 disabled:bg-zinc-800 disabled:text-zinc-500 text-sm"
            >
              →
            </button>
          </div>
        )}
      </header>
      <div className="grid grid-cols-[14rem_5rem_7rem_5rem_7rem_1fr] gap-2 px-2 py-2 text-xs text-zinc-400 border-b border-zinc-800">
        <button 
          onClick={() => handleHeaderClick('name')} 
          className="text-left hover:text-zinc-200 flex items-center gap-1"
        >
          Name {sort === 'name' && (sortDirection === 'asc' ? '↑' : '↓')}
        </button>
        <button 
          onClick={() => handleHeaderClick('type')} 
          className="text-left hover:text-zinc-200 flex items-center gap-1"
        >
          Type {sort === 'type' && (sortDirection === 'asc' ? '↑' : '↓')}
        </button>
        <button 
          onClick={() => handleHeaderClick('size')} 
          className="text-right hover:text-zinc-200 flex items-center justify-end gap-1"
        >
          Size {sort === 'size' && (sortDirection === 'asc' ? '↑' : '↓')}
        </button>
        <button 
          onClick={() => handleHeaderClick('loc')} 
          className="text-right hover:text-zinc-200 flex items-center justify-end gap-1"
        >
          LOC {sort === 'loc' && (sortDirection === 'asc' ? '↑' : '↓')}
        </button>
        <button 
          onClick={() => handleHeaderClick('recent')} 
          className="text-right hover:text-zinc-200 flex items-center justify-end gap-1"
        >
          Last Edit {sort === 'recent' && (sortDirection === 'asc' ? '↑' : '↓')}
        </button>
        <div>Path</div>
      </div>
      <div className="flex-1 min-h-0">
        <div className="h-full overflow-auto">
          {rows.map((r, index) => (
            <div key={r.id} className="grid grid-cols-[14rem_5rem_7rem_5rem_7rem_1fr] gap-2 px-2 py-1 border-b border-zinc-800 hover:bg-zinc-800/50">
              <div className="truncate" title={r.name}>{r.name}</div>
              <div className="text-zinc-400">{r.project_type ?? '-'}</div>
              <div className="text-zinc-400 text-right">{formatBytes(r.size_bytes ?? 0)}</div>
              <div className="text-zinc-400 text-right">{r.loc ?? 0}</div>
              <div className="text-zinc-400 text-right">{formatDate(r.last_edited_at)}</div>
              <button 
                onClick={() => {
                  setSelectedProject(r)
                  setShowOpenMenu(true)
                }}
                className="truncate text-zinc-300 text-left hover:text-white hover:underline" 
                title={r.path}
              >
                {r.path}
              </button>
            </div>
          ))}
        </div>
      </div>
      <footer className="p-2 text-xs text-zinc-500 border-t border-zinc-800 flex items-center gap-3">
        <span>{rows.length} items</span>
        {message && <span className="text-zinc-400">— {message}</span>}
      </footer>

      {/* Open In... Modal */}
      {showOpenMenu && selectedProject && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" onClick={() => setShowOpenMenu(false)}>
          <div className="bg-zinc-800 rounded-lg p-6 min-w-96 max-w-2xl" onClick={e => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4">Open Project</h3>
            <div className="mb-4">
              <p className="text-sm text-zinc-400 mb-2">Project:</p>
              <p className="text-white font-mono text-sm bg-zinc-900 p-2 rounded">{selectedProject.name}</p>
              <p className="text-xs text-zinc-500 mt-1">{selectedProject.path}</p>
            </div>
            <div className="flex flex-col gap-3">
              <button 
                onClick={async () => {
                  try {
                    await invoke('open_in_editor', { 
                      editor: 'windsurf', 
                      path: selectedProject.path 
                    })
                    setMessage(`Opening ${selectedProject.name} in Windsurf...`)
                  } catch (error) {
                    // Fallback: copy command to clipboard
                    navigator.clipboard.writeText(`windsurf "${selectedProject.path}"`)
                    setMessage('Command copied to clipboard: windsurf "' + selectedProject.path + '"')
                  }
                  setShowOpenMenu(false)
                }}
                className="flex items-center gap-3 p-3 rounded bg-blue-600 hover:bg-blue-700 transition-colors"
              >
                <div className="w-8 h-8 bg-blue-500 rounded flex items-center justify-center text-white font-bold">W</div>
                <div className="text-left">
                  <div className="font-medium">Open in Windsurf</div>
                  <div className="text-xs text-blue-200">windsurf "{selectedProject.path}"</div>
                </div>
              </button>
              <button 
                onClick={async () => {
                  try {
                    await invoke('open_in_editor', { 
                      editor: 'cursor', 
                      path: selectedProject.path 
                    })
                    setMessage(`Opening ${selectedProject.name} in Cursor...`)
                  } catch (error) {
                    // Fallback: copy command to clipboard
                    navigator.clipboard.writeText(`cursor "${selectedProject.path}"`)
                    setMessage('Command copied to clipboard: cursor "' + selectedProject.path + '"')
                  }
                  setShowOpenMenu(false)
                }}
                className="flex items-center gap-3 p-3 rounded bg-purple-600 hover:bg-purple-700 transition-colors"
              >
                <div className="w-8 h-8 bg-purple-500 rounded flex items-center justify-center text-white font-bold">C</div>
                <div className="text-left">
                  <div className="font-medium">Open in Cursor</div>
                  <div className="text-xs text-purple-200">cursor "{selectedProject.path}"</div>
                </div>
              </button>
            </div>
            <div className="flex justify-end mt-6">
              <button 
                onClick={() => setShowOpenMenu(false)}
                className="px-4 py-2 text-zinc-400 hover:text-white"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
