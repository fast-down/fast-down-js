import { prefetch } from '..'
import { join, resolve } from 'node:path'
import { promises as fs } from 'node:fs'

const URL =
  'https://software.download.prss.microsoft.com/dbazure/Win11_25H2_Pro_Chinese_Simplified_x64.iso?t=40a41c61-f324-4b94-80ed-c6f7db5e591a&P1=1772243337&P2=601&P3=2&P4=BfXe6ch%2bgL6laiXUtKBlSpNSZ02ToktN6CtImJoYs%2bzsOIc9UQBCEeiC4nFNEmmFzNDmbUoTxWx4APJE7nbgFYEsdqiBVspvJSukOlOcXPJLpK%2fwQWeVxaCzgItfXV52UkEXU7NENvtXBZXUGGrAHUrjETQ65pQ7f31Qrbj4Gm90gZaUrFH30nYUatGLGC2db2CI0zfLVz8NACDKmbtgKLvLgpk2ZF%2fuSvGFB7gA5%2bl7qavaQqUwfE6MKJJSow8tdpXdv4gyQq2Ohk6L1qYqkaJIv0iGfxgjuAzhNjHGRiffNICp1bNcm%2bfBHbQij%2fPjioL0eoIfV3xBaNWa15br0g%3d%3d'

async function main() {
  const task = await prefetch(URL, { proxy: 'no' })
  setTimeout(() => {
    task.cancel()
    console.log('Download canceled')
  }, 3000)
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
