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
/// # ponytail: 全局单锁 Mutex<HashMap>，按 key 粒度加锁/分片可提升并发吞吐；
/// 窗口记录只增不减，若 key 基数大且长期活跃，需定期清理过期窗口，当前场景（IP/用户维度）规模可忽略。
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
    /// - 距窗口起点已 ≥ window_secs → 视为新窗口，重置计数
    /// - 同窗口内计数 ≥ max_requests → 拒绝（false），否则计数 +1 放行（true）
    pub fn check_at(&self, key: &str, now_secs: u64) -> bool {
        let mut windows = self.windows.lock().unwrap_or_else(|poison| poison.into_inner());
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
}
