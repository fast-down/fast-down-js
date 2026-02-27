# @fast-down/fast-down

[![GitHub last commit](https://img.shields.io/github/last-commit/fast-down/fast-down-js/main)](https://github.com/fast-down/fast-down-js/commits/main)
[![CI](https://github.com/fast-down/fast-down-js/workflows/CI/badge.svg)](https://github.com/fast-down/fast-down-js/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/fast-down/fast-down-js/blob/main/LICENSE)

`@fast-down/fast-down` 是一个特别快下载器，封装自 [fast-down-ffi](https://github.com/fast-down/ffi)，由 Rust 驱动，简洁易用。

## 示例

```ts
import { prefetch } from '@fast-down/fast-down'

const task = await prefetch('https://example.com/test.zip')
await task.start(task.info.filename())
```

[查看更多示例](https://github.com/fast-down/fast-down-js/blob/main/example)
