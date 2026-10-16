import type { PreferencesFile } from './usePreferences'

export interface MarkdownCell {
  kind: 'markdown'
  id: string
  html: string
}

export interface CodeCell {
  kind: 'code'
  id: string
  fenceLanguage: string
  line: number
  source: string
  flags: string[]
  timeoutSeconds: number | null
  /** ```stdin / ```expected pairs; empty means an interactive run. */
  cases: TestCase[]
  /** Hidden template code compiled around `source`. */
  before: string
  after: string
}

export interface TestCase {
  stdin: string
  expected: string | null
}

export type Verdict =
  | { kind: 'accepted' }
  | { kind: 'wrongAnswer'; line: number; expected: string | null; actual: string | null }

export interface QuizCell {
  kind: 'quiz'
  id: string
  question: string
  options: { html: string; correct: boolean }[]
}

export interface StepsCell {
  kind: 'steps'
  id: string
  steps: { title: string; cells: Cell[] }[]
}

export type Cell = MarkdownCell | CodeCell | QuizCell | StepsCell

export interface Snapshot {
  fileName: string
  cells: Cell[]
  error: string | null
}

export type Stream = 'compiler' | 'stdout' | 'stderr' | 'terminal'

export type Outcome =
  | { kind: 'exited'; code: number }
  | { kind: 'signaled'; signal: string }
  | { kind: 'compileFailed' }
  | { kind: 'timedOut'; phase: 'compile' | 'run' }
  | { kind: 'outputLimitExceeded'; limit: number }
  | { kind: 'stopped' }
  | { kind: 'error'; message: string }

export type RunMessage =
  | { type: 'compiling'; runId: string }
  | { type: 'running'; runId: string; case: number; interactive: boolean }
  | { type: 'output'; runId: string; stream: Stream; text: string }
  | {
      type: 'caseFinished'
      runId: string
      case: number
      outcome: Outcome
      runMs: number
      verdict: Verdict | null
    }
  | {
      type: 'finished'
      runId: string
      outcome: Outcome
      compileMs: number | null
      runMs: number | null
    }

export type ServerMessage =
  | { type: 'document'; snapshot: Snapshot }
  | { type: 'preferences'; preferences: PreferencesFile }
  | { type: 'error'; message: string }
  | RunMessage
