import { describe, expect, it } from 'vitest'
import { parseRecallBlock, renderRecallBlock } from '../src/recall-block.ts'

const BLOCK = [
  '<openviking-context>',
  'Relevant memory from OpenViking.',
  '<memory uri="viking://a" type="resources" score="0.82" detail="uri" />',
  '<memory uri="viking://b" type="entities" score="0.80" detail="abstract">',
  '# B',
  'body of b',
  '</memory>',
  '<memory uri="viking://c" type="events" score="0.78" detail="full">c body</memory>',
  '</openviking-context>',
].join('\n')

describe('parseRecallBlock', () => {
  it('splits header, items, and trailer and keeps element text verbatim', () => {
    const block = parseRecallBlock(BLOCK)
    expect(block?.header).toBe('<openviking-context>\nRelevant memory from OpenViking.\n')
    expect(block?.items.map(item => [item.uri, item.content])).toEqual([
      ['viking://a', ''],
      ['viking://b', '# B\nbody of b'],
      ['viking://c', 'c body'],
    ])
    expect(block?.trailer).toBe('\n</openviking-context>')
  })

  it('re-renders to the identical text when every item is kept', () => {
    const block = parseRecallBlock(BLOCK)!
    expect(renderRecallBlock(block, block.items.map(() => ({ kind: 'keep' })))).toBe(BLOCK)
  })

  it('returns undefined for text without memory elements or with an element lacking a uri', () => {
    expect(parseRecallBlock('<openviking-context>\n- [memory 80%] note\n</openviking-context>')).toBeUndefined()
    expect(parseRecallBlock('<memory type="x">no uri</memory>')).toBeUndefined()
  })
})

describe('renderRecallBlock', () => {
  const block = parseRecallBlock(BLOCK)!

  it('drops, stubs, and keeps items and appends the note', () => {
    const text = renderRecallBlock(block, [
      { kind: 'drop' },
      { kind: 'stub', marker: 'withheld: "x" <y>' },
      { kind: 'keep' },
    ], '(1 omitted)')
    expect(text).toBe([
      '<openviking-context>',
      'Relevant memory from OpenViking.',
      '<memory uri="viking://b" screened="withheld: &quot;x&quot; &lt;y>" />',
      '<memory uri="viking://c" type="events" score="0.78" detail="full">c body</memory>',
      '(1 omitted)',
      '</openviking-context>',
    ].join('\n'))
  })

  it('keeps header and trailer when every item is dropped and there is no note', () => {
    expect(renderRecallBlock(block, [{ kind: 'drop' }, { kind: 'drop' }, { kind: 'drop' }]))
      .toBe('<openviking-context>\nRelevant memory from OpenViking.\n\n</openviking-context>')
  })

  it('keeps the note alone when every item is dropped', () => {
    expect(renderRecallBlock(block, [{ kind: 'drop' }, { kind: 'drop' }, { kind: 'drop' }], '(3 omitted)'))
      .toBe('<openviking-context>\nRelevant memory from OpenViking.\n(3 omitted)\n</openviking-context>')
  })
})
