// The exported HTML ships diagrams already drawn; it never loads Mermaid.
export default {
  initialize() {},
  render(): never {
    throw new Error('Mermaid is not part of the exported HTML')
  },
}
