import type { UserMessage } from '@deepseek-ai/dsh-llm'

/**
 * Recall producers this spec mounts by hand. The memory plugins that declare these kinds in
 * production were retired, and `MessageSourceMap` is merge-extensible, so the fixture states the
 * two kinds the filter recognizes instead of asserting an incomplete source into the union.
 */
declare module '@deepseek-ai/dsh-llm' {
  interface MessageSourceMap {
    'plugin:openviking-memory': {
      readonly kind: 'plugin:openviking-memory'
      readonly plugin: string
      readonly form: 'recall'
    }
    'cortexdb-memory': { readonly kind: 'cortexdb-memory' }
  }
}

/**
 * Build the source of one fixture recall plugin.
 * @param plugin - the memory plugin the recall text came from.
 * @returns a user-message source the filter reads as recall.
 */
export function recallSource(plugin = 'openviking-memory'): UserMessage['source'] {
  return plugin === 'cortexdb-memory'
    ? { kind: 'cortexdb-memory' }
    : { kind: 'plugin:openviking-memory', plugin, form: 'recall' }
}
