/**
 * Parser and renderer for recall blocks built from OpenViking `<memory>` elements: a header, one
 * element per recalled item (`<memory uri="…" …>body</memory>` or self-closing `<memory uri="…" />`),
 * and a trailer. Text between elements is retained verbatim so a re-rendered block differs from the
 * original only in the items a caller removed or replaced.
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

/**
 * Split one recall block into its `<memory>` items.
 * @param text - the recall message text.
 * @returns the parsed block, or `undefined` when it holds no element with a `uri` attribute, so
 *   callers pass unrecognized recall formats through unchanged.
 */
export function parseRecallBlock(text: string): ParsedRecallBlock | undefined {
  const items: RecallBlockItem[] = []
  let header: string | undefined
  let cursor = 0
  for (const match of text.matchAll(MEMORY_ELEMENT)) {
    // oxlint-disable-next-line typescript/no-non-null-assertion -- group 1 always participates in a match
    const uri = URI_ATTRIBUTE.exec(match[1]!)?.[1]
    if (uri === undefined) return undefined
    const leading = text.slice(cursor, match.index)
    if (header === undefined) header = leading
    items.push({
      uri,
      content: (match[2] ?? '').trim(),
      element: match[0],
      leading: items.length === 0 ? '' : leading,
    })
    cursor = match.index + match[0].length
  }
  if (header === undefined) return undefined
  return { header, items, trailer: text.slice(cursor) }
}

/** What to emit for one item when re-rendering. */
export type RecallItemRendering =
  | { readonly kind: 'keep' }
  | { readonly kind: 'drop' }
  /** Replace the element with a self-closing stub carrying the uri and one marker attribute. */
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
      return `<memory uri="${escapeAttribute(item.uri)}" screened="${escapeAttribute(rendering.marker)}" />`
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
