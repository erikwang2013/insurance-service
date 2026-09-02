//! 保险服务平台后端入口（bee-rust 阶段 0）
//!
//! 流程：`bee_rust::init()`（含日志初始化）→ 加载配置 → 装配 bee Router → axum serve。
//!
//! 说明：bee-rust 已通过本地 path 依赖激活（见 Cargo.toml [workspace.dependencies]
//! 注释），此处直接启动真实 HTTP 服务；业务过滤器链与控制器随阶段 1 接入。

use tracing::{error, info};

use insurance_service::config::AppConfig;
use insurance_service::controllers::AppState;
use insurance_service::routes;

/// 应用启动入口
#[tokio::main]
async fn main() {
    // 1. 初始化 bee-rust（含日志；LogHandle 须存活至进程退出）
    let _log_handle = match bee_rust::init() {
        Ok(h) => h,
        Err(e) => {
            error!("bee_rust::init() 失败: {e}");
            std::process::exit(1);
        }
    };

    // 吉祥物安安上场（控制台问候）
    println!("{}\n  🛡️ 安安 says：保险路上，为你护航！\n", routes::MASCOT_BANNER);

    // 2. 加载配置（缺失时无法运行，直接退出）
    let cfg = match AppConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("配置加载失败: {e}，请按 .env.example 配置环境变量");
            std::process::exit(1);
        }
    };
    info!(addr = %cfg.bind_addr(), db = %cfg.database.url, "配置加载完成");

    // 3. 装配控制器状态（阶段 1：模板引擎 / 会话缓存 / 业务 Controller）
    let state = match AppState::new(&cfg) {
        Ok(s) => s,
        Err(e) => {
            error!("应用状态装配失败: {e}");
            std::process::exit(1);
        }
    };

    // 4. 装配路由并启动
    let app = routes::build_bee_router(state);
    let addr = cfg.bind_addr();
    info!("bee-rust 服务监听 {addr} ...");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("无法绑定 {addr}: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        error!("服务异常退出: {e}");
        std::process::exit(1);
    }
}

