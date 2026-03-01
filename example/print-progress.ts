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
  const saveDir = resolve('download')
  await fs.mkdir(saveDir, { recursive: true })
  const path = join(saveDir, filename)
  console.log(path)

  const fileSize = task.info.size
  let progress: Range[] = []
  const printProgress = () => {
    const currSize = progress.reduce((acc, cur) => acc + cur.end - cur.start, 0)
    const percentage = (currSize / fileSize) * 100
    console.log(`${currSize}/${fileSize} (${percentage.toFixed(2)}%)`)
  }
  const intervalId = setInterval(printProgress, 1000)
  console.time('Download')
  await task.start(path, (event) => {
    switch (event.type) {
      case 'PushProgress':
        if (event.range.start === 0) progress = []
        mergeProgress(progress, event.range)
        break
    }
  })
  printProgress()
  console.timeEnd('Download')
  clearInterval(intervalId)
}

main()
