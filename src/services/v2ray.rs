use async_trait::async_trait;
use log::info;
use pingora::{
    http::{ResponseHeader, StatusCode},
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};

pub struct V2rayService {
    host: String,    // e.g. one.one.one.one
    address: String, // e.g. 127.0.0.1:10086
    ws_path: String, // e.g. /ray
    is_tls: bool,
}

#[async_trait]
impl ProxyHttp for V2rayService {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        if !self.check_host(session) {
            session.respond_error(404).await;

            return Ok(true);
        }

        let request_path = session.req_header().uri.path();

        if request_path == "/" {
            let mut resp_header = ResponseHeader::build(StatusCode::OK, None)?;
            resp_header.insert_header("Server", "Cloudflare")?;
            session.set_keepalive(None);
            session.write_response_header_ref(&resp_header).await?;
            session.write_response_body("Connecting...".into()).await?;

            // true: early return as the response is already written
            return Ok(true);
        }

        if !self.check_ws_path(request_path) {
            session.respond_error(404).await;

            // true: early return as the response is already written
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let peer = Box::new(HttpPeer::new(&self.address, self.is_tls, self.host.clone()));
        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace existing header if any
        upstream_response.insert_header("Server", "Cloudflare")?;

        // because pingora doesn't support h3
        upstream_response.remove_header("alt-svc");

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        info!(
            "{} response code: {response_code}",
            self.request_summary(session, ctx)
        );
    }
}

impl V2rayService {
    pub fn new(host: String, address: String, ws_path: String, is_tls: bool) -> Self {
        Self {
            host,
            address,
            ws_path,
            is_tls,
        }
    }

    fn check_ws_path(&self, path: &str) -> bool {
        path.starts_with(&self.ws_path)
    }

    fn check_host(&self, session: &mut Session) -> bool {
        // FIXME: Add error handling
        self.host == session.get_header("host").unwrap().to_str().unwrap()
    }
}
