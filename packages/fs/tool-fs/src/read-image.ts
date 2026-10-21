/**
 * The model-facing `read_image` tool commits a PNG/JPEG/WebP/GIF file. A path
 * without a file extension is identified from its file signature, while the
 * attachment service's full decode stays authoritative. The mounted `ctx.fs`
 * backend owns path resolution and read access; names only declare media type.
 *
 * The route gate is deliberately stricter than the host upload preflight. An
 * image-reading tool is useful only when the exact calling route can inspect
 * its result, so unknown capability refuses instead of relying on an adapter
 * failure after filesystem and attachment work. A deployment that configures
 * an OCR command instead returns recognized text to a route that declares no
 * image input, so a request the route cannot read never carries image bytes.
 * @module @deepseek-ai/dsh-tool-fs/src/read-image
 */

import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { basename, extname, join } from 'node:path'
import type { Context } from '@deepseek-ai/cordis'
import { AttachmentError, AttachmentId } from '@deepseek-ai/dsh-attachment'
import type { AttachmentStore, ImageAttachmentRef, ImageMediaType } from '@deepseek-ai/dsh-attachment'
import type { ContentBlock } from '@deepseek-ai/dsh-llm'
import { SubprocessExecutableNotFoundError } from '@deepseek-ai/dsh-subprocess'
import type { SubprocessHandle, SubprocessOutcome } from '@deepseek-ai/dsh-subprocess'
import { defineTool } from '@deepseek-ai/dsh-tools'
import type { GenericCallView, ToolExecution } from '@deepseek-ai/dsh-tools'
import { deadline, timeoutOf } from '@deepseek-ai/dsh-timeout'
import type {} from '@deepseek-ai/dsh-fs'
import { resolveRegularReadTarget } from './read-target.ts'

/** Extensions `read_image` accepts; magic-byte validation at the attachment service stays authoritative. */
const IMAGE_EXTENSIONS: Readonly<Record<string, ImageMediaType>> = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.gif': 'image/gif',
}

const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] as const
const JPEG_SIGNATURE = [0xff, 0xd8, 0xff] as const

function matchesBytes(data: Uint8Array, offset: number, expected: readonly number[]): boolean {
  if (data.byteLength < offset + expected.length) return false
  return expected.every((byte, index) => data[offset + index] === byte)
}

function matchesAscii(data: Uint8Array, offset: number, value: string): boolean {
  if (data.byteLength < offset + value.length) return false
  for (let index = 0; index < value.length; index += 1) {
    if (data[offset + index] !== value.charCodeAt(index)) return false
  }
  return true
}

/**
 * Identify the media type declared by a supported image file signature.
 * @param data - file bytes read through the current filesystem backend.
 * @returns the detected supported media type, or undefined for other bytes.
 */
export function sniffImageMediaType(data: Uint8Array): ImageMediaType | undefined {
  if (matchesBytes(data, 0, PNG_SIGNATURE)) return 'image/png'
  if (matchesBytes(data, 0, JPEG_SIGNATURE)) return 'image/jpeg'
  if (matchesAscii(data, 0, 'GIF87a') || matchesAscii(data, 0, 'GIF89a')) return 'image/gif'
  if (matchesAscii(data, 0, 'RIFF') && matchesAscii(data, 8, 'WEBP')) return 'image/webp'
  return undefined
}

/** Default cap on recognized text returned to the model, in bytes. */
export const OCR_TEXT_MAX_BYTES = 16_384

/** Default deadline for one OCR invocation, in milliseconds. */
export const OCR_TIMEOUT_MS = 30_000

/** Announcement budget: a killed OCR child leaves only the diagnostic tail worth quoting. */
const OCR_STDERR_MAX_BYTES = 4_096

/** Stderr characters quoted in a failure, so one runaway line cannot dominate the report. */
const OCR_STDERR_QUOTE_CHARS = 300

/**
 * Termination grace for the OCR child's process range. A recognized image is
 * one bounded batch job, so the range is expected to disappear immediately.
 */
const OCR_GRACE_MS = 5_000

/** Staged-file extension per media type, so an engine that sniffs by name still recognizes the format. */
const STAGED_EXTENSION: Readonly<Record<ImageMediaType, string>> = {
  'image/png': '.png',
  'image/jpeg': '.jpg',
  'image/webp': '.webp',
  'image/gif': '.gif',
}

/** The recognized-text payload fields, as one optional property of the tool result. */
const OCR_VALUE_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    text: { type: 'string', required: true },
    engine: { type: 'string', required: true },
    lang: { type: 'string' },
    truncated: { type: 'boolean' },
    bytesRead: { type: 'integer', required: true },
  },
} as const

/**
 * The committed-image payload. A nested `required: true` would mark the
 * *property* required, so this payload carries none: the image and the
 * recognized text are two optional result members of which the tool returns
 * exactly one.
 */
const IMAGE_VALUE_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    attachmentId: { type: 'string', required: true },
    mediaType: { type: 'string', enum: ['image/png', 'image/jpeg', 'image/webp', 'image/gif'], required: true },
    bytes: { type: 'integer', required: true },
    width: { type: 'integer', required: true },
    height: { type: 'integer', required: true },
    name: { type: 'string' },
    originalDimensions: {
      type: 'object',
      additionalProperties: false,
      properties: {
        width: { type: 'integer', required: true },
        height: { type: 'integer', required: true },
      },
    },
  },
} as const

/** The committed-image outcome: the image block a vision route receives. */
export interface ImageReadValue {
  /** Canonical absolute path in the filesystem provider's execution world. */
  path: string
  image: {
    attachmentId: string
    mediaType: ImageMediaType
    bytes: number
    width: number
    height: number
    name?: string
    /** Orientation-applied file dimensions before normalization; present only when storage reduced it. */
    originalDimensions?: {
      width: number
      height: number
    }
  }
}

/** Recognized text from one OCR invocation, as the model-facing output declares it. */
export interface ImageReadOcr {
  /** Recognized text, capped at the deployment's configured byte limit. */
  text: string
  /** Executable name that produced the text (the configured command's basename). */
  engine: string
  /** Language the deployment declared for recognition; absent when none is configured. */
  lang?: string
  /** True when recognition output was cut at the configured cap. */
  truncated?: boolean
  /** Bytes of image data read from the filesystem and handed to the engine. */
  bytesRead: number
}

/** The recognized-text outcome: what a route without image input receives instead of the image. */
export interface ImageReadOcrValue {
  /** Canonical absolute path in the filesystem provider's execution world. */
  path: string
  ocr: ImageReadOcr
}

/** Either outcome `read_image` can return; exactly one member carries the payload. */
export type ReadImageValue = ImageReadValue | ImageReadOcrValue

/**
 * OCR fallback settings for routes that declare no image input. Configured by
 * the deployment; omitted, such a route keeps the strict image refusal.
 */
export interface ReadImageOcrOptions {
  /** Absolute path to the OCR executable. */
  command: string
  /** Arguments placed before the staged image path. */
  args: readonly string[]
  /** Language recorded with the result; any recognition flag belongs in `args`. */
  lang?: string
  /** Deadline for one invocation, in milliseconds. */
  timeoutMs: number
  /** Cap on recognized text returned to the model, in bytes. */
  maxTextBytes: number
}

/**
 * Map a model-supplied path to its declared image media type by extension.
 * @param filePath - the raw `file_path` argument (not yet resolved).
 * @returns the declared media type, or undefined when the path does not claim an image.
 */
export function imageMediaTypeForPath(filePath: string): ImageMediaType | undefined {
  return IMAGE_EXTENSIONS[extname(filePath).toLowerCase()]
}

/** What the exact calling route can accept from a `read_image` result. */
type RouteImageSupport =
  | { readonly kind: 'image' }
  | { readonly kind: 'text'; readonly model: string }

/**
 * Resolve whether the exact calling route declares `image` input, from the
 * session's latest routed provider/model (request header config, then agent
 * options). An unresolvable route refuses: without a route the deployment
 * cannot tell a text-only session from a misconfiguration, and guessing would
 * attach image bytes no adapter promised to accept.
 * @param ctx - the plugin context used to resolve the optional `llm` service.
 * @param exec - the tool-execution context supplying the calling agent.
 * @param requestedPath - the raw, not-yet-resolved path rendered in refusal messages.
 * @returns `image` when the resolved route declares image input, else `text` with its model name.
 */
async function routeImageSupport(ctx: Context, exec: ToolExecution, requestedPath: string): Promise<RouteImageSupport> {
  const routed = exec.agent?.session.requestHeader()?.config
  const provider = routed?.provider ?? exec.agent?.options.provider
  const model = routed?.model ?? exec.agent?.options.model
  const llm = ctx.get('llm')
  if (provider === undefined || model === undefined || llm === undefined) {
    throw new Error(`cannot read "${requestedPath}" as an image: the current model route could not be resolved`)
  }
  const active = await llm.resolveModelInfo(provider, model, exec.signal)
  if (active.inputModalities === undefined || !active.inputModalities.includes('image')) {
    return { kind: 'text', model }
  }
  return { kind: 'image' }
}

/** The one refusal used when a route cannot inspect an image and no OCR command is configured. */
function imageInputRefusal(requestedPath: string, model: string): Error {
  return new Error(`cannot read "${requestedPath}" as an image: model "${model}" does not declare image input; switch to an image-capable model to read images`)
}

/**
 * Enforce the strict image-capability gate for the calling route. Resolves the
 * session's latest routed provider/model (request header config, then agent
 * options) and requires the exact resolved route to declare `image` input explicitly.
 * @param ctx - the plugin context used to resolve the optional `llm` service.
 * @param exec - the tool-execution context supplying the calling agent.
 * @param requestedPath - the raw, not-yet-resolved path rendered in refusal messages.
 */
export async function assertImageCapableRoute(ctx: Context, exec: ToolExecution, requestedPath: string): Promise<void> {
  const support = await routeImageSupport(ctx, exec, requestedPath)
  if (support.kind === 'text') throw imageInputRefusal(requestedPath, support.model)
}

/** Refuse a media type outside the deployment's accepted set, naming the offending path. */
function assertDeploymentAccepts(attachments: AttachmentStore, mediaType: ImageMediaType, displayPath: string): void {
  if (!attachments.imageLimits.mediaTypes.includes(mediaType)) {
    throw new Error(`cannot read "${displayPath}": ${mediaType} images are not accepted by this deployment`)
  }
}

/**
 * Re-brand a structured image outcome into the durable attachment reference an
 * `ImageBlock` carries.
 * @param image - the image metadata from the output schema.
 * @returns the branded attachment reference.
 */
export function imageRefFromValue(image: ImageReadValue['image']): ImageAttachmentRef {
  return {
    attachmentId: AttachmentId(image.attachmentId),
    mediaType: image.mediaType,
    bytes: image.bytes,
    width: image.width,
    height: image.height,
    ...image.name === undefined ? {} : { name: image.name },
    ...image.originalDimensions === undefined ? {} : {
      originalDimensions: { ...image.originalDimensions },
    },
  }
}

/**
 * Format an image read as the model-facing envelope beside its image block.
 * A downscaled read names the on-disk dimensions and the multiplier that maps
 * coordinates measured on the attached image back onto the original file.
 * @param displayPath - the backend-resolved path rendered in the envelope's `<path>` element.
 * @param image - the image metadata to summarize.
 * @returns the model-facing envelope; the image itself rides the adjacent image block.
 */
export function formatImageReadOutput(displayPath: string, image: ImageReadValue['image']): string {
  let scaled = ''
  if (image.originalDimensions !== undefined) {
    // Integer rounding can give the two axes slightly different ratios, so the
    // advice names one multiplier only when both round to the same value.
    const x = (image.originalDimensions.width / image.width).toFixed(2)
    const y = (image.originalDimensions.height / image.height).toFixed(2)
    const advice = x === y
      ? `multiply coordinates by ${x}`
      : `multiply x coordinates by ${x} and y coordinates by ${y}`
    scaled = ` (downscaled from ${image.originalDimensions.width}x${image.originalDimensions.height} px; ${advice} to locate features in the original file)`
  }
  return `<path>${displayPath}</path>
<type>image</type>
<content>
${image.mediaType} image, ${image.width}x${image.height} px, ${image.bytes} bytes${scaled}
</content>`
}

/**
 * Format a recognized-text read as the model-facing envelope. No image block
 * accompanies it, so this text is the whole payload; a capped result says so.
 * @param displayPath - the backend-resolved path rendered in the envelope's `<path>` element.
 * @param ocr - the recognized text and its provenance.
 * @returns the model-facing envelope carrying the recognized text.
 */
export function formatOcrReadOutput(displayPath: string, ocr: ImageReadOcr): string {
  // Only the head can be cut here: a result that overflowed the capture window
  // fails instead of returning a fragment.
  const truncated = ocr.truncated === true ? '\n\n[Recognized text was truncated; only the beginning is shown.]' : ''
  return `<path>${displayPath}</path>
<type>image text</type>
<content>
${ocr.text}${truncated}
</content>`
}

/**
 * Project one read outcome into content blocks: pixels for a route that
 * declares image input, recognized text for a route that does not.
 * @param value - the read outcome, carrying exactly one payload member.
 * @returns the content blocks used by native and nested dispatches.
 */
function imageReadContent(value: { path: string; image?: ImageReadValue['image']; ocr?: ImageReadOcr }): ContentBlock[] {
  if (value.ocr !== undefined) return [{ type: 'text', text: formatOcrReadOutput(value.path, value.ocr) }]
  if (value.image === undefined) throw new Error('read_image produced neither an image nor recognized text')
  return [
    { type: 'text', text: formatImageReadOutput(value.path, value.image) },
    { type: 'image', attachment: imageRefFromValue(value.image) },
  ]
}

/** Reduce a caught value to a message for one failure report. */
function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/**
 * Quote a failed child's diagnostic tail, bounded so one runaway line cannot
 * dominate the report. The staged directory never survives into the message:
 * OCR engines routinely echo the input path they were given.
 * @param text - the child's collected stderr.
 * @param stagedDir - the staging directory to replace with `<staged>`.
 * @returns the quoted tail, or an empty string when stderr carried nothing.
 */
function stderrQuote(text: string, stagedDir: string): string {
  // Scrub before slicing: a staged path straddling the slice boundary would
  // otherwise survive as a partial path.
  const scrubbed = scrubStaged(text.trim(), stagedDir)
  if (scrubbed.trim() === '') return ''
  return `: ${scrubbed.length > OCR_STDERR_QUOTE_CHARS ? scrubbed.slice(-OCR_STDERR_QUOTE_CHARS).trim() : scrubbed.trim()}`
}

/** Replace every staged-directory prefix in one message so a temp path is never disclosed. */
function scrubStaged(text: string, stagedDir: string): string {
  return text.split(stagedDir).join('<staged>')
}

/** The failure for a command that outlived the deployment's deadline. */
function ocrTimeout(displayPath: string, commandLine: string, timeoutMs: number): Error {
  return new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" did not finish within ${timeoutMs}ms`)
}

/** The failure for a command abandoned with the calling turn. */
function ocrCancelled(displayPath: string, commandLine: string): Error {
  return new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" was cancelled`)
}

/**
 * Cut recognized text at a byte cap without splitting a UTF-8 sequence.
 * @param text - recognized text to bound.
 * @param maxBytes - cap in bytes.
 * @returns the bounded text and whether anything was removed.
 */
export function truncateOcrText(text: string, maxBytes: number): { text: string; truncated: boolean } {
  if (Buffer.byteLength(text, 'utf8') <= maxBytes) return { text, truncated: false }
  const bytes = Buffer.from(text, 'utf8')
  let end = maxBytes
  // A continuation byte here means the cap landed mid-sequence: back off to the
  // start of that code point so the retained slice is valid UTF-8.
  while (end > 0 && ((bytes[end] ?? 0) & 0xc0) === 0x80) end -= 1
  return { text: bytes.subarray(0, end).toString('utf8'), truncated: true }
}

/**
 * Name a command that could not start, distinguishing a missing executable from
 * other provider faults. The provider's own message is scrubbed of the staging
 * directory before it reaches the model.
 * @param error - the provider failure.
 * @param displayPath - the user-facing image path.
 * @param commandLine - the configured command line, quoted in the message.
 * @param stagedDir - the staging directory to replace with `<staged>`.
 * @returns the explicit failure to throw.
 */
function ocrCommandFailure(error: unknown, displayPath: string, commandLine: string, stagedDir: string): Error {
  if (error instanceof SubprocessExecutableNotFoundError) {
    return new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" is not an executable file`)
  }
  return new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" could not run (${scrubStaged(messageOf(error), stagedDir)})`)
}

/**
 * Run the configured OCR command over one staged copy of the image and return
 * its recognized text. The image bytes always come through `ctx.fs`; the
 * staged copy under the OS temp directory is the only new write, and it is
 * owner-only and removed before this returns.
 * @param ctx - the plugin context providing the optional subprocess service.
 * @param exec - the current tool execution, supplying cancellation.
 * @param input - the resolved display path, the route's model, the image bytes, its media type, and the OCR settings.
 * @returns recognized text bounded by the configured cap.
 */
async function ocrImage(
  ctx: Context,
  exec: ToolExecution,
  input: {
    displayPath: string
    model: string
    data: Uint8Array
    mediaType: ImageMediaType
    ocr: ReadImageOcrOptions
  },
): Promise<ImageReadOcr> {
  const { displayPath, model, data, mediaType, ocr } = input
  const subprocess = ctx.get('subprocess')
  if (subprocess === undefined) {
    throw new Error(`cannot read "${displayPath}" as an image text: model "${model}" declares no image input and the configured OCR command "${ocr.command}" cannot run because no subprocess service is mounted`)
  }
  const commandLine = [ocr.command, ...ocr.args].join(' ')
  let stagedDir: string
  try {
    stagedDir = await mkdtemp(join(tmpdir(), 'dsh-read-image-ocr-'))
  } catch (error) {
    throw new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" needs a private temporary directory, and one could not be created`, { cause: error })
  }
  const stagedPath = join(stagedDir, `image${STAGED_EXTENSION[mediaType]}`)
  try {
    try {
      await writeFile(stagedPath, data, { mode: 0o600 })
    } catch (error) {
      throw new Error(`cannot read "${displayPath}" as an image text: the image could not be staged for the OCR command "${commandLine}"`, { cause: error })
    }
    // Recognize into a window several times the text cap so a normal result is
    // captured whole; a result that overflows the window fails below instead of
    // returning a fragment, because a bounded collector keeps its tail.
    const stdoutCap = Math.max(ocr.maxTextBytes * 4, 8_192)
    using call = deadline(exec.signal, ocr.timeoutMs, 'READ_IMAGE_OCR_TIMEOUT')
    // A turn aborted while the image was staged (or already aborted on entry)
    // must not start a child; the staging cleanup below still runs.
    if (exec.signal.aborted) throw ocrCancelled(displayPath, commandLine)
    // Resolve before spawning: the provider reports a missing executable only
    // here, while `spawn` would surface a raw ENOENT as a start failure.
    let executable: string
    try {
      executable = await subprocess.resolveExecutable(ocr.command, undefined, call.signal)
    } catch (error) {
      if (call.signal.aborted) throw ocrCancelled(displayPath, commandLine)
      throw ocrCommandFailure(error, displayPath, commandLine, stagedDir)
    }
    let handle: SubprocessHandle
    try {
      handle = subprocess.spawn({
        argv: [executable, ...ocr.args, stagedPath],
        cwd: stagedDir,
        graceMs: OCR_GRACE_MS,
        signal: call.signal,
        stdio: { stdin: 'ignore', stdout: { maxBytes: stdoutCap }, stderr: { maxBytes: OCR_STDERR_MAX_BYTES } },
      })
    } catch (error) {
      // The turn can abort between staging and start; report the cancellation
      // rather than a start failure.
      if (call.signal.aborted) throw ocrCancelled(displayPath, commandLine)
      throw ocrCommandFailure(error, displayPath, commandLine, stagedDir)
    }
    let outcome: SubprocessOutcome
    // The deadline stays armed through teardown, so the timeout and turn state
    // are sampled the moment the run settles: a child that exited before the
    // deadline must not be reported as a timeout because `waitForExit` crossed
    // it.
    let timedOut: boolean
    let cancelled: boolean
    try {
      outcome = await handle.done
      timedOut = timeoutOf(call.signal, 'READ_IMAGE_OCR_TIMEOUT') !== undefined
      cancelled = !timedOut && call.signal.aborted
    } catch (error) {
      // A provider may reject `done` on cancellation instead of resolving with a
      // signal outcome, so the deadline and turn abort are classified first and
      // only a genuine start failure falls through.
      if (timeoutOf(call.signal, 'READ_IMAGE_OCR_TIMEOUT') !== undefined) throw ocrTimeout(displayPath, commandLine, ocr.timeoutMs)
      // The deadline signal also aborts on upstream cancellation, so it carries
      // the turn's abort after the provider rejected the run.
      if (call.signal.aborted) throw ocrCancelled(displayPath, commandLine)
      throw ocrCommandFailure(error, displayPath, commandLine, stagedDir)
    } finally {
      // Empty the managed range before the staging directory disappears.
      handle.terminate()
      try { await handle.waitForExit() } catch {
        // The provider already released this range (or lost observation of it);
        // the staging cleanup below is what must still run.
      }
    }
    if (timedOut) throw ocrTimeout(displayPath, commandLine, ocr.timeoutMs)
    if (cancelled) throw ocrCancelled(displayPath, commandLine)
    if (outcome.exitCode !== 0) {
      const how = outcome.exitCode === null ? `was killed by ${outcome.signal ?? 'a signal'}` : `exited with code ${outcome.exitCode}`
      throw new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" ${how}${stderrQuote(handle.collected.stderr?.readFrom(0).text ?? '', stagedDir)}`)
    }
    const collected = handle.collected.stdout?.readFrom(0)
    // A bounded collector keeps the tail once the window overflows, so the
    // surviving bytes are a mid-document fragment. Fail rather than hand the
    // model text it cannot place, and name the knob that widens the window.
    if (collected?.lossy === true) {
      throw new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" produced more than the ${stdoutCap}-byte capture window, so the start of its text was already lost; raise readImageOcrMaxBytes to capture the whole result`)
    }
    const recognized = (collected?.text ?? '').trim()
    if (recognized === '') {
      throw new Error(`cannot read "${displayPath}" as an image text: the OCR command "${commandLine}" produced no text; the image may contain none`)
    }
    const bounded = truncateOcrText(recognized, ocr.maxTextBytes)
    return {
      text: bounded.text,
      engine: basename(ocr.command),
      ...ocr.lang === undefined ? {} : { lang: ocr.lang },
      ...bounded.truncated ? { truncated: true } : {},
      bytesRead: data.byteLength,
    }
  } finally {
    // Teardown of the temporary file is guaranteed; failure to remove must not
    // mask a successful OCR outcome, but does log a warning if an error occurs.
    await rm(stagedDir, { recursive: true, force: true }).catch((error: unknown) => {
      // In production or tests where rm fails, we ensure the error does not replace
      // the primary result while remaining visible to diagnostics.
      ctx.logger('tool-fs').warn('failed to clean up OCR staging directory "%s": %s', stagedDir, messageOf(error))
    })
  }
}

/**
 * Register the `read_image` tool into the given context. The composing plugin
 * owns the attachments gate: `src/index.ts` calls this inside
 * `ctx.inject(['attachments'], …)` so the tool exists only while a durable
 * store is mounted. Execution still re-checks `ctx.get('attachments')` for
 * direct callers and gates on the calling route's declared image input.
 * @param ctx - the registration scope; execution uses its `fs` service plus
 *   the optional `attachments`/`llm`/`subprocess` services.
 * @param ocr - the deployment's OCR command for a route that declares no image
 *   input; omitted, such a route keeps the strict image refusal.
 */
export function applyReadImageTool(ctx: Context, ocr?: ReadImageOcrOptions): void {
  const ocrOptions = ocr
  ctx.tools.register(defineTool({
    name: 'read_image',
    description: 'Read a PNG/JPEG/WebP/GIF file and return the image itself. '
      + 'Large images are downscaled automatically; do not install image libraries or create thumbnails to inspect an image.',
    parameters: {
      file_path: { type: 'string', required: true, description: 'Path to the image file, resolved by the filesystem backend.' },
    },
    output: {
      schema: {
        type: 'object',
        additionalProperties: false,
        properties: {
          path: { type: 'string', required: true, description: 'Canonical absolute path in the filesystem execution world.' },
          image: IMAGE_VALUE_SCHEMA,
          ocr: OCR_VALUE_SCHEMA,
        },
      },
      render: (_args, value) => imageReadContent(value),
      // Persist the resolved path only. The attachment reference is NOT copied
      // here: the settled `content` already carries the image block with the
      // complete reference, so a second copy would keep two records of one fact —
      // and a `tools/post-execute` hook that legitimately replaces the content
      // would leave the stale copy behind, showing an image the result no longer
      // returns. The path needs its own structured record because the content
      // carries it only as model-facing envelope text, which the client does not
      // parse.
      presentationMeta: (_args, value) => ({ path: value.path }),
    },
    // Content-addressed attachment writes are idempotent, so concurrent reads
    // of the same file cannot conflict.
    isConcurrencySafe: () => true,
    async execute(args, exec) {
      if (args.file_path.trim().length === 0) throw new Error('file_path must be a non-empty string')

      // Every pre-read gate runs before any filesystem I/O so a refusal never
      // leaks partial reads or attachment writes. An extension-less path
      // declares no format, so only its format and deployment media-type
      // checks wait for the bytes.
      const extension = extname(args.file_path).toLowerCase()
      const declared = imageMediaTypeForPath(args.file_path)
      if (declared === undefined && extension !== '') {
        throw new Error(`cannot read "${args.file_path}": the ${extension} extension does not declare a supported image format; read_image accepts PNG/JPEG/WebP/GIF files, including extension-less files in those formats`)
      }
      const attachments = ctx.get('attachments')
      if (attachments === undefined) {
        throw new Error(`cannot read "${args.file_path}" as an image: no attachment service is mounted`)
      }
      if (declared !== undefined) assertDeploymentAccepts(attachments, declared, args.file_path)
      const route = await routeImageSupport(ctx, exec, args.file_path)
      const textRoute = route.kind === 'text' && ocrOptions !== undefined ? { model: route.model, ocr: ocrOptions } : undefined
      // A text route needs a configured OCR command; without one the strict
      // refusal stands before any filesystem I/O, exactly as it always has.
      if (route.kind === 'text' && textRoute === undefined) throw imageInputRefusal(args.file_path, route.model)

      const { target, info } = await resolveRegularReadTarget(ctx, exec, args.file_path)

      // The tool result is one message carrying one image, so the per-message
      // aggregate bound applies beside the per-image bound.
      const byteCap = Math.min(attachments.imageLimits.maxImageBytes, attachments.imageLimits.maxMessageImageBytes)
      const data = await ctx.fs.readBytes(target, exec.signal, byteCap)
      const mediaType = declared ?? sniffImageMediaType(data)
      if (mediaType === undefined) {
        throw new Error(`cannot read "${target.displayPath}": the file content is not a supported image format; read_image accepts PNG/JPEG/WebP/GIF`)
      }
      if (declared === undefined) assertDeploymentAccepts(attachments, mediaType, target.displayPath)
      // A text route never commits the bytes: its request cannot carry an image
      // block, so the durable store stays free of objects it would never send.
      if (textRoute !== undefined) {
        const recognized = await ocrImage(ctx, exec, {
          displayPath: target.displayPath,
          model: textRoute.model,
          data,
          mediaType,
          ocr: textRoute.ocr,
        })
        ctx.emit('fs/observed', target, { kind: 'present', version: info.version }, exec)
        return { path: ctx.fs.processPath(target), ocr: recognized }
      }
      // Persist before returning: the image block must reference a durably
      // committed object by the time the tool/result event is appended.
      let ref: ImageAttachmentRef
      try {
        ref = await attachments.saveImage({ data, mediaType, name: basename(target.displayPath) })
      } catch (error: unknown) {
        if (!(error instanceof AttachmentError)) throw error
        // Dimension refusals stay recoverable tool errors: an oversized image
        // must never enter durable history, where it would ride every later
        // model request past provider-side dimension rejections.
        if (error.code === 'IMAGE_DIMENSION_TOO_LARGE') {
          throw new Error(
            `cannot read "${target.displayPath}": at least one image side exceeds the ${attachments.imageLimits.maxImageDimension}px limit; downscale the image and read the smaller copy`,
            { cause: error },
          )
        }
        if (error.code === 'IMAGE_TOO_MANY_PIXELS') {
          throw new Error(
            `cannot read "${target.displayPath}": the image exceeds the ${attachments.imageLimits.maxImagePixels}-pixel decoded-size limit; downscale the image and read the smaller copy`,
            { cause: error },
          )
        }
        if (error.code === 'IMAGE_TOO_LARGE') {
          throw new Error(
            `cannot read "${target.displayPath}": the image cannot be stored within the deployment's byte limits; downscale the image and read the smaller copy`,
            { cause: error },
          )
        }
        if (error.code === 'ATTACHMENT_WRITE_FAILED' && /16-bit PNG/iu.test(error.message)) {
          throw new Error(
            `cannot read "${target.displayPath}": the 16-bit PNG could not be converted to the normalized 8-bit sRGB form; convert it to an 8-bit PNG/JPEG/WebP and retry`,
            { cause: error },
          )
        }
        if (error.code === 'INVALID_IMAGE' && declared === undefined) {
          throw new Error(
            `cannot read "${target.displayPath}": the bytes do not decode as a supported PNG/JPEG/WebP/GIF image; the file may be truncated or corrupt`,
            { cause: error },
          )
        }
        if (error.code !== 'IMAGE_TYPE_MISMATCH') throw error
        if (declared === undefined) {
          throw new Error(
            `cannot read "${target.displayPath}": the file signature claims ${mediaType}, but the bytes decode as a different image format; the file may be corrupt`,
            { cause: error },
          )
        }
        throw new Error(
          `cannot read "${target.displayPath}": the ${extension} extension declares ${mediaType}, but the bytes use a different image format; rename the file to match its actual format if it is PNG/JPEG/WebP/GIF, or convert it to one of those formats`,
          { cause: error },
        )
      }
      ctx.emit('fs/observed', target, { kind: 'present', version: info.version }, exec)
      const value: ImageReadValue = {
        path: ctx.fs.processPath(target),
        image: {
          attachmentId: ref.attachmentId,
          mediaType: ref.mediaType,
          bytes: ref.bytes,
          width: ref.width,
          height: ref.height,
          ...ref.name === undefined ? {} : { name: ref.name },
          ...ref.originalDimensions === undefined ? {} : {
            originalDimensions: { ...ref.originalDimensions },
          },
        },
      }
      return value
    },
    // Pure display: a generic card in the read family with a follow-along
    // location on the image file.
    presentCall(args): GenericCallView {
      return {
        card: 'generic',
        title: `Read image ${args.file_path}`,
        kind: 'read',
        locations: [{ path: args.file_path }],
      }
    },
  }))
}
