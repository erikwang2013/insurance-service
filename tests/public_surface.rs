//! 公开可见性 / 搜索 / trace 回归测试（任务 8）。
//!
//! 公开可见性与搜索用例由 `TEST_DATABASE_URL` 门控：未设置时直接返回（跳过而
//! 非失败）。trace 用例不触库，任何环境都会执行。

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use insurance_service::models::insurance_product::InsuranceProduct;
use insurance_service::routes::build_bee_router;
use serde_json::Value as Json;
use tower::ServiceExt;

use common::{build_state, call, db_url, insert_product};

// ---------- b. 公开可见性 / 搜索 ----------

#[tokio::test]
async fn public_catalog_hides_drafts_and_search_pagination_is_stable() {
    let Some(url) = db_url() else { return };
    let state = build_state(&url);
    let app = build_bee_router(state.clone());
    let db = &state.db;

    let (on_sale, code, name) = insert_product(db, InsuranceProduct::STATUS_ON_SALE, None).await;
    // 非在售商品名称包含在售商品的唯一关键字：若列表/搜索漏掉状态过滤即会命中
    let draft_name = format!("{name}草稿");
    let (draft, _, _) = insert_product(db, InsuranceProduct::STATUS_DRAFT, Some(&draft_name)).await;
    let off_shelf_name = format!("{name}下架");
    let (off_shelf, _, _) = insert_product(
        db,
        InsuranceProduct::STATUS_OFF_SHELF,
        Some(&off_shelf_name),
    )
    .await;

    // 公开列表忽略 status=DRAFT，只返回在售；DRAFT / OFF_SHELF 均不可见
    let (_, list) = call(
        &app,
        Method::GET,
        "/api/v1/products?status=DRAFT&size=100",
        None,
        None,
    )
    .await;
    assert_eq!(list["code"], 0, "{list}");
    let items = list["data"].as_array().expect("列表应为数组");
    assert!(
        items.iter().all(|p| p["status"] == "ON_SALE"),
        "公开列表混入非在售: {list}"
    );
    assert!(
        !items.iter().any(|p| p["id"].as_i64() == Some(draft)),
        "DRAFT 不应出现在公开列表: {list}"
    );
    assert!(
        !items.iter().any(|p| p["id"].as_i64() == Some(off_shelf)),
        "OFF_SHELF 不应出现在公开列表: {list}"
    );

    // 非在售详情 / 条款一律按不存在处理
    for id in [draft, off_shelf] {
        let (_, detail) = call(
            &app,
            Method::GET,
            &format!("/api/v1/products/{id}"),
            None,
            None,
        )
        .await;
        assert_eq!(detail["code"], 40400, "非在售详情应 404({id}): {detail}");
        let (_, clauses) = call(
            &app,
            Method::GET,
            &format!("/api/v1/products/{id}/clauses"),
            None,
            None,
        )
        .await;
        assert_eq!(clauses["code"], 40400, "非在售条款应 404({id}): {clauses}");
    }

    // 中文唯一关键字搜索：ON_SALE 名被非在售名称包含，状态过滤后 total 精确为 1
    let query =
        serde_urlencoded::to_string([("keyword", name.as_str()), ("page", "1"), ("size", "100")])
            .expect("搜索查询串编码失败");
    let (_, search) = call(
        &app,
        Method::GET,
        &format!("/api/v1/search?{query}"),
        None,
        None,
    )
    .await;
    assert_eq!(search["code"], 0, "{search}");
    assert_eq!(
        search["data"]["total"], 1,
        "唯一中文关键字应精确命中: {search}"
    );
    let hits = search["data"]["hits"].as_array().expect("hits 应为数组");
    let on_sale_doc = on_sale.to_string();
    let draft_doc = draft.to_string();
    let off_shelf_doc = off_shelf.to_string();
    assert!(
        hits.iter()
            .any(|h| h["doc_id"].as_str() == Some(on_sale_doc.as_str())),
        "在售商品应可搜索到: {search}"
    );
    assert!(
        !hits
            .iter()
            .any(|h| h["doc_id"].as_str() == Some(draft_doc.as_str())),
        "DRAFT 不应出现在搜索结果: {search}"
    );
    assert!(
        !hits
            .iter()
            .any(|h| h["doc_id"].as_str() == Some(off_shelf_doc.as_str())),
        "OFF_SHELF 不应出现在搜索结果: {search}"
    );

    // 超出页：窗口函数无行，回退 COUNT 后 total 仍正确（唯一编码 → total 恒为 1）
    let (_, past) = call(
        &app,
        Method::GET,
        &format!("/api/v1/search?keyword={code}&page=99&size=1"),
        None,
        None,
    )
    .await;
    assert_eq!(past["code"], 0, "{past}");
    assert_eq!(
        past["data"]["total"], 1,
        "空页 total 应由 COUNT 回退补全: {past}"
    );
    assert!(past["data"]["hits"].as_array().unwrap().is_empty());
}

// ---------- e. trace id：404 与 429 ----------

/// 404 fallback 与限流 429 的 `X-Trace-Id` 必须与信封 `trace_id` 一致。
#[tokio::test]
async fn trace_id_matches_between_header_and_envelope() {
    // 无需数据库：Db 为惰性连接池，404/429 都不会触库
    let app = build_bee_router(build_state("mysql://u:p@127.0.0.1:3306/unused"));

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/no-such-path")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let header_trace = resp.headers()["x-trace-id"].to_str().unwrap().to_string();
    let body: Json = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["code"], 40400);
    assert_eq!(body["trace_id"], header_trace);

    // 独立 router：耗尽自身限流窗口后第 301 次请求 429（/search 缺 keyword 不触库）
    let limited = build_bee_router(build_state("mysql://u:p@127.0.0.1:3306/unused"));
    let mut last = None;
    for _ in 0..301 {
        last = Some(
            limited
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/search")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap(),
        );
    }
    let resp = last.expect("至少发起一次请求");
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let header_trace = resp.headers()["x-trace-id"].to_str().unwrap().to_string();
    let body: Json = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["code"], 42900);
    assert_eq!(body["trace_id"], header_trace);
}
