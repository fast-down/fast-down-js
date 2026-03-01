use crate::Range;
use fast_down_ffi::utils::Proxy;
use napi_derive::napi;
use std::{collections::HashMap, time::Duration};

#[napi(object)]
pub struct Config {
  /// 线程数量，默认值 `32`。线程越多不意味着越快
  pub threads: Option<u32>,
  /// 设置代理，默认值 `system`。支持 https、http、socks5 代理
  ///
  /// - `"no"` => `Proxy::No`
  /// - `"system"` => `Proxy::System`
  /// - `proxy_str` => `Proxy::Custom(proxy_str)`
  #[napi(ts_type = "'no' | 'system' | (string & {})")]
  pub proxy: Option<String>,
  /// 自定义请求头
  pub headers: Option<HashMap<String, String>>,
  /// 最小分块大小，单位为字节，默认值 `8 * 1024 * 1024`
  ///
  /// - 分块太小容易造成强烈竞争
  /// - 当无法分块的时候会进入冗余竞争模式
  pub min_chunk_size: Option<u32>,
  /// 写入缓冲区大小，单位为字节，默认值 `16 * 1024 * 1024`
  ///
  /// - 只对 `std` 写入方法有效，有利于将随机写入转换为顺序写入，提高写入速度
  /// - 对于 `mmap` 写入方法无效，因为写入缓冲区由系统决定
  pub write_buffer_size: Option<u32>,
  /// 写入队列容量，默认值 `10240`
  ///
  /// 如果下载线程太快，填满了写入队列，会触发压背，降低下载速度，防止内存占用过大
  pub write_queue_cap: Option<u32>,
  /// 请求失败后的默认重试间隔，默认值 `500ms`
  ///
  /// 如果服务器返回中有 `Retry-After` 头，则遵循服务器返回的设定
  pub retry_gap_ms: Option<u32>,
  /// 拉取超时时间，默认值 `5000ms`
  ///
  /// 请求发出后，接收字节中，如果在 `pull_timeout` 这一段时间内一个字节也没收到，则中断连接，重新请求。
  /// 有利于触发 TCP 重新检测拥塞状态，提高下载速度
  pub pull_timeout_ms: Option<u32>,
  /// 是否接受无效证书（危险），默认值 `false`
  pub accept_invalid_certs: Option<bool>,
  /// 是否接受无效主机名（危险），默认值 `false`
  pub accept_invalid_hostnames: Option<bool>,
  /// 写入磁盘方式，默认值 `"mmap"`
  ///
  /// - `"mmap"` 写入方式速度最快，将写入交给操作系统执行，但是：
  ///     1. 在 32 位系统上最大只能映射 4GB 的文件，所以在 32 位系统上，会自动回退到 `"std"`
  ///     2. 必须知道文件大小，否则会自动回退到 `"std"`
  ///     3. 特殊情况下会出现系统把所有数据全部缓存在内存中，下载完成后一次性写入磁盘，造成下载完成后长时间卡顿
  /// - `"std"` 写入方式兼容性最好，会在 `write_buffer_size` 内对片段进行排序，尽量转换为顺序写入
  #[napi(ts_type = "'mmap' | 'std'")]
  pub write_method: Option<String>,
  /// 设置获取元数据的重试次数，默认值 `10`。注意，这不是下载中的重试次数
  pub retry_times: Option<u32>,
  /// 使用哪些地址来发送请求，默认值 `Vec::new()`
  ///
  /// 如果你有多个网卡可用，可以填写他们的对外 IP 地址，请求会在这些 IP 地址上轮换，下载不一定会更快
  pub local_address: Option<Vec<String>>,
  /// 冗余线程数，默认值 `3`
  ///
  /// 当块大小小于 `min_chunk_size` 后无法分块，进入冗余竞争模式。
  /// 最多有 `max_speculative` 个线程在同一分块上竞争下载，以解决下载卡进度 99% 的问题
  pub max_speculative: Option<u32>,
  /// 已经下载过的部分，如果你想下载整个文件，就传 `Vec::new()`
  pub downloaded_chunk: Option<Vec<Range>>,
  /// 已下载分块的平滑窗口，单位为字节，默认值 `8 * 1024`
  ///
  /// 它会过滤掉 `downloaded_chunk` 中小于 `chunk_window` 的小空洞，以减小 HTTP 请求数量
  pub chunk_window: Option<u32>,
}

impl Config {
  #[must_use]
  pub fn to_ffi_config(&self) -> fast_down_ffi::Config {
    fast_down_ffi::Config {
      threads: self.threads.unwrap_or(32) as usize,
      proxy: match self.proxy.as_deref() {
        Some("no") => Proxy::No,
        Some("system") | None => Proxy::System,
        Some(p) => Proxy::Custom(p.to_string()),
      },
      headers: self.headers.clone().unwrap_or_default(),
      min_chunk_size: self.min_chunk_size.unwrap_or(8 * 1024 * 1024).into(),
      write_buffer_size: self.write_buffer_size.unwrap_or(16 * 1024 * 1024) as usize,
      write_queue_cap: self.write_queue_cap.unwrap_or(10240) as usize,
      retry_gap: Duration::from_millis(self.retry_gap_ms.unwrap_or(500).into()),
      pull_timeout: Duration::from_millis(self.pull_timeout_ms.unwrap_or(5000).into()),
      accept_invalid_certs: self.accept_invalid_certs.unwrap_or(false),
      accept_invalid_hostnames: self.accept_invalid_hostnames.unwrap_or(false),
      write_method: match self.write_method.as_deref().unwrap_or_default() {
        "std" => fast_down_ffi::WriteMethod::Std,
        _ => fast_down_ffi::WriteMethod::Mmap,
      },
      retry_times: self.retry_times.unwrap_or(10) as usize,
      local_address: self
        .local_address
        .as_ref()
        .map(|e| e.iter().filter_map(|p| p.parse().ok()).collect())
        .unwrap_or_default(),
      max_speculative: self.max_speculative.unwrap_or(3) as usize,
      #[allow(clippy::cast_sign_loss)]
      downloaded_chunk: self
        .downloaded_chunk
        .as_ref()
        .map(|e| e.iter().map(|p| p.start as u64..p.end as u64).collect())
        .unwrap_or_default(),
      chunk_window: self.chunk_window.unwrap_or(8 * 1024).into(),
    }
  }
}
