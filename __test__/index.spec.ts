import test from 'ava'
import { prefetch } from '../dist'
import { join, resolve } from 'node:path'
import { promises as fs } from 'node:fs'
import { createHash } from 'node:crypto'
import { createReadStream } from 'node:fs'
import { pipeline } from 'node:stream/promises'

test('sync function from native code', async (t) => {
  t.timeout(300000)

  const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'
  const task = await prefetch(URL, {
    proxy: 'no',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0',
    },
  })
  const filename = task.info.filename()
  const save_dir = resolve('download')
  await fs.mkdir(save_dir, { recursive: true })
  const path = join(save_dir, filename)
  console.log(path)
  const start = performance.now()
  console.time('Download')
  await task.start(path)
  const end = performance.now()
  console.timeEnd('Download')
  const speed = task.info.size / ((end - start) / 1000)
  console.log(`Download speed: ${formatSize(speed)}/s`)
  const hash = await sha256(path)
  console.log('File sha256:', hash)
  t.is(hash, 'c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972')
})

async function sha256(filePath: string) {
  const hash = createHash('sha256')
  const rs = createReadStream(filePath)
  await pipeline(rs, hash)
  return hash.digest('hex')
}

function formatSize(size: number) {
  const UNITS = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB']
  const LEN = UNITS.length
  let unitIndex = 0
  while (size >= 1024 && unitIndex < LEN - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(2)} ${UNITS[unitIndex]}`
}
