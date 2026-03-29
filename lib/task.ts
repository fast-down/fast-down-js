import { DownloadTask as RawDownloadTask, Range } from '../index.js'

export type Event =
  | { type: 'PrefetchError'; message: string }
  | { type: 'Pulling'; id: number }
  | { type: 'PullError'; id: number; message: string }
  | { type: 'PullTimeout'; id: number }
  | { type: 'PullProgress'; id: number; range: Range }
  | { type: 'Pushing'; id: number; range: Range }
  | { type: 'PushError'; id: number; message: string; range: Range }
  | { type: 'PushProgress'; id: number; range: Range }
  | { type: 'Flushing' }
  | { type: 'FlushError'; message: string }
  | { type: 'Finished'; id: number }

export interface Pusher {
  push: (offset: number, data: Uint8Array) => Promise<void>
  flush?: () => Promise<void>
}

export class DownloadTask {
  constructor(private _rawTask: RawDownloadTask) {}
  get info() {
    return this._rawTask.info
  }
  /** 彻底取消下载任务，不可恢复 */
  cancel() {
    this._rawTask.cancel()
  }
  isCancelled() {
    return this._rawTask.isCancelled()
  }
  /** 暂停下载任务，可恢复 */
  pause() {
    this._rawTask.pause()
  }
  isPaused() {
    return this._rawTask.isPaused()
  }
  async start(savePath: string, callback?: (event: Event) => void): Promise<void> {
    if (!callback) return this._rawTask.start(savePath)
    return this._rawTask.start(savePath, (rawEvent) => {
      callback(rawEvent as unknown as Event)
    })
  }
  async startInMemory(callback?: (event: Event) => void): Promise<Uint8Array> {
    return this._rawTask.startInMemory((rawEvent) => {
      callback?.(rawEvent as unknown as Event)
    })
  }
  async startWithPusher(pusher: Pusher, callback?: (event: Event) => void): Promise<void> {
    return this._rawTask.startWithPusher(
      async (args) => await pusher.push(args[0], args[1]),
      pusher.flush,
      (rawEvent) => {
        callback?.(rawEvent as unknown as Event)
      },
    )
  }
}
