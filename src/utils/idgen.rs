//! 全局雪花主键生成（idgen_rs 0.2.0 封装）
//!
//! 约定：业务层统一调用 [`next_id`]，应用层生成 id 后显式 INSERT（主键自 AUTO_INCREMENT
//! 切换为 snowflake）。线程安全：库内部为 AtomicU64 + CAS 无锁实现，此处仅需同步一次惰性初始化。

use std::sync::Once;

/// 首次调用时按环境变量 `IDGEN_WORKER_ID`（默认 0）初始化全局生成器
static INIT: Once = Once::new();

/// 生成下一个 snowflake 主键（>0，全局唯一）
pub fn next_id() -> i64 {
    INIT.call_once(|| {
        // 机器码：多实例部署时各实例需配置不同 IDGEN_WORKER_ID（0~63，默认位长 6）
        let worker_id = std::env::var("IDGEN_WORKER_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        idgen_rs::snowflake_init(worker_id);
    });
    // snowflake 63 位内为正；库返回 u64，位长配置未超 22 时不可能溢出 i64
    i64::try_from(idgen_rs::id_helper::next_id()).expect("snowflake id 超出 i64 范围")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_positive_and_unique() {
        // 单测不依赖 MySQL；同一毫秒内序列号自增，仅断言正数与唯一性
        let ids: Vec<i64> = (0..1000).map(|_| next_id()).collect();
        assert!(ids.iter().all(|&id| id > 0));
        let uniq: HashSet<i64> = ids.iter().copied().collect();
        assert_eq!(uniq.len(), ids.len(), "存在重复 id");
    }
}
