import type { RunMessage, ServerMessage, Snapshot } from './types'
import type { PreferencesFile } from './usePreferences'

type RunListener = (message: RunMessage) => void

const RECONNECT_DELAY_MS = 1000

const snapshot = ref<Snapshot | null>(null)
const connected = ref(false)
const lastError = ref<string | null>(null)
const activeRuns = ref(0)
const serverPreferences = ref<PreferencesFile | null>(null)
const runListeners = new Map<string, RunListener>()
let socket: WebSocket | null = null

/** Single WebSocket shared by the whole page, reconnecting while the server restarts. */
export function useConnection() {
  if (!socket && import.meta.client) {
    connect()
  }
  return { snapshot, connected, lastError, activeRuns, serverPreferences, send, onRun, offRun }
}

function connect() {
  socket = new WebSocket(socketUrl())
  socket.onopen = () => {
    connected.value = true
  }
  socket.onclose = () => {
    connected.value = false
    socket = null
    setTimeout(connect, RECONNECT_DELAY_MS)
  }
  socket.onmessage = (event) => dispatch(JSON.parse(event.data) as ServerMessage)
}

function dispatch(message: ServerMessage) {
  switch (message.type) {
    case 'document':
      snapshot.value = message.snapshot
      break
    case 'preferences':
      serverPreferences.value = message.preferences
      break
    case 'error':
      lastError.value = message.message
      break
    default:
      runListeners.get(message.runId)?.(message)
  }
}

function socketUrl(): string {
  const backend = useRuntimeConfig().public.backendUrl as string
  const base = backend ? new URL(backend) : new URL(window.location.href)
  base.protocol = base.protocol === 'https:' ? 'wss:' : 'ws:'
  base.pathname = '/ws'
  return base.toString()
}

function send(message: object) {
  socket?.send(JSON.stringify(message))
}

function onRun(runId: string, listener: RunListener) {
  runListeners.set(runId, listener)
  activeRuns.value = runListeners.size
}

function offRun(runId: string) {
  runListeners.delete(runId)
  activeRuns.value = runListeners.size
}
