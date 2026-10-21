/**
 * The `read_image` OCR fallback: a route that declares no image input receives
 * recognized text instead of the strict refusal when the deployment configures
 * an OCR command. Every case runs a fake OCR binary, so the suite needs no
 * platform OCR tool. Cleanup, staging mode, and the capture-window failure are
 * asserted through the staged path the child itself records, so those tests
 * cannot pass on the strength of unrelated output.
 */

import { provideWorkingDirectoryFixture } from '@deepseek-ai/dsh-agent-loop-testkit'
import { afterEach, beforeEach, describe, expect, it, onTestFinished, vi } from 'vitest'
import { chmod, mkdtemp, readFile, readdir, realpath, rm, stat, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { basename, dirname, join } from 'node:path'
import { Context } from '@deepseek-ai/cordis'
import { ToolCallId, LlmAdapter, LlmRuntime } from '@deepseek-ai/dsh-llm'
import type { GenerateOptions, LlmModelInfo, LlmResolvedModelInfo, StreamChunk } from '@deepseek-ai/dsh-llm'
import SystemPrompt from '@deepseek-ai/dsh-system-prompt'
import ToolRuntime from '@deepseek-ai/dsh-tools'
import LocalFileSystem from '@deepseek-ai/dsh-fs-local'
import * as FsPolicy from '@deepseek-ai/dsh-fs-observation-policy'
import LocalAttachmentStore from '@deepseek-ai/dsh-attachment-local'
import LocalSubprocessRuntime from '@deepseek-ai/dsh-subprocess-local'
import * as ToolFs from '@deepseek-ai/dsh-tool-fs'
import { formatOcrReadOutput, truncateOcrText } from '../src/read-image.ts'

/** 1x1 red PNG (valid signature, IHDR, IDAT). */
const PNG_1X1 = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC', 'base64')

/** Prefix of every staging directory the fallback creates. */
const STAGING_PREFIX = 'dsh-read-image-ocr-'

/** Fake adapter answering only model capability lookups. */
class CatalogAdapter extends LlmAdapter {
  constructor(private readonly models: LlmModelInfo[]) {
    super()
  }

  override listModels(_provider: string): Promise<readonly LlmModelInfo[]> {
    return Promise.resolve(this.models)
  }

  override resolveModel(provider: string, model: string): Promise<LlmResolvedModelInfo> {
    const resolved = this.models.find(candidate => candidate.id === model)
    return Promise.resolve({
      provider,
      id: model,
      name: resolved?.name ?? model,
      ...resolved?.inputModalities === undefined ? {} : { inputModalities: [...resolved.inputModalities] },
    })
  }

  override stream(_options: GenerateOptions): AsyncIterable<StreamChunk> {
    throw new Error('read_image tests never stream')
  }
}

let dir: string
let home: string
let bin: string

beforeEach(async () => {
  // These prefixes deliberately avoid the staging prefix, which `stagingDirs`
  // scans for when a test must prove no staging directory survived.
  dir = await mkdtemp(join(tmpdir(), 'dsh-ocr-case-'))
  home = await mkdtemp(join(tmpdir(), 'dsh-ocr-home-'))
  bin = await mkdtemp(join(tmpdir(), 'dsh-ocr-bin-'))
})
afterEach(async () => {
  await rm(dir, { recursive: true, force: true })
  await rm(home, { recursive: true, force: true })
  await rm(bin, { recursive: true, force: true })
})

/**
 * Write one executable fake OCR command.
 * @param body - shell body run with the staged image path as `$1`.
 * @param name - file name inside the per-test bin directory.
 * @returns the absolute path to the executable.
 */
async function fakeOcr(body: string, name = 'fake-ocr'): Promise<string> {
  const path = join(bin, name)
  await writeFile(path, `#!/bin/sh\n${body}\n`, { mode: 0o755 })
  await chmod(path, 0o755)
  return path
}

/** One fake OCR command that records the staged path it was handed. */
interface Recording {
  /** Executable to configure as `readImageOcr`. */
  command: string
  /** Whether the command ran at all. */
  ran: () => Promise<boolean>
  /** The staged path the command received. */
  staged: () => Promise<string>
  /** Assert the staged file and its directory are both gone. */
  expectCleanedUp: () => Promise<void>
}

/**
 * Write a fake OCR command that records its staged path before running `body`,
 * so cleanup is provable from what the child saw rather than from tool output.
 * @param body - shell body executed after the path is recorded.
 * @param name - file name inside the per-test bin directory.
 * @returns the command plus assertions over the recorded staged path.
 */
async function recordingOcr(body: string, name = 'recording-ocr'): Promise<Recording> {
  const marker = join(bin, `${name}-marker`)
  const command = await fakeOcr(`printf '%s' "$1" > '${marker}'\n${body}`, name)
  const staged = async () => (await readFile(marker, 'utf8')).trim()
  return {
    command,
    ran: async () => stat(marker).then(() => true, () => false),
    staged,
    expectCleanedUp: async () => {
      const path = await staged()
      expect(dirname(path)).toContain(STAGING_PREFIX)
      await expect(stat(path)).rejects.toThrow()
      await expect(stat(dirname(path))).rejects.toThrow()
    },
  }
}

/**
 * Wait until a predicate holds.
 * @param predicate - condition to poll.
 * @param label - failure message when the deadline passes.
 */
async function waitFor(predicate: () => Promise<boolean>, label: string): Promise<void> {
  for (let attempt = 0; attempt < 500; attempt += 1) {
    if (await predicate()) return
    await new Promise((resolve) => { setTimeout(resolve, 10) })
  }
  throw new Error(label)
}

/** Install a subprocess stub and count spawn attempts. */
function stubSubprocess(ctx: Context, onSpawn: () => unknown): { spawns: () => number } {
  let spawns = 0
  ctx.provide('subprocess', {
    resolveExecutable: async (command: string) => command,
    spawn: () => {
      spawns += 1
      return onSpawn()
    },
  } as never)
  return { spawns: () => spawns }
}

/** Staging directories currently present under the OS temp directory. */
async function stagingDirs(): Promise<string[]> {
  const entries = await readdir(tmpdir())
  return entries.filter(entry => entry.startsWith(STAGING_PREFIX)).sort()
}

interface SetupOptions {
  toolFs?: ToolFs.Config
  subprocess?: boolean
  llm?: boolean
}

async function setup(options: SetupOptions = {}) {
  const ctx = new Context()
  onTestFinished(async () => { await ctx.fiber.dispose() })
  provideWorkingDirectoryFixture(ctx)
  await ctx.plugin(SystemPrompt)
  await ctx.plugin(ToolRuntime, { mode: 'native' })
  await ctx.plugin(LocalFileSystem, { cwd: dir })
  await ctx.plugin(FsPolicy)
  await ctx.plugin(LocalAttachmentStore, { dshHome: home })
  if (options.llm !== false) {
    await ctx.plugin(LlmRuntime)
    ctx.llm.registerAdapter(['visual'], new CatalogAdapter([
      { provider: 'visual', id: 'vision-model', name: 'Vision', inputModalities: ['text', 'image'] },
      { provider: 'visual', id: 'text-model', name: 'Text', inputModalities: ['text'] },
    ]))
  }
  if (options.subprocess !== false) await ctx.plugin(LocalSubprocessRuntime)
  await ctx.plugin(ToolFs, options.toolFs ?? {})
  return ctx
}

/** A fake calling agent pinned to one routed provider/model. */
function agentOn(model: string, provider = 'visual'): object {
  return {
    options: {},
    session: {
      header: { cwd: dir },
      requestHeader: () => ({ config: { provider, model } }),
      deriveMessages: () => [],
      append: () => undefined,
    },
  }
}

let callCounter = 0
function readImage(ctx: Context, filePath: string, model: string, signal?: AbortSignal) {
  return ctx.tools.execute({
    signal: signal ?? new AbortController().signal,
    callId: ToolCallId(`ocr-call-${++callCounter}`),
    name: 'read_image',
    arguments: { file_path: filePath },
    agent: agentOn(model) as never,
  })
}

/** One JSON object member of a tool result, or a failure naming what was missing. */
function jsonObject(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${label} is not an object`)
  }
  return value as Record<string, unknown>
}

/** One string member of a JSON object. */
function jsonString(value: Record<string, unknown>, key: string): string {
  const member = value[key]
  if (typeof member !== 'string') throw new Error(`${key} is not a string`)
  return member
}

/** One number member of a JSON object. */
function jsonNumber(value: Record<string, unknown>, key: string): number {
  const member = value[key]
  if (typeof member !== 'number') throw new Error(`${key} is not a number`)
  return member
}

/** Concatenated text blocks of one result. */
function text(result: { content: { type: string; text?: string }[] }): string {
  return result.content.filter(block => block.type === 'text').map(block => block.text).join('')
}

/** The payload inside the envelope's `<content>` element. */
function envelopeBody(result: { content: { type: string; text?: string }[] }): string {
  const raw = text(result)
  const open = '<content>\n'
  const start = raw.indexOf(open)
  const end = raw.lastIndexOf('\n</content>')
  return start < 0 || end < 0 ? raw : raw.slice(start + open.length, end)
}

describe('truncateOcrText', () => {
  it('keeps text under the cap untouched', () => {
    expect(truncateOcrText('hello', 16)).toEqual({ text: 'hello', truncated: false })
    expect(truncateOcrText('hello', 5)).toEqual({ text: 'hello', truncated: false })
  })

  it('cuts at the cap and reports truncation', () => {
    expect(truncateOcrText('hello world', 5)).toEqual({ text: 'hello', truncated: true })
  })

  it('never splits a multibyte character', () => {
    // Each 'é' is two bytes, so a 3-byte cap can hold only the first one.
    expect(truncateOcrText('ééé', 3)).toEqual({ text: 'é', truncated: true })
    // A 4-byte emoji is dropped whole rather than left half-encoded.
    expect(truncateOcrText('😀😀', 5)).toEqual({ text: '😀', truncated: true })
  })
})

describe('formatOcrReadOutput', () => {
  it('renders text in the envelope without an image', () => {
    expect(formatOcrReadOutput('/tmp/a.png', { text: 'HELLO', engine: 'fake-ocr', bytesRead: 3 }))
      .toBe('<path>/tmp/a.png</path>\n<type>image text</type>\n<content>\nHELLO\n</content>')
  })

  it('says that only the beginning of a capped result is shown', () => {
    expect(formatOcrReadOutput('/tmp/a.png', { text: 'HE', engine: 'fake-ocr', bytesRead: 3, truncated: true }))
      .toContain('[Recognized text was truncated; only the beginning is shown.]')
  })
})

describe('OCR fallback on a route without image input', () => {
  it('returns recognized text and no image block', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "HELLO FROM OCR\\n"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')

    expect(result.isError).toBe(false)
    expect(result.content).toHaveLength(1)
    expect(result.content[0]?.type).toBe('text')
    expect(text(result)).toContain('HELLO FROM OCR')
    expect(text(result)).toContain('<type>image text</type>')
    expect(result.content.some(block => block.type === 'image')).toBe(false)
    expect(result.meta).toMatchObject({ path: await realpath(join(dir, 'red.png')) })
  })

  it('reports the engine, the bytes it read, and the configured language', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "TEXT\\n"', 'engine-name')
    const ctx = await setup({
      toolFs: { readImageOcr: command, readImageOcrArgs: [], readImageOcrLang: 'en-US' },
    })

    const result = await readImage(ctx, 'red.png', 'text-model')
    const ocrValue = jsonObject(jsonObject(result.value, 'result value').ocr, 'ocr')
    expect(jsonString(ocrValue, 'engine')).toBe('engine-name')
    expect(jsonString(ocrValue, 'lang')).toBe('en-US')
    expect(jsonNumber(ocrValue, 'bytesRead')).toBe(PNG_1X1.length)
    expect(jsonString(ocrValue, 'text')).toBe('TEXT')
  })

  it('passes configured arguments before the staged image path', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    // Echo every argument so the assertion sees exactly what the child received.
    const command = await fakeOcr('printf "%s\\n" "$@"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: ['--stdout', '--lang=en'] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    const lines = envelopeBody(result).split('\n')
    expect(lines[0]).toBe('--stdout')
    expect(lines[1]).toBe('--lang=en')
    expect(lines[2]).toMatch(/image\.png$/)
    // The staged copy is the OCR input and lives outside the workspace.
    expect(lines[2]?.startsWith(dir)).toBe(false)
  })

  it('stages the bytes under the temp directory and removes them afterwards', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "%s" "$1"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    const staged = envelopeBody(result).trim()
    expect(staged.startsWith(tmpdir())).toBe(true)
    await expect(stat(staged)).rejects.toThrow()
  })

  it('stages an extension-less image under a media-type extension', async () => {
    // No extension: the format comes from the signature, and the staged copy
    // must still look like a PNG to an engine that dispatches on the name.
    await writeFile(join(dir, 'blob'), PNG_1X1)
    const command = await fakeOcr('printf "%s" "$1"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'blob', 'text-model')
    expect(text(result)).toContain('image.png')
    expect(result.isError).toBe(false)
  })

  it('truncates recognized text at the configured cap and keeps the beginning', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "abcdefghij"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: [], readImageOcrMaxBytes: 4 } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    const ocrValue = jsonObject(jsonObject(result.value, 'result value').ocr, 'ocr')
    expect(jsonString(ocrValue, 'text')).toBe('abcd')
    expect(ocrValue.truncated).toBe(true)
    expect(text(result)).toContain('[Recognized text was truncated; only the beginning is shown.]')
  })

  it('fails instead of returning a fragment when the capture window overflows', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    // Non-uniform output makes the failure prove itself: the leading marker line
    // is absent from a retained tail window, so a fragment could not satisfy the
    // negative assertion by accident.
    const ocr = await recordingOcr('printf "HEAD-MARKER\\n"; head -c 200000 /dev/zero | tr "\\\\0" "x"')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('capture window')
    expect(text(result)).toContain('raise readImageOcrMaxBytes')
    expect(text(result)).not.toContain('HEAD-MARKER')
    expect(result.content.some(block => block.type === 'image')).toBe(false)
    await ocr.expectCleanedUp()
  })
})

describe('an image-capable route is unchanged', () => {
  it('still returns the image block and never runs the OCR command', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('printf "SHOULD NOT RUN"')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'vision-model')

    expect(result.isError).toBe(false)
    expect(result.content).toHaveLength(2)
    expect(result.content[1]?.type).toBe('image')
    expect(text(result)).toContain('image/png image, 1x1 px')
    expect(await ocr.ran()).toBe(false)
  })

  it('keeps the pre-change envelope and value exactly, with no ocr member', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup()

    const result = await readImage(ctx, 'red.png', 'vision-model')
    const resolved = await realpath(join(dir, 'red.png'))
    const value = jsonObject(result.value, 'result value')
    const image = jsonObject(value.image, 'image')

    // `image` became optional in the output schema, so this pins that a vision
    // route still yields it and never gains the OCR member.
    expect(Object.keys(value).sort()).toEqual(['image', 'path'])
    expect('ocr' in value).toBe(false)
    expect(jsonString(value, 'path')).toBe(resolved)
    expect(jsonString(image, 'name')).toBe('red.png')
    expect(jsonString(image, 'mediaType')).toBe('image/png')
    expect(jsonNumber(image, 'width')).toBe(1)
    expect(jsonNumber(image, 'height')).toBe(1)
    expect(result.content).toEqual([
      { type: 'text', text: `<path>${resolved}</path>\n<type>image</type>\n<content>\nimage/png image, 1x1 px, ${jsonNumber(image, 'bytes')} bytes\n</content>` },
      { type: 'image', attachment: {
        attachmentId: jsonString(image, 'attachmentId'),
        mediaType: 'image/png',
        bytes: jsonNumber(image, 'bytes'),
        width: 1,
        height: 1,
        name: 'red.png',
      } },
    ])
    // The envelope template itself is pinned byte for byte.
    expect(text(result)).toBe(`<path>${resolved}</path>\n<type>image</type>\n<content>\nimage/png image, 1x1 px, ${image.bytes} bytes\n</content>`)
  })
})

describe('OCR failures are explicit', () => {
  it('keeps the strict refusal when no OCR command is configured', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup()

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    // Byte-identical to the pre-OCR refusal, down to the tool's error prefix.
    expect(text(result)).toBe('Error: cannot read "red.png" as an image: model "text-model" does not declare image input; switch to an image-capable model to read images')
  })

  it('names a missing executable', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup({ toolFs: { readImageOcr: join(bin, 'absent-ocr'), readImageOcrArgs: [] } })
    const before = await stagingDirs()

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    // Exact message: only SubprocessExecutableNotFoundError produces it, and it
    // comes from resolveExecutable rather than from a failed spawn.
    expect(text(result)).toBe(`Error: cannot read "${join(dir, 'red.png')}" as an image text: the OCR command "${join(bin, 'absent-ocr')}" is not an executable file`)
    // The staging directory made before the failed start is still removed.
    expect(await stagingDirs()).toEqual(before)
  })

  it('names the exit code, quotes the stderr tail, scrubs the staged path, and cleans up', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('printf "cannot open %s\\n" "$1" >&2\nexit 1')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('exited with code 1')
    expect(text(result)).toContain('cannot open <staged>')
    // The staged path the engine echoed never reaches the model.
    expect(text(result)).not.toContain(await ocr.staged())
    await ocr.expectCleanedUp()
  })

  it('scrubs the staged path before slicing the stderr tail', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    // The engine echoes 100 characters, then its input path, then 250 more, so
    // the path straddles the 300-character tail boundary: scrubbing after the
    // slice would leave a partial directory name in the message.
    const ocr = await recordingOcr([
      'i=0; a=""; while [ $i -lt 100 ]; do a="${a}a"; i=$((i+1)); done',
      'i=0; b=""; while [ $i -lt 250 ]; do b="${b}b"; i=$((i+1)); done',
      'printf "%s%s%s" "$a" "$1" "$b" >&2',
      'exit 1',
    ].join('\n'))
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    const stagedDir = dirname(await ocr.staged())
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('<staged>/image.png')
    expect(text(result)).not.toContain(stagedDir)
    // A partial directory name would be the leak the pre-slice scrub prevents.
    expect(text(result)).not.toContain(basename(stagedDir))
    await ocr.expectCleanedUp()
  })

  it('fails when the command produces no text and still cleans up', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('exit 0')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('produced no text')
    await ocr.expectCleanedUp()
  })

  it('fails on the configured deadline and still cleans up', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('sleep 30')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [], readImageOcrTimeoutMs: 500 } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('did not finish within 500ms')
    await ocr.expectCleanedUp()
  })

  it('fails when the caller cancels and still cleans up', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('sleep 30')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })
    const abort = new AbortController()
    // Abort only once the child is running, so this exercises the cancellation
    // path rather than the pre-spawn guard.
    const pending = readImage(ctx, 'red.png', 'text-model', abort.signal).catch((error: unknown) => error)
    await waitFor(() => ocr.ran(), 'the fake OCR command never started')
    abort.abort(new Error('caller stopped the turn'))

    const result = await pending as Awaited<ReturnType<typeof readImage>>
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('was cancelled')
    await ocr.expectCleanedUp()
  })

  it('classifies a provider that rejects the run on cancellation as cancelled', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "TEXT\\n"')
    // No local subprocess runtime: this case provides its own stub below.
    const ctx = await setup({ subprocess: false, toolFs: { readImageOcr: command, readImageOcrArgs: [] } })
    // A provider may reject `done` on cancellation instead of resolving with a
    // signal outcome; the tool must still report the cancellation itself.
    let started = false
    const before = await stagingDirs()
    ctx.provide('subprocess', {
      resolveExecutable: async (command: string) => command,
      spawn: (options: { signal?: AbortSignal }) => {
        started = true
        const done = new Promise((_resolve, reject) => {
          options.signal?.addEventListener('abort', () => { reject(new Error('child terminated by signal')) }, { once: true })
        })
        return {
          done,
          terminate: () => undefined,
          waitForExit: async () => ({ exitCode: null, signal: 'SIGTERM' }),
          collected: {},
        }
      },
    } as never)

    const abort = new AbortController()
    const pending = readImage(ctx, 'red.png', 'text-model', abort.signal).catch((error: unknown) => error)
    await waitFor(async () => started, 'the stubbed provider was never asked to spawn')
    abort.abort(new Error('turn cancelled'))

    const result = await pending as Awaited<ReturnType<typeof readImage>>
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('was cancelled')
    expect(text(result)).not.toContain('could not run')
    expect(await stagingDirs()).toEqual(before)
  })

  it('keeps a run that settled before the deadline successful across teardown', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup({
      subprocess: false,
      toolFs: { readImageOcr: '/bin/sh', readImageOcrArgs: [], readImageOcrTimeoutMs: 100 },
    })
    // The run settles at once, but the provider's teardown outlives the 100 ms
    // deadline. Sampling the deadline after teardown would report a timeout for
    // a recognition that already succeeded.
    stubSubprocess(ctx, () => ({
      done: Promise.resolve({ exitCode: 0, signal: null }),
      terminate: () => undefined,
      waitForExit: async () => {
        await new Promise((resolve) => { setTimeout(resolve, 250) })
        return { exitCode: 0, signal: null }
      },
      collected: {
        stdout: { readFrom: () => ({ text: 'SETTLED TEXT', lossy: false }) },
        stderr: { readFrom: () => ({ text: '', lossy: false }) },
      },
    }))

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(false)
    expect(text(result)).toContain('SETTLED TEXT')
  })

  it('names the missing subprocess service', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const command = await fakeOcr('printf "TEXT\\n"')
    const ctx = await setup({ subprocess: false, toolFs: { readImageOcr: command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('no subprocess service is mounted')
  })

  it('keeps the filesystem vocabulary for a missing file', async () => {
    const command = await fakeOcr('printf "TEXT\\n"')
    const ctx = await setup({ toolFs: { readImageOcr: command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'absent.png', 'text-model')
    expect(result.isError).toBe(true)
    expect(text(result)).toContain('not found')
  })
})

describe('OCR staging', () => {
  it('stages the image owner-only', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    // Report the staged file's permission bits through the tool's text result.
    const ocr = await recordingOcr(
      `'${process.execPath}' -e 'const fs=require("node:fs");process.stdout.write((fs.statSync(process.argv[1]).mode & 511).toString(8))' "$1"`,
      'mode-ocr',
    )
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(false)
    expect(envelopeBody(result).trim()).toBe('600')
    await ocr.expectCleanedUp()
  })

  it('fails before running the command when the turn is already aborted', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('printf "SHOULD NOT RUN"')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })
    const before = await stagingDirs()
    const abort = new AbortController()
    abort.abort(new Error('turn already cancelled'))

    const result = await readImage(ctx, 'red.png', 'text-model', abort.signal)

    expect(result.isError).toBe(true)
    // The dispatcher refuses before the tool runs, so nothing is staged.
    expect(text(result)).toBe('Error: tool call aborted before dispatch')
    // No command ran, and no staging directory survived the attempt.
    expect(await ocr.ran()).toBe(false)
    expect(await stagingDirs()).toEqual(before)
  })

  it('cancels before spawning when the turn aborts while the image is staged', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup({ subprocess: false, toolFs: { readImageOcr: '/bin/sh', readImageOcrArgs: [] } })
    const abort = new AbortController()
    // Abort after the bytes are in hand: staging still runs, and the guard that
    // precedes spawn is what refuses the child.
    const fsService = ctx.fs
    const readBytes = fsService.readBytes.bind(fsService)
    const spy = vi.spyOn(fsService, 'readBytes').mockImplementation(async (target, signal, maxBytes) => {
      const data = await readBytes(target, signal, maxBytes)
      abort.abort(new Error('turn cancelled before start'))
      return data
    })
    onTestFinished(() => { spy.mockRestore() })
    const stub = stubSubprocess(ctx, () => { throw new Error('spawn must not run for an aborted turn') })
    const before = await stagingDirs()

    const result = await readImage(ctx, 'red.png', 'text-model', abort.signal)

    expect(text(result)).toBe(`Error: cannot read "${join(dir, 'red.png')}" as an image text: the OCR command "/bin/sh" was cancelled`)
    expect(stub.spawns()).toBe(0)
    // The staged copy made before the guard is removed before the call returns.
    expect(await stagingDirs()).toEqual(before)
  })

  it('cancels at the start boundary when the turn aborts as the child would start', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ctx = await setup({ subprocess: false, toolFs: { readImageOcr: '/bin/sh', readImageOcrArgs: [] } })
    const abort = new AbortController()
    // Reaching spawn proves the image was staged; aborting there and failing the
    // start must still be reported as the cancellation it is.
    const stub = stubSubprocess(ctx, () => {
      abort.abort(new Error('turn cancelled at start'))
      throw new Error('spawn observed an aborted turn')
    })
    const before = await stagingDirs()

    const result = await readImage(ctx, 'red.png', 'text-model', abort.signal)

    expect(text(result)).toBe(`Error: cannot read "${join(dir, 'red.png')}" as an image text: the OCR command "/bin/sh" was cancelled`)
    expect(stub.spawns()).toBe(1)
    // The staged copy is removed even though no child ever observed it.
    expect(await stagingDirs()).toEqual(before)
  })

  it('cleans the staging directory after a successful run', async () => {
    await writeFile(join(dir, 'red.png'), PNG_1X1)
    const ocr = await recordingOcr('printf "CLEANUP_CHECK"')
    const ctx = await setup({ toolFs: { readImageOcr: ocr.command, readImageOcrArgs: [] } })

    const result = await readImage(ctx, 'red.png', 'text-model')
    expect(result.isError).toBe(false)
    expect(text(result)).toContain('CLEANUP_CHECK')
    await ocr.expectCleanedUp()
  })
})

describe('OCR configuration is validated at load', () => {
  it('rejects a relative command path', async () => {
    await expect(setup({ toolFs: { readImageOcr: 'vision-ocr' } }))
      .rejects.toThrow('readImageOcr must be an absolute path to an OCR executable')
  })

  it('rejects a non-integer timeout or text cap', async () => {
    const command = await fakeOcr('printf "TEXT\\n"')
    await expect(setup({ toolFs: { readImageOcr: command, readImageOcrTimeoutMs: 0 } }))
      .rejects.toThrow('readImageOcrTimeoutMs must be a positive integer')
    await expect(setup({ toolFs: { readImageOcr: command, readImageOcrMaxBytes: 1.5 } }))
      .rejects.toThrow('readImageOcrMaxBytes must be a positive integer')
  })
})

describe('tool-fs Loader composition and unwrapExports', () => {
  it('keeps function-plugin namespace and inject declarations', async () => {
    expect('default' in ToolFs).toBe(false)
    const { default: Loader } = await import('@deepseek-ai/cordis-plugin-loader')
    const loader = Object.create(Loader.prototype) as InstanceType<typeof Loader>
    const unwrapped = loader.unwrapExports(ToolFs) as Record<string, unknown>
    expect(unwrapped).toBe(ToolFs)
    expect(unwrapped.name).toBe('tool-fs')
    expect(unwrapped.inject).toEqual(['tools', 'fs', 'systemPrompt', 'workingDirectory'])
    expect(typeof unwrapped.Config).toBe('function')
    expect(typeof unwrapped.apply).toBe('function')
  })

  it('boots through unwrapExports with OCR config over local subprocess and registers tools', async () => {
    const { default: Loader } = await import('@deepseek-ai/cordis-plugin-loader')
    const ctx = new Context()
    onTestFinished(async () => { await ctx.fiber.dispose() })
    provideWorkingDirectoryFixture(ctx)
    await ctx.plugin(SystemPrompt)
    await ctx.plugin(ToolRuntime, { mode: 'native' })
    await ctx.plugin(LocalFileSystem, { cwd: dir })
    await ctx.plugin(FsPolicy)
    await ctx.plugin(LocalAttachmentStore, { dshHome: home })
    await ctx.plugin(LocalSubprocessRuntime)

    const loader = Object.create(Loader.prototype) as InstanceType<typeof Loader>
    const unwrapped = loader.unwrapExports(ToolFs) as Parameters<Context['plugin']>[0]
    const command = await fakeOcr('printf "LOADER TEST"')
    await ctx.plugin(unwrapped, {
      readImageOcr: command,
      readImageOcrArgs: ['--stdout'],
      readImageOcrTimeoutMs: 15_000,
    })

    const toolNames = ctx.tools.schemas().map(s => s.name)
    expect(toolNames).toEqual(expect.arrayContaining(['read', 'read_image', 'write', 'edit']))
  })
})
