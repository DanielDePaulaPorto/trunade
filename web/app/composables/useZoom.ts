/*
 * One code block at a time fills the screen (Z), for the back of a big room.
 * The block keeps working: editing, running and typing into the terminal.
 */
const zoomedCell = ref<string | null>(null)

export function useZoom() {
  return { zoomedCell, toggleZoom, zoomCell, closeZoom }
}

function zoomCell(id: string) {
  zoomedCell.value = id
  logEvent('zoom', { cell: id })
}

function closeZoom() {
  zoomedCell.value = null
}

function toggleZoom() {
  if (zoomedCell.value) return closeZoom()
  const id = blockToZoom()?.dataset.cellId
  if (id) zoomCell(id)
}

/** The block being edited, or else the visible one nearest the middle of the screen. */
function blockToZoom() {
  const focused = document.activeElement?.closest<HTMLElement>('.code-cell')
  if (focused) return focused
  const middle = window.innerHeight / 2
  const distance = (element: HTMLElement) => {
    const box = element.getBoundingClientRect()
    return Math.abs((box.top + box.bottom) / 2 - middle)
  }
  return [...document.querySelectorAll<HTMLElement>('.code-cell')]
    .filter((element) => element.offsetParent !== null)
    .sort((a, b) => distance(a) - distance(b))[0]
}

/** Bigger text on a wider screen, within limits that still fit a typical program. */
export function zoomScale() {
  return Math.min(2.2, Math.max(1.3, window.innerWidth / 900))
}
