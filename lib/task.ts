import { DownloadTask as RawDownloadTask, Range } from '../index.js'

export type Event =
  | { type: 'PrefetchError'; message: string; id?: never; range?: never }
  | { type: 'Pulling'; id: number; message?: never; range?: never }
  | { type: 'PullError'; id: number; message: string; range?: never }
  | { type: 'PullTimeout'; id: number; message?: never; range?: never }
  | { type: 'PullProgress'; id: number; range: Range; message?: never }
  | { type: 'PushError'; id: number; message: string; range?: never }
  | { type: 'PushProgress'; id: number; range: Range; message?: never }
  | { type: 'FlushError'; message: string; id?: never; range?: never }
  | { type: 'Finished'; id: number; message?: never; range?: never }

export class DownloadTask {
  constructor(private _rawTask: RawDownloadTask) {}
  get info() {
    return this._rawTask.info
  }
  cancel() {
    this._rawTask.cancel()
  }
  async start(savePath: string, callback?: (event: Event) => void): Promise<void> {
    if (!callback) return this._rawTask.start(savePath)
    return this._rawTask.start(savePath, (rawEvent) => {
      callback(rawEvent as unknown as Event)
    })
  }
}
