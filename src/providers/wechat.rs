//! 微信小程序登录渠道客户端（code2session，可配置骨架）
//!
//! 说明：微信 code2session 真实对接依赖外部凭据（WECHAT_APPID / WECHAT_SECRET），
//! 属「需外部凭据」渠道——凭据注入即激活；未配置（任一凭据为空）时**绝不发起
//! 网络请求**，直接返回含「未配置」的业务错误。微信侧成功/失败路径需凭据注入后
//! 手工验证（见 tests/wechat_channel_test.rs 文档注释）。

use std::time::Duration;

use serde::Deserialize;

use crate::config::WechatConfig;
use crate::error::{AppError, Result};

/// code2session 接口地址（微信官方）
const CODE2SESSION_URL: &str = "https://api.weixin.qq.com/sns/jscode2session";

/// code2session 成功返回的会话（openid / session_key / unionid）
#[derive(Debug, Clone)]
pub struct WechatSession {
    pub openid: String,
    pub session_key: String,
    pub unionid: Option<String>,
}

/// 微信登录渠道客户端
#[derive(Debug, Clone)]
pub struct WechatClient {
    cfg: WechatConfig,
    http: reqwest::Client,
}

impl WechatClient {
    /// 由渠道配置构造客户端（HTTP 层统一 10s 超时）
    pub fn from_config(cfg: WechatConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("构建 HTTP 客户端失败");
        Self { cfg, http }
    }

    /// 小程序登录 code 换会话（openid/session_key）
    ///
    /// - 未配置（app_id/app_secret 任一为空）→ 业务错误，不发请求
    /// - errcode 非 0 → 业务错误并携带微信 errmsg
    pub async fn code2session(&self, code: &str) -> Result<WechatSession> {
        if self.cfg.app_id.is_empty() || self.cfg.app_secret.is_empty() {
            return Err(AppError::business(
                "微信登录未配置:需设置 WECHAT_APPID / WECHAT_SECRET",
            ));
        }
        if code.is_empty() {
            return Err(AppError::validation("微信登录 code 不能为空"));
        }
        let text = self
            .http
            .get(CODE2SESSION_URL)
            .query(&[
                ("appid", self.cfg.app_id.as_str()),
                ("secret", self.cfg.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(AppError::internal)?
            .text()
            .await
            .map_err(AppError::internal)?;
        parse_response(&text)
    }
}

/// 解析 code2session 响应体（纯逻辑，可单测）
///
/// 成功响应 JSON：`{"openid":..,"session_key":..,"unionid"?:..}`；
/// 失败响应 JSON：`{"errcode":..,"errmsg":..}`（HTTP 200 + JSON 错误码）。
fn parse_response(text: &str) -> Result<WechatSession> {
    #[derive(Deserialize)]
    struct Raw {
        #[serde(default)]
        errcode: i64,
        #[serde(default)]
        errmsg: String,
        openid: Option<String>,
        session_key: Option<String>,
        #[serde(default)]
        unionid: Option<String>,
    }
    let raw: Raw = serde_json::from_str(text)?;
    if raw.errcode != 0 {
        return Err(AppError::business(format!(
            "微信登录失败(errcode {}):{}",
            raw.errcode, raw.errmsg
        )));
    }
    match (raw.openid, raw.session_key) {
        (Some(openid), Some(session_key)) => Ok(WechatSession {
            openid,
            session_key,
            unionid: raw.unionid,
        }),
        _ => Err(AppError::business(
            "微信登录失败:响应缺少 openid/session_key",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success_resp() {
        let s = parse_response(r#"{"openid":"o-1","session_key":"k-1","unionid":"u-1"}"#)
            .expect("成功响应应解析通过");
        assert_eq!(s.openid, "o-1");
        assert_eq!(s.session_key, "k-1");
        assert_eq!(s.unionid.as_deref(), Some("u-1"));
    }

    #[test]
    fn parse_success_resp_without_unionid() {
        let s = parse_response(r#"{"openid":"o-2","session_key":"k-2"}"#).expect("unionid 可缺省");
        assert_eq!(s.unionid, None);
    }

    #[test]
    fn parse_errcode_resp_is_business_error() {
        let err = parse_response(r#"{"errcode":40029,"errmsg":"invalid code"}"#)
            .expect_err("errcode 非 0 应返回错误");
        let msg = err.to_string();
        assert!(msg.contains("40029") && msg.contains("invalid code"), "实际: {msg}");
    }

    #[test]
    fn parse_missing_key_resp_is_error() {
        let err = parse_response(r#"{"openid":"o-3"}"#).expect_err("缺 session_key 应报错");
        assert!(err.to_string().contains("openid/session_key"));
    }
}
