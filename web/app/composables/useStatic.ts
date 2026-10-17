/*
 * A lesson exported as one HTML file (see server/src/export.rs). There is no
 * server behind it: ▶ replays the run recorded in the page at export time, and
 * a block that was not run (or was edited since) says so.
 */
import type { Cell, Outcome, RunMessage, ServerMessage, Snapshot, TestCase } from './types'
import type { PreferencesFile } from './usePreferences'

type Recorded<T> = T extends unknown ? Omit<T, 'runId'> : never
export type RecordedMessage = Recorded<RunMessage>

export interface RunRecord {
  source: string
  cases: TestCase[]
  /** Server messages with their time (ms) since ▶. */
  events: { at: number; message: RecordedMessage }[]
}

export interface StaticLesson {
  snapshot: Snapshot
  preferences: PreferencesFile
  runs: Record<string, RunRecord>
  /** Mermaid source → SVG drawn at export time. */
  diagrams: Record<string, string>
  exportedAt: string
}

/** Long pauses (compiling, a program sleeping) are shortened on replay. */
const MAX_REPLAY_GAP_MS = 600

declare global {
  interface Window {
    __TRUNADE_STATIC__?: StaticLesson
  }
}

export const staticLesson: StaticLesson | null = (import.meta.client && window.__TRUNADE_STATIC__) || null

const replays = new Map<string, ReturnType<typeof setTimeout>[]>()

/** Plays the part of the server for the exported page. */
export function staticSend(message: { type: string; [key: string]: unknown }, deliver: (message: ServerMessage) => void) {
  if (message.type === 'run') replay(message as unknown as RunRequest, deliver)
  if (message.type === 'stop') stopReplay(message.runId as string, deliver)
}

interface RunRequest {
  runId: string
  cellId: string
  source: string
  cases: TestCase[]
}

function replay(request: RunRequest, deliver: (message: ServerMessage) => void) {
  const record = staticLesson!.runs[request.cellId]
  const { t } = useI18n()
  if (!record) return finishWith(request.runId, t('staticNotRun'), deliver)
  if (record.source !== request.source || JSON.stringify(record.cases) !== JSON.stringify(request.cases)) {
    return finishWith(request.runId, t('staticEdited'), deliver)
  }
  let time = 0
  let previous = 0
  const timers = record.events.map(({ at, message }) => {
    time += Math.min(MAX_REPLAY_GAP_MS, Math.max(0, at - previous))
    previous = at
    return setTimeout(() => {
      if (message.type === 'finished') replays.delete(request.runId)
      deliver({ ...message, runId: request.runId } as ServerMessage)
    }, time)
  })
  replays.set(request.runId, timers)
}

function stopReplay(runId: string, deliver: (message: ServerMessage) => void) {
  const timers = replays.get(runId)
  if (!timers) return
  timers.forEach(clearTimeout)
  replays.delete(runId)
  deliver({ type: 'finished', runId, outcome: { kind: 'stopped' }, compileMs: null, runMs: null })
}

function finishWith(runId: string, message: string, deliver: (message: ServerMessage) => void) {
  const outcome: Outcome = { kind: 'error', message }
  setTimeout(() => deliver({ type: 'finished', runId, outcome, compileMs: null, runMs: null }))
}

/** The document as the page shows it: edited code and input replace the file's. */
export function withPageEdits(cells: Cell[], edits: Map<string, { source: string; cases: TestCase[] }>): Cell[] {
  return cells.map((cell) => {
    if (cell.kind === 'code') return { ...cell, ...edits.get(cell.id) }
    if (cell.kind === 'steps') {
      return { ...cell, steps: cell.steps.map((step) => ({ ...step, cells: withPageEdits(step.cells, edits) })) }
    }
    return cell
  })
}
