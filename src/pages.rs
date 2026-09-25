//! 面向浏览器人工访问的 HTML 页面（落地页 + 错误页）
//!
//! 边界：仅**非 API 路径**按 `Accept` 协商渲染 HTML；`/api/v1/*` 一律保持既有
//! JSON 信封契约（三端 App 依赖），浏览器误入 API 路径也只得到 JSON。
//!
//! 渲染走 `bee_template::TemplateEngine`（Tera），`.html` 插值自动转义——
//! 错误页回显请求路径不构成反射型 XSS。

use std::collections::HashMap;

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use bee_rust::bee_template::TemplateEngine;

use crate::controllers::AppState;
use crate::response::{ApiResponse, ResponseEnvelope};
use crate::routes::MASCOT_SVG;

/// 请求是否期望 HTML（浏览器；`Accept` 含 `text/html`）
pub fn accepts_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"))
}

/// 渲染模板体（注入吉祥物与版本号）；失败返回错误串，由调用方决定降级形态。
pub fn render_body(
    templates: &TemplateEngine,
    template: &str,
    mut data: HashMap<String, serde_json::Value>,
) -> Result<String, String> {
    data.insert("mascot".into(), serde_json::json!(MASCOT_SVG));
    data.insert("version".into(), serde_json::json!(env!("CARGO_PKG_VERSION")));
    templates.render(template, &data).map_err(|e| e.to_string())
}

/// 渲染模板 → HTML 响应；模板故障降级为纯文本状态行，不伪装成其它错误码。
pub fn render_page(
    templates: &TemplateEngine,
    template: &str,
    data: HashMap<String, serde_json::Value>,
    status: StatusCode,
) -> Response {
    match render_body(templates, template, data) {
        Ok(body) => (status, Html(body)).into_response(),
        Err(e) => {
            tracing::error!(template, error = %e, "HTML 页面渲染失败");
            let line = format!(
                "{} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("")
            );
            (status, line).into_response()
        }
    }
}

/// API 命名空间前缀（该前缀下永远是 JSON 信封，不渲染 HTML）
const API_PREFIX: &str = "/api/v1";

/// 统一 JSON 信封响应（非浏览器客户端的错误形态）
pub fn json_error(code: i32, msg: &str, status: StatusCode) -> Response {
    let envelope: ApiResponse = ResponseEnvelope::err(code, msg);
    (status, axum::Json(envelope)).into_response()
}

/// 未匹配路径：浏览器 → HTML 错误页；客户端 → JSON 信封 404（保持既有契约）
///
/// `/api/v1/*` 下即便带 `text/html` 的 Accept 也返回 JSON：三端 App 与
/// 代理可能透传浏览器式 Accept，API 契约面不应因请求头而改变。
pub fn not_found_response(templates: &TemplateEngine, html: bool, path: &str) -> Response {
    if !html || path.starts_with(API_PREFIX) {
        return json_error(40400, "资源不存在", StatusCode::NOT_FOUND);
    }
    let mut data = HashMap::new();
    data.insert("status".into(), serde_json::json!(404));
    data.insert("title".into(), serde_json::json!("页面不存在"));
    data.insert(
        "detail".into(),
        serde_json::json!(format!(
            "路径 {path} 没有对应的资源；若是接口调用，请改用 /api/v1 前缀。"
        )),
    );
    data.insert(
        "trace_id".into(),
        serde_json::json!(crate::middleware::trace::current_trace_id()),
    );
    render_page(templates, "error.html", data, StatusCode::NOT_FOUND)
}

/// `GET /` — 落地页（给服务一个像样的门面）
pub async fn landing(State(state): State<AppState>) -> Response {
    render_page(
        &state.templates,
        "landing.html",
        HashMap::new(),
        StatusCode::OK,
    )
}

/// 未匹配路由兜底（`Router::fallback`）
pub async fn not_found(State(state): State<AppState>, req: Request) -> Response {
    let html = accepts_html(req.headers());
    not_found_response(&state.templates, html, req.uri().path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn engine() -> TemplateEngine {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/templates");
        TemplateEngine::new(&dir).expect("src/templates 应可加载")
    }

    fn accept(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::ACCEPT, value.parse().unwrap());
        h
    }

    /// 浏览器 / 客户端判定：只有带 text/html 的 Accept 才算浏览器
    #[test]
    fn detects_browser_accept() {
        assert!(accepts_html(&accept("text/html,application/xhtml+xml,*/*;q=0.8")));
        assert!(accepts_html(&accept("text/html")));
        assert!(!accepts_html(&accept("application/json")));
        assert!(!accepts_html(&HeaderMap::new()), "无 Accept 头视为非浏览器");
    }

    /// 落地页：吉祥物内联（SVG 未被转义）、版本号与关键文案齐备
    #[test]
    fn landing_renders_inline_mascot() {
        let body = render_body(&engine(), "landing.html", HashMap::new()).expect("落地页应渲染成功");
        assert!(body.contains("<svg"), "应内联吉祥物 SVG");
        assert!(body.contains("url(#ice)"), "SVG 内容不应被转义");
        assert!(body.contains("保险服务平台"));
        assert!(body.contains(env!("CARGO_PKG_VERSION")), "应显示版本号");
    }

    /// 错误页：请求路径必须转义（防反射型 XSS），且带 trace_id 便于排障
    #[test]
    fn error_page_escapes_request_path() {
        let mut data = HashMap::new();
        data.insert("status".into(), serde_json::json!(404));
        data.insert("title".into(), serde_json::json!("页面不存在"));
        data.insert(
            "detail".into(),
            serde_json::json!("路径 /<script>alert(1)</script> 没有对应的资源"),
        );
        data.insert("trace_id".into(), serde_json::json!("t-1"));
        let body = render_body(&engine(), "error.html", data).expect("错误页应渲染成功");
        assert!(
            !body.contains("<script>alert(1)</script>"),
            "路径必须转义，不能原样注入"
        );
        assert!(body.contains("&lt;script&gt;"), "应出现转义后的路径");
        assert!(body.contains("t-1"), "应回显 trace_id");
    }

    /// 客户端拿到 JSON 信封而非 HTML
    #[test]
    fn api_clients_get_json_not_html() {
        let resp = not_found_response(&engine(), false, "/nope");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let ct = resp.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(ct.starts_with("application/json"), "实际: {ct}");
    }

    /// 浏览器拿到 HTML 错误页（Linux 下 Accept 命中即可，不依赖 DB）
    #[test]
    fn browsers_get_html_error_page() {
        let resp = not_found_response(&engine(), true, "/nope");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let ct = resp.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(ct.starts_with("text/html"), "实际: {ct}");
    }

    /// `/api/v1/*` 下即便 Accept 是 text/html 也返回 JSON（API 契约不因请求头改变）
    #[test]
    fn api_prefix_stays_json_even_for_browsers() {
        let resp = not_found_response(&engine(), true, "/api/v1/typo");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let ct = resp.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(ct.starts_with("application/json"), "实际: {ct}");
    }
}
