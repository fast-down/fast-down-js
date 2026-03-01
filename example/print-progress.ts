import { prefetch, mergeProgress, Range } from '../dist/'
import { join, resolve } from 'node:path'
import { promises as fs } from 'node:fs'

const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'

async function main() {
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

  let progress: Range[] = []
  let intervalId = setInterval(() => {
    const curr_size = progress.reduce((acc, cur) => acc + cur.end - cur.start, 0)
    const percentage = Math.round((curr_size / task.info.size) * 100)
    console.log(`Downloaded ${curr_size} bytes (${percentage}%)`)
  }, 1000)
  console.time('Download')
  await task.start(path, (event) => {
    switch (event.type) {
      case 'PushProgress':
        if (event.range.start === 0) progress = []
        mergeProgress(progress, event.range)
        break
    }
  })
  clearInterval(intervalId)
  console.timeEnd('Download')
}

main()
