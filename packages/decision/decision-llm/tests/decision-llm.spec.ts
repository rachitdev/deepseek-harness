import { describe, expect, it } from 'vitest'
import { Context } from '@deepseek-ai/cordis'
import DecisionRuntime, { type DecisionRequest } from '@deepseek-ai/dsh-decision'
import * as llmPlugin from '@deepseek-ai/dsh-decision-llm'
import {
  DECISION_SYSTEM_PROMPT,
  LlmDecisionProvider,
  parseDecisionResponse,
  renderDecisionPrompt,
} from '@deepseek-ai/dsh-decision-llm'
import type { GenerateOptions, StreamChunk } from '@deepseek-ai/dsh-llm'

const request: DecisionRequest = {
  state: 'search results page',
  questions: {
    element: { type: 'choice', instructions: 'Which element?', criteria: { a: 'first', b: 'second' } },
    severity: { type: 'score', instructions: 'How bad?', criteria: ['fine', 'bad', 'critical'] },
    done: { type: 'boolean', instructions: 'Is the task complete?' },
  },
}

const validReply = JSON.stringify({
  element: { choice: 'a', probabilities: { a: 0.8, b: 0.2 } },
  severity: { score: 1.5, probabilities: { '0': 0.1, '1': 0.4, '2': 0.5 } },
  done: { probability: 0.3 },
})

/** Emit one text block, the way an adapter streams a complete reply. */
function* textStream(text: string): Generator<StreamChunk> {
  yield { type: 'block-start', index: 0, blockType: 'text' }
  yield { type: 'text-delta', index: 0, text }
  yield { type: 'block-end', index: 0, block: { type: 'text', text } }
  yield { type: 'finish', reason: { kind: 'stop' } }
}

/** Mount the decision seam over a stub `llm` service that replies with `text`. */
async function mountWithReply(text: string): Promise<{ ctx: Context; seen: GenerateOptions[] }> {
  const seen: GenerateOptions[] = []
  const ctx = new Context()
  ctx.provide('llm', {
    stream: (options: GenerateOptions) => {
      seen.push(options)
      return textStream(text)
    },
  } as never)
  await ctx.plugin(DecisionRuntime, {})
  await ctx.plugin(llmPlugin, { provider: 'stub', model: 'cheap-model' })
  return { ctx, seen }
}

describe('decision prompt rendering', () => {
  it('names every option id and level index the model must answer with', () => {
    const prompt = renderDecisionPrompt(request)
    expect(prompt).toContain('search results page')
    expect(prompt).toContain('- a: first')
    expect(prompt).toContain('- 2: critical')
    expect(prompt).toContain('type: boolean')
  })

  it('refuses an unknown question type', () => {
    expect(() => renderDecisionPrompt({
      state: 's',
      questions: { odd: { type: 'ranking', instructions: 'i' } },
    } as unknown as DecisionRequest))
      .toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })
})

describe('decision response parsing', () => {
  it('accepts a well-formed reply', () => {
    expect(parseDecisionResponse(validReply, request)).toMatchObject({
      element: { type: 'choice', choice: 'a' },
      severity: { type: 'score', score: 1.5 },
      done: { type: 'boolean', probability: 0.3 },
    })
  })

  it('accepts a reply wrapped in a Markdown code fence', () => {
    expect(parseDecisionResponse(`\`\`\`json\n${validReply}\n\`\`\``, request))
      .toMatchObject({ element: { choice: 'a' } })
  })

  it.each([
    ['a JSON array', '[]'],
    ['a JSON scalar', '42'],
  ])('rejects %s', (_label, text) => {
    expect(() => parseDecisionResponse(text, request))
      .toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })

  it('rejects a reply missing one of the questions asked', () => {
    expect(() => parseDecisionResponse(JSON.stringify({ element: { choice: 'a', probabilities: { a: 1, b: 0 } } }), request))
      .toThrow(/no answer object for "severity"/)
  })

  it('rejects an option the question never offered', () => {
    const reply = JSON.stringify({
      ...JSON.parse(validReply) as Record<string, unknown>,
      element: { choice: 'invented', probabilities: { a: 0.5, b: 0.5 } },
    })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/not one of its options/)
  })

  it('rejects a score outside the rubric', () => {
    const reply = JSON.stringify({
      ...JSON.parse(validReply) as Record<string, unknown>,
      severity: { score: 9, probabilities: { '0': 0, '1': 0, '2': 1 } },
    })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/outside 0\.\.2/)
  })

  it('rejects a probability outside 0..1', () => {
    const reply = JSON.stringify({
      ...JSON.parse(validReply) as Record<string, unknown>,
      done: { probability: 1.5 },
    })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/outside 0\.\.1/)
  })

  it('rejects a distribution missing an option', () => {
    const reply = JSON.stringify({
      ...JSON.parse(validReply) as Record<string, unknown>,
      element: { choice: 'a', probabilities: { a: 1 } },
    })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/entry "b"/)
  })

  it('rejects a non-object distribution', () => {
    const reply = JSON.stringify({
      ...JSON.parse(validReply) as Record<string, unknown>,
      element: { choice: 'a', probabilities: 'high' },
    })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/no probability map/)
  })

  it('refuses to validate an answer for an unknown question type', () => {
    const odd = {
      state: 's',
      questions: { odd: { type: 'ranking', instructions: 'i' } },
    } as unknown as DecisionRequest
    expect(() => parseDecisionResponse(JSON.stringify({ odd: { probability: 1 } }), odd))
      .toThrow(expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error)
  })

  it('accepts a code fence the model left unterminated', () => {
    expect(parseDecisionResponse(`\`\`\`json\n${validReply}`, request))
      .toMatchObject({ element: { choice: 'a' } })
  })

  it('rejects a non-object answer entry', () => {
    const reply = JSON.stringify({ ...JSON.parse(validReply) as Record<string, unknown>, done: 'yes' })
    expect(() => parseDecisionResponse(reply, request)).toThrow(/no answer object for "done"/)
  })
})

describe('LlmDecisionProvider', () => {
  it('constrains the model with the decision system prompt and returns parsed answers', async () => {
    const { ctx, seen } = await mountWithReply(validReply)
    const result = await ctx.decision.evaluate(request)
    expect(result).toMatchObject({ providerId: 'llm', answers: { element: { choice: 'a' } } })
    expect(seen[0]?.system).toBe(DECISION_SYSTEM_PROMPT)
    expect(seen[0]?.model).toBe('cheap-model')
  })

  it('reports an empty reply rather than inventing answers', async () => {
    const { ctx } = await mountWithReply('   ')
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(/produced no text/)
  })

  it('reports unparseable JSON as a provider error', async () => {
    const { ctx } = await mountWithReply('this is not json')
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('wraps a non-DecisionError parse failure as a provider error', async () => {
    // A deeply nested reply makes JSON.parse throw RangeError rather than the SyntaxError the
    // DecisionError path already covers, exercising the non-DecisionError rethrow.
    const deep = `${'['.repeat(200000)}${']'.repeat(200000)}`
    const { ctx } = await mountWithReply(deep)
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_ERROR' }) as Error,
    )
  })

  it('is unavailable without the llm service', () => {
    const ctx = new Context()
    const provider = new LlmDecisionProvider(ctx, { provider: 'p', model: 'm', maxTokens: 64 })
    expect(provider.available()).toBe(false)
  })

  it('fails loudly when the llm service disappears between checks', async () => {
    const ctx = new Context()
    const provider = new LlmDecisionProvider(ctx, { provider: 'p', model: 'm', maxTokens: 64 })
    await expect(provider.evaluate(request)).rejects.toThrow(/requires the llm service/)
  })

  it.each([
    ['an empty provider route', { provider: '', model: 'm' }],
    ['an empty model id', { provider: 'p', model: '' }],
  ])('is unavailable with %s', async (_label, overrides) => {
    const { ctx } = await mountWithReply(validReply)
    const provider = new LlmDecisionProvider(ctx, { ...overrides, maxTokens: 64 })
    expect(provider.available()).toBe(false)
  })

  it('forwards a cancellation signal to the model call', async () => {
    const { ctx, seen } = await mountWithReply(validReply)
    const controller = new AbortController()
    await ctx.decision.evaluate(request, controller.signal)
    expect(seen[0]?.signal).toBe(controller.signal)
  })
})

describe('decision-llm plugin', () => {
  it('applies the configured token bound over its default', async () => {
    const seen: GenerateOptions[] = []
    const ctx = new Context()
    ctx.provide('llm', {
      stream: (options: GenerateOptions) => {
        seen.push(options)
        return textStream(validReply)
      },
    } as never)
    await ctx.plugin(DecisionRuntime, {})
    await ctx.plugin(llmPlugin, { provider: 'stub', model: 'cheap-model', maxTokens: 77 })
    await ctx.decision.evaluate(request)
    expect(seen[0]?.maxTokens).toBe(77)
  })

  it('is removed from the seam when the fiber disposes', async () => {
    const ctx = new Context()
    ctx.provide('llm', { stream: () => textStream(validReply) } as never)
    await ctx.plugin(DecisionRuntime, {})
    const fiber = await ctx.plugin(llmPlugin, { provider: 'stub', model: 'cheap-model' })
    await expect(ctx.decision.evaluate(request)).resolves.toMatchObject({ providerId: 'llm' })
    await fiber.dispose()
    await expect(ctx.decision.evaluate(request)).rejects.toThrow(
      expect.objectContaining({ code: 'DECISION_PROVIDER_UNAVAILABLE' }) as Error,
    )
  })
})

describe('decision-llm defaults without schema filling', () => {
  // `apply` is reachable directly as well as through the Loader; this covers the in-code token
  // bound that stands in for the schema default.
  it('falls back to the packaged token bound', async () => {
    const seen: GenerateOptions[] = []
    const ctx = new Context()
    ctx.provide('llm', {
      stream: (options: GenerateOptions) => {
        seen.push(options)
        return textStream(validReply)
      },
    } as never)
    await ctx.plugin(DecisionRuntime, {})
    llmPlugin.apply(ctx, { provider: 'stub', model: 'cheap-model' })
    await ctx.decision.evaluate(request)
    expect(seen[0]?.maxTokens).toBe(1024)
  })
})
