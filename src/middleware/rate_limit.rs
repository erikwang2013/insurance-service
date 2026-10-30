//! 固定窗口限流器（纯逻辑，不依赖 DB / 真实时钟）
//!
//! 挂载方式：由 routes.rs 用 `axum::middleware::from_fn_with_state` 接入，
//! 本模块只保证类型层可直接被共享（`Clone` + 原子内部状态）。
//!
//! 简化说明：固定窗口即可满足当前需求——同窗口计数 ≥ 上限即拒绝，窗口边界
//! （距窗口起点 ≥ window_secs）到来时整体重置。未采用滑动窗口/令牌桶，
//! 二者在突发平滑上更优，但引入复杂度，后续需要再升级。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// 单个 key 在某个窗口内的计数
struct Window {
    count: u64,
    /// 窗口起点（unix 秒）；窗口区间为 [start_secs, start_secs + window_secs)
    start_secs: u64,
}

/// 固定窗口限流器：按 key 独立计数，跨窗口自动重置。
///
/// 内存边界：key 数量达到 10_000 时清理过期窗口；清理后仍 ≥ 100_000 时
/// 直接拒绝（fail closed），避免 key 泛滥拖垮内存。
///
/// # ponytail: 全局单锁 Mutex<HashMap>，按 key 粒度加锁/分片可提升并发吞吐；
/// 当前场景（IP 维度）规模可忽略，需要更高吞吐时再分片。
#[derive(Clone)]
pub struct RateLimiter {
    max_requests: u64,
    window_secs: u64,
    windows: Arc<Mutex<HashMap<String, Window>>>,
}

impl RateLimiter {
    /// 新建限流器：每个 key 在 window_secs 秒窗口内最多放行 max_requests 次。
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 核心判定（时钟可注入，便于测试）：
    /// - key 数量达上限阈值时先清理过期窗口；仍达硬上限 → fail closed
    /// - 距窗口起点已 ≥ window_secs → 视为新窗口，重置计数
    /// - 同窗口内计数 ≥ max_requests → 拒绝（false），否则计数 +1 放行（true）
    pub fn check_at(&self, key: &str, now_secs: u64) -> bool {
        let mut windows = self.windows.lock().unwrap_or_else(|poison| poison.into_inner());
        if windows.len() >= 10_000 {
            let window_secs = self.window_secs;
            windows.retain(|_, w| now_secs.saturating_sub(w.start_secs) < window_secs);
            if windows.len() >= 100_000 {
                return false;
            }
        }
        let entry = windows
            .entry(key.to_string())
            .or_insert(Window { count: 0, start_secs: now_secs });

        // 新窗口：重置起点与计数。saturating_sub 兜底时钟回拨场景。
        if now_secs.saturating_sub(entry.start_secs) >= self.window_secs {
            entry.count = 0;
            entry.start_secs = now_secs;
        }
        if entry.count >= self.max_requests {
            return false;
        }
        entry.count += 1;
        true
    }

    /// 对外便捷入口：用系统真实时间（unix 秒）判定。
    pub fn allow(&self, key: &str) -> bool {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.check_at(key, now_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 第 N+1 次拒绝：窗口内放行恰好 max_requests 次，之后同窗口一律拒绝
    #[test]
    fn rejects_after_max_in_window() {
        let rl = RateLimiter::new(3, 60);
        assert!(rl.check_at("ip-1", 1_000));
        assert!(rl.check_at("ip-1", 1_001));
        assert!(rl.check_at("ip-1", 1_002));
        assert!(!rl.check_at("ip-1", 1_003), "第 4 次应被拒绝");
        assert!(!rl.check_at("ip-1", 1_004), "窗口内后续仍拒绝");
    }

    /// 跨窗口自动重置：窗口边界（start + window_secs 那一秒）算新窗口
    #[test]
    fn resets_on_new_window() {
        let rl = RateLimiter::new(2, 10);
        // 窗口 [100, 110)
        assert!(rl.check_at("k", 100));
        assert!(rl.check_at("k", 105));
        assert!(!rl.check_at("k", 109), "窗口内第 3 次拒绝");
        // t=110 恰好进入新窗口 [110, 120)，重置后可再次放行
        assert!(rl.check_at("k", 110), "跨窗口应自动重置");
        assert!(rl.check_at("k", 119));
        assert!(!rl.check_at("k", 119), "新窗口内也超限");
    }

    /// 不同 key 独立计数：一个 key 耗尽不影响另一个
    #[test]
    fn keys_are_independent() {
        let rl = RateLimiter::new(2, 60);
        assert!(rl.check_at("a", 100));
        assert!(rl.check_at("a", 101));
        assert!(!rl.check_at("a", 102));
        assert!(rl.check_at("b", 103), "key b 应不受 key a 影响");
        assert!(rl.check_at("b", 104));
        assert!(!rl.check_at("b", 105));
    }

    /// 窗口边界 exactly：半开区间 [start, start + window)，临界秒重置
    #[test]
    fn window_boundary_is_exact() {
        let rl = RateLimiter::new(1, 10);
        assert!(rl.check_at("k", 0), "窗口 [0, 10) 首个请求");
        assert!(!rl.check_at("k", 9), "窗口内第 2 次拒绝");
        assert!(rl.check_at("k", 10), "t=10 恰为新窗口 [10, 20)");
        assert!(!rl.check_at("k", 19), "仍处 [10, 20)，拒绝");
    }

    /// 达到 10_000 阈值时清理过期窗口：过期 key 被移除，活跃 key 保留
    #[test]
    fn cleans_expired_windows_at_threshold() {
        let rl = RateLimiter::new(1, 10);
        {
            let mut windows = rl.windows.lock().unwrap();
            for i in 0..5_000 {
                windows.insert(format!("old-{i}"), Window { count: 1, start_secs: 0 });
            }
            for i in 0..5_000 {
                windows.insert(format!("new-{i}"), Window { count: 1, start_secs: 100 });
            }
        }
        // t=100 触发清理：old-* 已过期（100-0 >= 10），new-* 仍在窗口内
        assert!(rl.check_at("fresh", 100));
        let len = rl.windows.lock().unwrap().len();
        assert_eq!(len, 5_001, "过期窗口被清理，仅活跃 + 新 key 保留");
    }

    /// 硬上限：清理后仍 ≥ 100_000 个活跃 key → fail closed，不再增长
    #[test]
    fn fails_closed_at_hard_cap() {
        let rl = RateLimiter::new(1, 10);
        {
            let mut windows = rl.windows.lock().unwrap();
            for i in 0..100_000 {
                windows.insert(format!("k-{i}"), Window { count: 1, start_secs: 100 });
            }
        }
        assert!(!rl.check_at("another", 100), "满额且无过期窗口时应拒绝");
        assert_eq!(rl.windows.lock().unwrap().len(), 100_000, "拒绝时不写入新 key");
    }
}
