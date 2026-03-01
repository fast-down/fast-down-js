import { DownloadTask as RawDownloadTask, Range } from '../index.js'

export type Event =
  | { type: 'PrefetchError'; message: string }
  | { type: 'Pulling'; id: number }
  | { type: 'PullError'; id: number; message: string }
  | { type: 'PullTimeout'; id: number }
  | { type: 'PullProgress'; id: number; range: Range }
  | { type: 'PushError'; id: number; message: string }
  | { type: 'PushProgress'; id: number; range: Range }
  | { type: 'FlushError'; message: string }
  | { type: 'Finished'; id: number }

export class DownloadTask {
  constructor(private _rawTask: RawDownloadTask) {}
  get info() {
    return this._rawTask.info
  }
  cancel() {
    this._rawTask.cancel()
  }
  isCancelled() {
    return this._rawTask.isCancelled()
  }
  async start(savePath: string, callback?: (event: Event) => void): Promise<void> {
    if (!callback) return this._rawTask.start(savePath)
    return this._rawTask.start(savePath, (rawEvent) => {
      callback(rawEvent as unknown as Event)
    })
  }
}
