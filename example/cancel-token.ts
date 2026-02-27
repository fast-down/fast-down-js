import { CancellationToken, prefetch } from '..'
import { join, resolve } from 'node:path'
import { promises as fs } from 'node:fs'

const URL = 'https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso'

async function main() {
  const token = new CancellationToken()
  setTimeout(() => {
    token.cancel()
    console.log('Download canceled', token.isCancelled())
  }, 3000)
  const task = await prefetch(
    URL,
    {
      proxy: 'no',
      headers: {
        'User-Agent':
          'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0',
      },
    },
    token,
  )
  const filename = task.info.filename()
  const save_dir = resolve('downlaod')
  try {
    await fs.mkdir(save_dir, { recursive: true })
  } catch (error) {
    console.error('Error creating directory:', error)
  }
  const path = join(save_dir, filename)
  console.log(path)
  console.time('Download')
  await task.start(path)
  console.timeEnd('Download')
}

main()
