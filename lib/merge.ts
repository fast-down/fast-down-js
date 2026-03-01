import { Range } from '../index.js'

export function mergeProgress(arr: Range[], range: Range) {
  let { start: cStart, end: cEnd } = range
  let left = 0
  let right = arr.length
  while (left < right) {
    let mid = Math.floor((left + right) / 2)
    if (arr[mid].end < cStart) left = mid + 1
    else right = mid
  }
  let i = left
  if (i === arr.length) {
    arr.push({ start: cStart, end: cEnd })
    return
  }
  if (arr[i].start <= cStart && arr[i].end >= cEnd) return
  let j = i
  while (j < arr.length) {
    let entry = arr[j]
    if (entry.start > cEnd) break
    if (entry.start < cStart) cStart = entry.start
    if (entry.end > cEnd) cEnd = entry.end
    j++
  }
  let deleteCount = j - i
  if (deleteCount === 1) {
    arr[i].start = cStart
    arr[i].end = cEnd
  } else arr.splice(i, deleteCount, { start: cStart, end: cEnd })
}
