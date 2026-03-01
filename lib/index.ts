import { CancellationToken, Config, Event, Range, UrlInfo, WriteMethod, prefetch as rawPrefetch } from '../index.js'
import { DownloadTask } from './task.js'

export async function prefetch(url: string, config?: Config & { signal?: AbortSignal }) {
  if (!config?.signal) return new DownloadTask(await rawPrefetch(url, config))
  const token = new CancellationToken()
  config.signal.addEventListener('abort', () => token.cancel())
  return new DownloadTask(await rawPrefetch(url, config, token))
}

export { Config, DownloadTask, Event, Range, UrlInfo, WriteMethod }
export * from './merge.js'
