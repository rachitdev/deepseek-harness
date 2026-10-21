/**
 * Model-facing read, read_image, write, and edit tools over `ctx.fs`. This package owns schemas, validation,
 * read windows, formatting, and observation events, never a concrete provider. An optional
 * event policy supplies mutation guards; without one the tools use unconditional provider calls.
 * @module @deepseek-ai/dsh-tool-fs
 */

import type {} from '@deepseek-ai/dsh-working-directory'
import type { Context } from '@deepseek-ai/cordis'
import { isAbsolute } from 'node:path'
import z from '@deepseek-ai/schemastery'
import type {} from '@deepseek-ai/dsh-user-approval'
import { applyReadTool, READ_LIMIT, STREAM_MIN_SIZE } from './read.ts'
import { applyWriteTool } from './write.ts'
import { applyEditTool } from './edit.ts'
import { applyReadImageTool, OCR_TEXT_MAX_BYTES, OCR_TIMEOUT_MS } from './read-image.ts'
import type { ReadImageOcrOptions } from './read-image.ts'
import { READ_MAX_BYTES, READ_MAX_LINE_LENGTH } from './read-render.ts'
import { FsSandboxController } from './sandbox.ts'

/** Cordis plugin name used by loader diagnostics. */
export const name = 'tool-fs'

/** Services required by the filesystem tool suite. */
export const inject = ['tools', 'fs', 'systemPrompt', 'workingDirectory']

/** Plugin config (all optional — `Config` supplies the defaults). */
export interface Config {
  /** Register `read` and its guidance. Defaults to true. */
  read?: boolean
  /** Register `read_image` while an attachment store is mounted. Defaults to true. */
  readImage?: boolean
  /** Register `write` and its guidance. Defaults to true. */
  write?: boolean
  /** Register `edit` and its guidance. Defaults to true. */
  edit?: boolean
  /** Default and maximum number of lines returned by one `read` call. */
  readLimit?: number
  /** Maximum characters returned for a single line before truncation. */
  readMaxLineLength?: number
  /** Maximum bytes returned for the selected lines of one `read` call. */
  readMaxBytes?: number
  /** Files at or above this size stream instead of loading whole into memory. */
  readStreamMinSize?: number
  /**
   * Absolute path to an OCR executable that returns recognized text for an
   * image. Set, a route that declares no image input receives that text
   * instead of the strict refusal; unset, `read_image` refuses exactly as before.
   */
  readImageOcr?: string
  /** Arguments for `readImageOcr`, placed before the staged image path it receives. */
  readImageOcrArgs?: string[]
  /** Recognition language recorded with a result; pass the engine's own language flag through `readImageOcrArgs`. */
  readImageOcrLang?: string
  /** Deadline for one OCR invocation, in milliseconds. */
  readImageOcrTimeoutMs?: number
  /** Cap on recognized text returned to the model, in bytes. */
  readImageOcrMaxBytes?: number
}

export const Config: z<Config> = z.object({
  read: z.boolean().default(true),
  readImage: z.boolean().default(true),
  write: z.boolean().default(true),
  edit: z.boolean().default(true),
  readLimit: z.number().default(READ_LIMIT),
  readMaxLineLength: z.number().default(READ_MAX_LINE_LENGTH),
  readMaxBytes: z.number().default(READ_MAX_BYTES),
  readStreamMinSize: z.number().default(STREAM_MIN_SIZE),
  readImageOcr: z.string(),
  readImageOcrArgs: z.array(z.string()).default([]),
  readImageOcrLang: z.string(),
  readImageOcrTimeoutMs: z.number().default(OCR_TIMEOUT_MS),
  readImageOcrMaxBytes: z.number().default(OCR_TEXT_MAX_BYTES),
})

/**
 * Configuration after schemastery applied the defaults. The OCR command and its
 * language stay optional: an absent command is the deployment's opt-out, not a
 * defaulted value.
 */
type ResolvedConfig = Required<Omit<Config, 'readImageOcr' | 'readImageOcrLang'>>
  & Pick<Config, 'readImageOcr' | 'readImageOcrLang'>

/** Every read cap counts lines/chars/bytes — a positive integer, or windowing arithmetic misbehaves silently. */
function assertPositiveInteger(name: string, value: number): void {
  if (!Number.isInteger(value) || value < 1) {
    throw new Error(`tool-fs: ${name} must be a positive integer`)
  }
}

/** Compose `read_image`'s OCR settings, or undefined when the deployment configured no command. */
function readImageOcrOptions(resolved: ResolvedConfig): ReadImageOcrOptions | undefined {
  const command = resolved.readImageOcr
  if (command === undefined) return undefined
  return {
    command,
    args: resolved.readImageOcrArgs,
    ...resolved.readImageOcrLang === undefined ? {} : { lang: resolved.readImageOcrLang },
    timeoutMs: resolved.readImageOcrTimeoutMs,
    maxTextBytes: resolved.readImageOcrMaxBytes,
  }
}

/** Register the selected filesystem tools; `read_image` additionally requires a mounted attachment store. */
export function apply(ctx: Context, config: Config): void {
  // schemastery (Config) has already filled every defaulted field.
  const resolved = config as ResolvedConfig
  assertPositiveInteger('readLimit', resolved.readLimit)
  assertPositiveInteger('readMaxLineLength', resolved.readMaxLineLength)
  assertPositiveInteger('readMaxBytes', resolved.readMaxBytes)
  assertPositiveInteger('readStreamMinSize', resolved.readStreamMinSize)
  // An OCR command is a deployment path, never a bare name: the child runs
  // without a shell, so a relative value could only resolve against whatever
  // directory the harness happens to have. Its caps matter only once a command
  // exists, so they are validated together with it; misconfiguration fails at load.
  const ocrCommand = resolved.readImageOcr
  if (ocrCommand !== undefined) {
    if (!isAbsolute(ocrCommand)) {
      throw new Error(`tool-fs: readImageOcr must be an absolute path to an OCR executable, got "${ocrCommand}"`)
    }
    assertPositiveInteger('readImageOcrTimeoutMs', resolved.readImageOcrTimeoutMs)
    assertPositiveInteger('readImageOcrMaxBytes', resolved.readImageOcrMaxBytes)
  }
  if (resolved.read) {
    applyReadTool(ctx, {
      limit: resolved.readLimit,
      maxLineLength: resolved.readMaxLineLength,
      maxBytes: resolved.readMaxBytes,
      streamMinSize: resolved.readStreamMinSize,
    })
  }
  // read_image is composition-conditional: without a mounted attachment store
  // the deployment cannot durably commit image bytes, so the tool never
  // registers; the execute body keeps a defensive re-check for direct callers.
  if (resolved.readImage) {
    ctx.inject(['attachments'], (imageCtx) => {
      applyReadImageTool(imageCtx, readImageOcrOptions(resolved))
    })
  }
  // One escalation API shared by both mutating tools: advertisement gating,
  // per-call policy resolution, and denial-marker mapping, all keyed off whether
  // the mounted ctx.fs confines (ctx.fs.sandboxMode).
  if (resolved.write || resolved.edit) {
    const sandbox = new FsSandboxController(ctx)
    if (resolved.write) applyWriteTool(ctx, sandbox)
    if (resolved.edit) applyEditTool(ctx, sandbox)
  }
}
