import test from 'ava'
import { prefetch } from '../dist'
import { join, resolve } from 'node:path'
import { promises as fs } from 'node:fs'
import { BinaryLike, createHash } from 'node:crypto'
import { createReadStream } from 'node:fs'
import { pipeline } from 'node:stream/promises'
import { arch } from 'node:process'

const is32Bit = arch === 'ia32' || arch === 'arm'

test.serial('自定义写入器测试-Node File API', async (t) => {
  t.timeout(300000)

  const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'
  const task = await prefetch(URL, {
    proxy: 'no',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0',
    },
  })
  const filename = '2-' + task.info.filename()
  const saveDir = resolve('download')
  await fs.mkdir(saveDir, { recursive: true })
  const path = join(saveDir, filename)
  console.log(path)
  const file = await fs.open(path, 'w')
  const start = performance.now()
  console.time('Download with Nodejs File API')
  await task.startWithPusher({
    push: async (offset, buf) => {
      await file.write(buf, { position: offset })
    },
  })
  await file.close()
  const end = performance.now()
  console.timeEnd('Download with Nodejs File API')
  const speed = task.info.size / ((end - start) / 1000)
  console.log(`Download speed: ${formatSize(speed)}/s`)
  const hash = await sha256File(path)
  console.log('File sha256:', hash)
  t.is(hash, 'c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972')
})
;(is32Bit ? test.skip : test.serial)('自定义写入器测试-写入内存', async (t) => {
  t.timeout(300000)

  const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'
  const task = await prefetch(URL, {
    proxy: 'no',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0',
    },
  })
  const start = performance.now()
  console.time('Download with Nodejs Uint8Array')
  const fileSize = task.info.size
  const data = new Uint8Array(fileSize)
  await task.startWithPusher({
    push: async (offset, buf) => {
      data.set(buf, offset)
    },
  })
  const end = performance.now()
  console.timeEnd('Download with Nodejs Uint8Array')
  const speed = task.info.size / ((end - start) / 1000)
  console.log(`Download speed: ${formatSize(speed)}/s`)
  const hash = sha256(Uint8Array.from(data))
  console.log('File sha256:', hash)
  t.is(hash, 'c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972')
})
test.serial('mmap 写入测试', async (t) => {
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
  const saveDir = resolve('download')
  await fs.mkdir(saveDir, { recursive: true })
  const path = join(saveDir, filename)
  console.log(path)
  const start = performance.now()
  console.time('Download with Mmap')
  await task.start(path)
  const end = performance.now()
  console.timeEnd('Download with Mmap')
  const speed = task.info.size / ((end - start) / 1000)
  console.log(`Download speed: ${formatSize(speed)}/s`)
  const hash = await sha256File(path)
  console.log('File sha256:', hash)
  t.is(hash, 'c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972')
})
;(is32Bit ? test.skip : test.serial)('下载到内存测试', async (t) => {
  t.timeout(300000)

  const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'
  const task = await prefetch(URL, {
    proxy: 'no',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0',
    },
  })
  const start = performance.now()
  console.time('Download in Memory')
  const data = await task.startInMemory()
  const end = performance.now()
  console.timeEnd('Download in Memory')
  const speed = task.info.size / ((end - start) / 1000)
  console.log(`Download speed: ${formatSize(speed)}/s`)
  const hash = sha256(data)
  console.log('File sha256:', hash)
  t.is(hash, 'c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972')
})

async function sha256File(filePath: string) {
  const hash = createHash('sha256')
  const rs = createReadStream(filePath)
  await pipeline(rs, hash)
  return hash.digest('hex')
}

function sha256(data: BinaryLike) {
  return createHash('sha256').update(data).digest('hex')
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
