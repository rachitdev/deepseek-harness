/**
 * Parser and renderer for recall blocks. OpenViking blocks are a header, one `<memory uri="…">`
 * element per item, and a trailer. CortexDB blocks are a header, markdown bullets
 * (`- <title|id>: <snippet>`), and a footer. Text between items is retained verbatim so a
 * re-rendered block differs from the original only in the items a caller removed or replaced.
 * @module @deepseek-ai/dsh-decision-consumers/recall-block
 */

/** One `<memory>` element and the text that precedes it. */
export interface RecallBlockItem {
  /** Value of the element's `uri` attribute. */
  readonly uri: string
  /** Element body, trimmed; empty for a self-closing element. */
  readonly content: string
  /** Complete element source text. */
  readonly element: string
  /** Source text between the previous element (or the header) and this one. */
  readonly leading: string
}

/** A recall block split into header, items, and trailer. */
export interface ParsedRecallBlock {
  readonly header: string
  readonly items: readonly RecallBlockItem[]
  readonly trailer: string
}

const MEMORY_ELEMENT = /<memory\s([^>]*?)(?:\/>|>([\s\S]*?)<\/memory>)/g
const URI_ATTRIBUTE = /\buri="([^"]*)"/
/** CortexDB injects one bullet per hit: `- ${title || id}: ${snippet}`. The separator is colon-space, so ids may contain `://`. */
const MARKDOWN_ITEM = /^- (.+?): (.+)$/gm

/**
 * Split one recall block into its items.
 * @param text - the recall message text.
 * @returns the parsed block, or `undefined` when it holds neither a `<memory>` element with a
 *   `uri` attribute nor a CortexDB markdown bullet, so callers pass unrecognized formats through.
 */
export function parseRecallBlock(text: string): ParsedRecallBlock | undefined {
  return text.includes('<memory') ? parseMemoryElements(text) : parseMarkdownItems(text)
}

function parseMemoryElements(text: string): ParsedRecallBlock | undefined {
  return parseItems(text, MEMORY_ELEMENT, (match) => {
    // oxlint-disable-next-line typescript/no-non-null-assertion -- group 1 always participates in a match
    const uri = URI_ATTRIBUTE.exec(match[1]!)?.[1]
    if (uri === undefined) return undefined
    return { uri, content: (match[2] ?? '').trim() }
  })
}

function parseMarkdownItems(text: string): ParsedRecallBlock | undefined {
  return parseItems(text, MARKDOWN_ITEM, (match) => {
    const uri = match[1]
    const content = match[2]
    if (uri === undefined || content === undefined || content.trim().length === 0) return undefined
    return { uri, content: content.trim() }
  })
}

function parseItems(
  text: string,
  pattern: RegExp,
  read: (match: RegExpMatchArray) => { uri: string; content: string } | undefined,
): ParsedRecallBlock | undefined {
  const items: RecallBlockItem[] = []
  let header: string | undefined
  let cursor = 0
  for (const match of text.matchAll(pattern)) {
    const readItem = read(match)
    const index = match.index
    if (readItem === undefined) return undefined
    const leading = text.slice(cursor, index)
    if (header === undefined) header = leading
    items.push({
      uri: readItem.uri,
      content: readItem.content,
      element: match[0],
      leading: items.length === 0 ? '' : leading,
    })
    cursor = index + match[0].length
  }
  if (header === undefined) return undefined
  return { header, items, trailer: text.slice(cursor) }
}

/** What to emit for one item when re-rendering. */
export type RecallItemRendering =
  | { readonly kind: 'keep' }
  | { readonly kind: 'drop' }
  /** Replace the item with a stub that keeps its uri and one withhold marker. */
  | { readonly kind: 'stub'; readonly marker: string }

function escapeAttribute(value: string): string {
  return value.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;')
}

function renderedElement(item: RecallBlockItem, rendering: RecallItemRendering): string | undefined {
  switch (rendering.kind) {
    case 'keep':
      return item.element
    case 'drop':
      return undefined
    case 'stub':
      return item.element.startsWith('- ')
        ? `- ${item.uri}: ${rendering.marker}`
        : `<memory uri="${escapeAttribute(item.uri)}" screened="${escapeAttribute(rendering.marker)}" />`
  }
}

/**
 * Re-render a parsed block with per-item decisions.
 * @param block - the parsed block.
 * @param renderings - one rendering per item, in item order.
 * @param note - optional line appended after the last emitted item.
 * @returns the block text; header and trailer are always kept, so a block whose items were all
 *   dropped still tells the model that recall ran.
 */
export function renderRecallBlock(
  block: ParsedRecallBlock,
  renderings: readonly RecallItemRendering[],
  note?: string,
): string {
  const parts: string[] = []
  block.items.forEach((item, index) => {
    // oxlint-disable-next-line typescript/no-non-null-assertion -- callers pass one rendering per item
    const element = renderedElement(item, renderings[index]!)
    if (element === undefined) return
    parts.push(parts.length === 0 ? element : `${item.leading}${element}`)
  })
  if (note !== undefined) parts.push(`${parts.length === 0 ? '' : '\n'}${note}`)
  return `${block.header}${parts.join('')}${block.trailer}`
}
