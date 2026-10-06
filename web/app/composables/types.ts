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
  stdin: string | null
}

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
  | { type: 'running'; runId: string; interactive: boolean }
  | { type: 'output'; runId: string; stream: Stream; text: string }
  | {
      type: 'finished'
      runId: string
      outcome: Outcome
      compileMs: number | null
      runMs: number | null
    }

export type ServerMessage =
  | { type: 'document'; snapshot: Snapshot }
  | { type: 'error'; message: string }
  | RunMessage
