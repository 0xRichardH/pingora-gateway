use async_trait::async_trait;
use http::StatusCode;
use pingora::{http::ResponseHeader, proxy::Session};

use super::proxy::ProxyCtx;

#[async_trait]
pub trait FilterRequest: Send + Sync {
    async fn filter(&self, session: &mut Session, ctx: &mut ProxyCtx) -> pingora::Result<bool>;
}

#[derive(Clone)]
pub struct DefaultResponseFilter {}

#[async_trait]
impl FilterRequest for DefaultResponseFilter {
    async fn filter(&self, session: &mut Session, ctx: &mut ProxyCtx) -> pingora::Result<bool> {
        if ctx.get_request_path() == "/" {
            let mut resp_header = ResponseHeader::build(StatusCode::OK, None)?;
            resp_header.insert_header("Server", "Cloudflare")?;
            session.set_keepalive(None);
            session.write_response_header_ref(&resp_header).await?;
            session
                .write_response_body(Some("Connecting...".into()), true)
                .await?;

            return Ok(true);
        }

        return Ok(false);
    }
}

#[derive(Clone)]
pub struct SimplePathFilter {
    path: String,
}

impl SimplePathFilter {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn check_path(&self, path: &str) -> bool {
        path.starts_with(&self.path)
    }
}

#[async_trait]
impl FilterRequest for SimplePathFilter {
    async fn filter(&self, session: &mut Session, ctx: &mut ProxyCtx) -> pingora::Result<bool> {
        if !self.check_path(ctx.get_request_path().as_str()) {
            session
                .respond_error(StatusCode::NOT_FOUND.as_u16())
                .await?;

            return Ok(true);
        }

        Ok(false)
    }
}
