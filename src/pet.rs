//! 项目宠物「霜霜」的表情形态——SVG 编译期内嵌，随二进制分发。
//!
//! 资源真源在 `docs/pet-expressions/`，README 与 HTTP 出口引用同一批文件；
//! `include_str!` 避免仓库里出现第二份内容而漂移。HTTP 出口见
//! [`svg_handler`]（`GET /pet/{slug}`）。

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

/// 一种表情形态
#[derive(Debug, Clone, Copy)]
pub struct Expression {
    /// URL 片段：`/pet/{slug}`
    pub slug: &'static str,
    /// 中文名（README / 文档使用）
    pub name: &'static str,
    /// 表情代码
    pub code: &'static str,
    /// 完整 SVG 源（编译期内嵌）
    pub svg: &'static str,
}

/// 12 种表情形态，顺序与 README 一致
pub const EXPRESSIONS: &[Expression] = &[
    Expression {
        slug: "smile",
        name: "笑",
        code: ":emoji-smile:",
        svg: include_str!("../docs/pet-expressions/smile.svg"),
    },
    Expression {
        slug: "joy",
        name: "欢乐",
        code: ":emoji-joy:",
        svg: include_str!("../docs/pet-expressions/joy.svg"),
    },
    Expression {
        slug: "cry",
        name: "哭",
        code: ":emoji-cry:",
        svg: include_str!("../docs/pet-expressions/cry.svg"),
    },
    Expression {
        slug: "depressed",
        name: "沮丧",
        code: ":emoji-depressed:",
        svg: include_str!("../docs/pet-expressions/depressed.svg"),
    },
    Expression {
        slug: "terrified",
        name: "惊恐",
        code: ":emoji-terrified:",
        svg: include_str!("../docs/pet-expressions/terrified.svg"),
    },
    Expression {
        slug: "angry",
        name: "愤怒",
        code: ":emoji-angry:",
        svg: include_str!("../docs/pet-expressions/angry.svg"),
    },
    Expression {
        slug: "coquettish",
        name: "撒娇",
        code: ":emoji-coquettish:",
        svg: include_str!("../docs/pet-expressions/coquettish.svg"),
    },
    Expression {
        slug: "cute",
        name: "卖萌",
        code: ":emoji-cute:",
        svg: include_str!("../docs/pet-expressions/cute.svg"),
    },
    Expression {
        slug: "pleasing",
        name: "讨好",
        code: ":emoji-pleasing:",
        svg: include_str!("../docs/pet-expressions/pleasing.svg"),
    },
    Expression {
        slug: "knocking",
        name: "敲打",
        code: ":emoji-knocking:",
        svg: include_str!("../docs/pet-expressions/knocking.svg"),
    },
    Expression {
        slug: "facepalm",
        name: "捂脸",
        code: ":emoji-facepalm:",
        svg: include_str!("../docs/pet-expressions/facepalm.svg"),
    },
    Expression {
        slug: "tongue",
        name: "吐舌头",
        code: ":emoji-tongue:",
        svg: include_str!("../docs/pet-expressions/tongue.svg"),
    },
];

/// 按 slug 查找（大小写敏感；未知返回 `None`）
pub fn find(slug: &str) -> Option<&'static Expression> {
    EXPRESSIONS.iter().find(|e| e.slug == slug)
}

/// `GET /pet/{slug}`：返回内嵌 SVG；未知 slug → 404 纯文本。
pub async fn svg_handler(Path(slug): Path<String>) -> Response {
    match find(&slug) {
        Some(expression) => {
            ([(header::CONTENT_TYPE, "image/svg+xml")], expression.svg).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!("未知表情：{slug}"),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 元数据完备：12 种、slug 唯一、SVG 可识别且含中文名
    #[test]
    fn metadata_is_consistent() {
        assert_eq!(EXPRESSIONS.len(), 12);
        let mut slugs: Vec<_> = EXPRESSIONS.iter().map(|e| e.slug).collect();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), EXPRESSIONS.len(), "slug 不应重复");

        for e in EXPRESSIONS {
            assert!(e.svg.starts_with("<svg "), "{} 应以 <svg 开头", e.slug);
            assert!(e.svg.ends_with("</svg>"), "{} 应以 </svg> 收尾", e.slug);
            assert!(e.svg.contains("aria-label="), "{} 缺少无障碍标签", e.slug);
            assert!(e.svg.contains(e.name), "{} 的 SVG 应含中文名", e.slug);
            assert!(
                e.code.starts_with(":emoji-") && e.code.ends_with(':'),
                "{} 表情代码格式错误",
                e.slug
            );
        }
    }

    /// SVG 自检：每个 `url(#id)` / `href="#id"` 都能找到对应 id 定义
    #[test]
    fn svg_references_resolve() {
        for e in EXPRESSIONS {
            for (at, _) in e.svg.match_indices("url(#") {
                let rest = &e.svg[at + "url(#".len()..];
                let id = &rest[..rest.find(')').expect("url(# 引用未闭合")];
                assert!(
                    e.svg.contains(&format!("id=\"{id}\"")),
                    "{}: url(#{id}) 缺少对应 id 定义",
                    e.slug
                );
            }
            for (at, _) in e.svg.match_indices("href=\"#") {
                let rest = &e.svg[at + "href=\"#".len()..];
                let id = &rest[..rest.find('"').expect("href 引用未闭合")];
                assert!(
                    e.svg.contains(&format!("id=\"{id}\"")),
                    "{}: href=#{id} 缺少对应 id 定义",
                    e.slug
                );
            }
        }
    }

    #[test]
    fn find_by_slug() {
        assert_eq!(find("smile").map(|e| e.name), Some("笑"));
        assert!(find("nope").is_none());
    }

    /// HTTP 出口：命中返回 SVG，未知 slug 返回 404
    #[tokio::test]
    async fn handler_serves_svg_and_404() {
        use axum::body::to_bytes;

        let resp = svg_handler(Path("smile".into())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "image/svg+xml");
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert!(body.starts_with(b"<svg"));

        let resp = svg_handler(Path("nope".into())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
