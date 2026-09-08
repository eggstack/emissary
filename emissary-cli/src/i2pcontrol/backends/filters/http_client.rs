//! Bounded outbound HTTP proxy parsing and anonymity normalization.
//!
//! This filter deliberately has no socket or DNS capability.  It classifies a
//! request into an I2P destination or an explicitly configured I2P outproxy,
//! then serializes a new request with proxy identity removed.

use std::{
    collections::{HashMap, VecDeque},
    io,
    net::IpAddr,
    time::Duration,
};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub const REQUEST_LINE_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEADER_TIMEOUT: Duration = Duration::from_secs(15);
pub const BODY_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_REQUEST_LINE: usize = 8 * 1024;
pub const MAX_HEADER_LINE: usize = 8 * 1024;
pub const MAX_HEADER_COUNT: usize = 64;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;

/// M142 bounded HTTP-client residual configuration.
///
/// Pinned Java reference (`I2PTunnelHTTPClientBase` at
/// `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`):
/// `PROP_SSL_OUTPROXIES = "i2ptunnel.httpclient.SSLOutproxies"` is split with
/// `DataHelper.split(s, "[,; \\r\\n\\t]")`, returns `None` when unconfigured,
/// returns the single entry directly, otherwise selects randomly with a
/// hostname-scoped cache (`LHMCache(32)`) while avoiding the last failed
/// proxy. `PROP_JUMP_SERVERS = "i2ptunnel.httpclient.jumpServers"` is a
/// comma/space-separated list of `http://*.i2p/*` jump-service URL prefixes
/// rendered only on the destination-not-found path.
pub const MAX_SSL_RAW_LEN: usize = 2048;
pub const MAX_SSL_ENTRIES: usize = 8;
pub const MAX_SSL_ENTRY_LEN: usize = 255;
pub const MAX_JUMP_RAW_LEN: usize = 2048;
pub const MAX_JUMP_ENTRIES: usize = 8;
pub const MAX_JUMP_ENTRY_LEN: usize = 256;
pub const SSL_CACHE_CAPACITY: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutproxyTarget {
    pub destination: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpTarget {
    I2p {
        destination: String,
        port: u16,
    },
    Clearnet {
        host: String,
        port: u16,
        outproxy: OutproxyTarget,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
    pub allow_user_agent: bool,
    pub allow_referer: bool,
    pub allow_accept: bool,
    pub outproxy_authorization: Option<String>,
}

impl std::fmt::Debug for HttpClientPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientPolicy")
            .field("allow_user_agent", &self.allow_user_agent)
            .field("allow_referer", &self.allow_referer)
            .field("allow_accept", &self.allow_accept)
            .field(
                "outproxy_authorization",
                &self.outproxy_authorization.as_ref().map(|_| "***"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientRequest {
    pub method: String,
    pub path: String,
    pub host: String,
    pub authority: String,
    pub port: u16,
    pub content_length: usize,
    pub proxy_authorization: Option<String>,
    pub is_ssl: bool,
    headers: Vec<Header>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    name: String,
    value: String,
}

impl HttpClientRequest {
    pub fn parse(input: &[u8], outproxy: Option<OutproxyTarget>) -> io::Result<Self> {
        validate_header_block(input)?;
        let mut slots = [httparse::EMPTY_HEADER; MAX_HEADER_COUNT];
        let mut request = httparse::Request::new(&mut slots);
        let body_start = match request.parse(input).map_err(parse_error)? {
            httparse::Status::Complete(body_start) => body_start,
            httparse::Status::Partial => return Err(invalid("incomplete HTTP request")),
        };
        if body_start != input.len() {
            return Err(invalid("request body is not part of the header block"));
        }
        let method = request.method.ok_or_else(|| invalid("missing method"))?.to_ascii_uppercase();
        // M142: the HTTP client owns both plain-HTTP methods and CONNECT.
        // CONNECT (and absolute https://) is the reference SSL request class
        // routed via SSLProxies; all other methods stay on the ordinary
        // ProxyList path.
        if !matches!(
            method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" | "CONNECT"
        ) {
            return Err(invalid("HTTP method unsupported"));
        }
        let version = request.version.ok_or_else(|| invalid("missing HTTP version"))?;
        if version != 1 {
            return Err(invalid("HTTP version unsupported"));
        }
        let raw_target = request.path.ok_or_else(|| invalid("missing request target"))?;
        if raw_target == "*" || raw_target.starts_with("//") {
            return Err(invalid("request target unsupported"));
        }

        let headers = request
            .headers
            .iter()
            .map(|header| {
                let name = header.name.to_ascii_lowercase();
                if name.is_empty() || !name.bytes().all(is_token_byte) {
                    return Err(invalid("invalid header name"));
                }
                let value = std::str::from_utf8(header.value)
                    .map_err(|_| invalid("invalid header value"))?
                    .trim()
                    .to_owned();
                if value.chars().any(|character| character.is_control()) {
                    return Err(invalid("invalid header value"));
                }
                Ok(Header { name, value })
            })
            .collect::<io::Result<Vec<_>>>()?;

        let hosts = headers.iter().filter(|header| header.name == "host").collect::<Vec<_>>();
        if hosts.len() != 1 {
            return Err(invalid("exactly one Host header is required"));
        }
        let host_authority = hosts[0].value.clone();
        // CONNECT uses authority-form (host:port, no scheme). All other
        // methods use origin-form (Host supplies the authority) or
        // absolute-form (http:// or https://).
        if method == "CONNECT" {
            let (target_host, target_port) = parse_authority(raw_target)?;
            let target_port =
                target_port.ok_or_else(|| invalid("CONNECT target requires a port"))?;
            if target_port == 0 {
                return Err(invalid("CONNECT port is invalid"));
            }
            if !authority_matches(&host_authority, &target_host, target_port)? {
                return Err(invalid("Host does not match request target"));
            }
            let host = normalize_host(&target_host)?;
            // M142: SSL-class clearnet is allowed here even when the ordinary
            // ProxyList is absent; the handler enforces SSLProxies presence.
            // Ordinary-class clearnet still requires the ordinary outproxy now.
            if !is_i2p_destination(&host) && !is_connect_or_https(&method, None) {
                classify_target(&host, target_port, outproxy)?;
            } else {
                reject_local_target(&host)?;
                if is_i2p_destination(&host) {
                    let _ = classify_target(&host, target_port, outproxy.clone());
                }
            }
            let content_length = content_length(&headers)?;
            if content_length != 0 {
                return Err(invalid("CONNECT request body is unsupported"));
            }
            let proxy_authorization = headers
                .iter()
                .find(|header| header.name == "proxy-authorization")
                .map(|header| header.value.clone());
            return Ok(Self {
                method,
                path: String::new(),
                host,
                authority: host_authority,
                port: target_port,
                content_length,
                proxy_authorization,
                is_ssl: true,
                headers,
            });
        }
        let (target_host, target_port, path, absolute_scheme) = if raw_target.starts_with('/') {
            let (host, port) = parse_authority(&host_authority)?;
            (host, port.unwrap_or(80), raw_target.to_owned(), None)
        } else {
            let url =
                url::Url::parse(raw_target).map_err(|_| invalid("invalid absolute target"))?;
            let scheme = url.scheme().to_ascii_lowercase();
            if scheme != "http" && scheme != "https" {
                return Err(invalid("only HTTP proxy requests are supported"));
            }
            let host = url.host_str().ok_or_else(|| invalid("absolute target has no host"))?;
            let default_port = if scheme == "https" { 443 } else { 80 };
            let port = url.port().unwrap_or(default_port);
            let path = match url.query() {
                Some(query) => format!(
                    "{}?{query}",
                    if url.path().is_empty() {
                        "/"
                    } else {
                        url.path()
                    }
                ),
                None if url.path().is_empty() => "/".to_owned(),
                None => url.path().to_owned(),
            };
            (host.to_owned(), port, path, Some(scheme))
        };
        let is_ssl = is_connect_or_https(&method, absolute_scheme.as_deref());
        let default_port = if absolute_scheme.as_deref() == Some("https") {
            443
        } else {
            80
        };
        if absolute_scheme.is_some()
            && !authority_matches_with_default(
                &host_authority,
                &target_host,
                target_port,
                default_port,
            )?
        {
            return Err(invalid("Host does not match request target"));
        }
        let host = normalize_host(&target_host)?;
        if is_ssl {
            // SSL-class clearnet defers outproxy presence to the SSL selector;
            // I2P targets are always allowed. Ordinary-class clearnet still
            // requires the ordinary outproxy here.
            if !is_i2p_destination(&host) {
                reject_local_target(&host)?;
            } else {
                let _ = classify_target(&host, target_port, outproxy.clone());
            }
        } else {
            let _target = classify_target(&host, target_port, outproxy)?;
        }
        let content_length = content_length(&headers)?;
        let proxy_authorization = headers
            .iter()
            .find(|header| header.name == "proxy-authorization")
            .map(|header| header.value.clone());
        Ok(Self {
            method,
            path,
            host,
            authority: host_authority,
            port: target_port,
            content_length,
            proxy_authorization,
            is_ssl,
            headers,
        })
    }

    #[allow(dead_code)]
    pub fn target(&self, outproxy: Option<OutproxyTarget>) -> io::Result<HttpTarget> {
        classify_target(&self.host, self.port, outproxy)
    }

    pub fn is_connect(&self) -> bool {
        self.method == "CONNECT"
    }

    pub fn host_key(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn is_i2p(&self) -> bool {
        is_i2p_destination(&self.host)
    }

    pub fn serialize(
        &self,
        destination_host: &str,
        target: &HttpTarget,
        policy: &HttpClientPolicy,
    ) -> io::Result<Vec<u8>> {
        let host = match target {
            HttpTarget::I2p { .. } => destination_host.to_owned(),
            HttpTarget::Clearnet { host, port, .. } => authority_with_port(host, *port),
        };
        let mut output = Vec::with_capacity(MAX_HEADER_BYTES.min(4096));
        output.extend_from_slice(format!("{} {} HTTP/1.1\r\n", self.method, self.path).as_bytes());
        append_header(&mut output, "Host", &host);
        if policy.allow_user_agent {
            if let Some(value) = self.headers.iter().find(|header| header.name == "user-agent") {
                append_header(&mut output, "User-Agent", &value.value);
            }
        } else {
            append_header(&mut output, "User-Agent", "Emissary-I2P/1.0");
        }
        for header in &self.headers {
            if should_strip(&header.name)
                || header.name == "host"
                || header.name == "user-agent"
                || (header.name.starts_with("accept") && !policy.allow_accept)
            {
                continue;
            }
            if header.name == "referer"
                && (!policy.allow_referer || !referer_matches(&header.value, &self.host))
            {
                continue;
            }
            append_header(&mut output, &header.name, &header.value);
        }
        if let Some(authorization) = &policy.outproxy_authorization {
            if matches!(target, HttpTarget::Clearnet { .. }) {
                append_header(&mut output, "Proxy-Authorization", authorization);
            }
        }
        append_header(&mut output, "Connection", "close");
        output.extend_from_slice(b"\r\n");
        Ok(output)
    }
}

pub async fn read_header_block<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    tokio::time::timeout(REQUEST_LINE_TIMEOUT, reader.read_until(b'\n', &mut output))
        .await
        .map_err(|_| invalid("HTTP request line timeout"))??;
    if output.is_empty() || output.len() > MAX_REQUEST_LINE || !output.ends_with(b"\r\n") {
        return Err(invalid("invalid or overlong HTTP request line"));
    }
    tokio::time::timeout(HEADER_TIMEOUT, async {
        loop {
            if output.len() > MAX_HEADER_BYTES {
                return Err(invalid("HTTP headers too large"));
            }
            let start = output.len();
            let read = reader.read_until(b'\n', &mut output).await?;
            if read == 0 {
                return Err(invalid("incomplete HTTP headers"));
            }
            if output.len() - start > MAX_HEADER_LINE {
                return Err(invalid("HTTP header line too large"));
            }
            if output.ends_with(b"\r\n\r\n") {
                return Ok(output);
            }
            if !output[start..].ends_with(b"\r\n") {
                return Err(invalid("HTTP headers require CRLF"));
            }
        }
    })
    .await
    .map_err(|_| invalid("HTTP header timeout"))?
}

pub async fn copy_body<R, W>(reader: &mut R, writer: &mut W, length: usize) -> io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    if length == 0 {
        return Ok(());
    }
    let mut limited = reader.take(length as u64);
    let copied = tokio::time::timeout(BODY_TIMEOUT, tokio::io::copy(&mut limited, writer))
        .await
        .map_err(|_| invalid("HTTP body timeout"))??;
    if copied != length as u64 {
        return Err(invalid("short HTTP body"));
    }
    Ok(())
}

pub fn parse_authority(authority: &str) -> io::Result<(String, Option<u16>)> {
    let url = url::Url::parse(&format!("http://{authority}"))
        .map_err(|_| invalid("invalid authority"))?;
    let host = url.host_str().ok_or_else(|| invalid("authority has no host"))?;
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(invalid("authority contains a path"));
    }
    Ok((normalize_host(host)?, url.port()))
}

fn authority_matches(authority: &str, host: &str, port: u16) -> io::Result<bool> {
    authority_matches_with_default(authority, host, port, 80)
}

fn authority_matches_with_default(
    authority: &str,
    host: &str,
    port: u16,
    default_port: u16,
) -> io::Result<bool> {
    let (authority_host, authority_port) = parse_authority(authority)?;
    Ok(authority_host.eq_ignore_ascii_case(host) && authority_port.unwrap_or(default_port) == port)
}

fn is_connect_or_https(method: &str, scheme: Option<&str>) -> bool {
    method.eq_ignore_ascii_case("CONNECT")
        || scheme.is_some_and(|s| s.eq_ignore_ascii_case("https"))
}

pub fn is_i2p_destination(host: &str) -> bool {
    is_i2p_name(host) || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(host)
}

/// Plain `.i2p` hostnames (non-b32, non-full-destination) are the only class
/// that receives jump-server address-helper links, matching the pinned Java
/// `dnfh` branch. B32 and full destinations use the lease-set-not-found path
/// without jump links.
pub fn is_plain_i2p_hostname_for_jump(host: &str) -> bool {
    is_i2p_name(host)
        && !host.ends_with(".b32.i2p")
        && !crate::i2pcontrol::address_book_runtime::is_valid_full_destination(host)
}

/// Split a configured list on comma/semicolon/whitespace, matching the pinned
/// `DataHelper.split(s, "[,; \\r\\n\\t]")` shape. Empty tokens are dropped.
fn split_config_list(raw: &str) -> Vec<String> {
    raw.split([',', ';', ' ', '\t', '\r', '\n'])
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Parse and validate the Proposal `SSLProxies` string.
///
/// Each entry is an I2P outproxy authority (`host[:port]`, default 4444) that
/// must resolve through the accepted I2P destination/address-book path. No
/// entry may become a direct TCP/DNS clearnet target. Entries are lowercased,
/// deduplicated preserving first occurrence, and bounded before allocation.
/// The legacy `exit.storymcloud.i2p` typo is repaired to
/// `exit.stormycloud.i2p` exactly as the pinned reference does.
pub fn parse_ssl_proxy_list(raw: &str) -> io::Result<Vec<OutproxyTarget>> {
    if raw.len() > MAX_SSL_RAW_LEN {
        return Err(invalid("SSLProxies list too large"));
    }
    // Tab/CR/LF are valid reference separators (`[,; \\r\\n\\t]`); all other
    // control characters are rejected before allocation.
    if raw.chars().any(|c| c.is_control() && !matches!(c, '\t' | '\r' | '\n')) {
        return Err(invalid("SSLProxies contains control characters"));
    }
    let mut proxies = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for mut token in split_config_list(raw) {
        if token.len() > MAX_SSL_ENTRY_LEN {
            return Err(invalid("SSLProxies entry too large"));
        }
        // Pinned default-config typo repair.
        if token.eq_ignore_ascii_case("exit.storymcloud.i2p") {
            token = "exit.stormycloud.i2p".to_owned();
        }
        let (destination, port) =
            parse_authority(&token).map_err(|_| invalid("SSLProxies entry is invalid"))?;
        if !(destination.ends_with(".i2p")
            || destination.ends_with(".b32.i2p")
            || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(&destination))
        {
            return Err(invalid("SSLProxies entry must be an I2P destination"));
        }
        // Local/private literals can never be an I2P outproxy.
        reject_local_target(&destination)?;
        let port = port.unwrap_or(4444);
        if port == 0 {
            return Err(invalid("SSLProxies port is invalid"));
        }
        let key = format!("{destination}:{port}");
        if seen.insert(key) {
            proxies.push(OutproxyTarget { destination, port });
        }
        if proxies.len() > MAX_SSL_ENTRIES {
            return Err(invalid("too many SSLProxies entries"));
        }
    }
    Ok(proxies)
}

/// Parse and validate the Proposal `JumpList` string.
///
/// Each entry must be an `http://*.i2p/*` jump-service URL prefix. Only the
/// `http` scheme is accepted (matching the pinned `writeErrorMessage`
/// validation), the host must end in `.i2p`, and entries must not contain
/// control characters, CRLF, quotes, angle brackets, or whitespace that could
/// break out of the generated `href` attribute. Userinfo and fragments are
/// rejected. Hosts are lowercased; entries are deduplicated preserving first
/// occurrence and bounded before allocation.
pub fn parse_jump_server_list(raw: &str) -> io::Result<Vec<String>> {
    if raw.len() > MAX_JUMP_RAW_LEN {
        return Err(invalid("JumpList too large"));
    }
    // Tab/CR/LF are valid reference separators; all other control characters
    // are rejected before allocation.
    if raw.chars().any(|c| c.is_control() && !matches!(c, '\t' | '\r' | '\n')) {
        return Err(invalid("JumpList contains control characters"));
    }
    let mut servers = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for token in split_config_list(raw) {
        if token.len() > MAX_JUMP_ENTRY_LEN {
            return Err(invalid("JumpList entry too large"));
        }
        if token.bytes().any(|b| matches!(b, b'"' | b'\'' | b'<' | b'>' | b' ' | b'\t')) {
            return Err(invalid("JumpList entry is invalid"));
        }
        let url = url::Url::parse(&token).map_err(|_| invalid("JumpList entry is invalid"))?;
        if !url.scheme().eq_ignore_ascii_case("http") {
            return Err(invalid("JumpList entry must use http"));
        }
        if url.username() != "" || url.password().is_some() {
            return Err(invalid("JumpList entry must not carry userinfo"));
        }
        if url.fragment().is_some() {
            return Err(invalid("JumpList entry must not carry a fragment"));
        }
        let host = url.host_str().ok_or_else(|| invalid("JumpList entry has no host"))?;
        if host.len() > 255 {
            return Err(invalid("JumpList host too large"));
        }
        let host_lower = host.to_ascii_lowercase();
        if !host_lower.ends_with(".i2p") || host_lower.len() <= ".i2p".len() {
            return Err(invalid("JumpList host must be an I2P name"));
        }
        // Re-encode with a lowercased host so diagnostics and output are
        // deterministic; path/query are preserved byte-for-byte.
        let normalized = {
            let mut normalized = String::with_capacity(token.len());
            normalized.push_str("http://");
            normalized.push_str(&host_lower);
            if let Some(port) = url.port() {
                normalized.push(':');
                normalized.push_str(&port.to_string());
            }
            normalized.push_str(url.path());
            if let Some(query) = url.query() {
                normalized.push('?');
                normalized.push_str(query);
            }
            normalized
        };
        if normalized.len() > MAX_JUMP_ENTRY_LEN {
            return Err(invalid("JumpList entry too large"));
        }
        if seen.insert(normalized.clone()) {
            servers.push(normalized);
        }
        if servers.len() > MAX_JUMP_ENTRIES {
            return Err(invalid("too many JumpList entries"));
        }
    }
    Ok(servers)
}

/// Minimal HTML escaping for text and double-quoted attribute values.
pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(character),
        }
    }
    output
}

/// Render the bounded destination-not-found address-helper response.
///
/// Pure and side-effect-free: no fetch, no connect, no spawn. The caller
/// decides when this path applies (I2P resolution failure for a plain
/// `.i2p` hostname with a non-empty JumpList). The target host and every
/// jump prefix are HTML-escaped; jump entries were validated to `http://*.i2p`
/// at configuration time so no arbitrary scheme, header, or script can be
/// injected.
pub fn render_jump_helper_response(target_host: &str, jump_servers: &[String]) -> Vec<u8> {
    let escaped_target = escape_html(target_host);
    let mut body = String::new();
    body.push_str(
        "<html><body><h1>I2P ERROR: DESTINATION NOT FOUND</h1>\
         That I2P Destination was not found. Perhaps you pasted in the wrong \
         BASE64 I2P Destination or the link you are following is bad. \
         Could not find the following Destination:<br><br><div>",
    );
    body.push_str(&escaped_target);
    body.push_str("</div><br>\n<div id=\"jumplinks\">\n<h4>");
    body.push_str("Click a link below for an address helper from a jump service:");
    body.push_str("</h4>\n");
    for jump in jump_servers {
        let (jump_host, escaped_jump) = match url::Url::parse(jump) {
            Ok(url) => (
                url.host_str().unwrap_or("jump service").to_owned(),
                escape_html(jump),
            ),
            Err(_) => continue,
        };
        body.push_str("<a href=\"");
        body.push_str(&escaped_jump);
        body.push_str(&escaped_target);
        body.push_str("\">");
        body.push_str(&escape_html(&jump_host));
        body.push_str(" jump service</a><br>\n");
    }
    body.push_str("</div>\n</body></html>\n");
    let body_bytes = body.as_bytes();
    let mut response = Vec::with_capacity(body_bytes.len() + 128);
    response.extend_from_slice(
        format!(
            "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html; charset=utf-8\r\n\
             Cache-Control: no-cache\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            body_bytes.len()
        )
        .as_bytes(),
    );
    response.extend_from_slice(body_bytes);
    response
}

/// Bounded hostname-scoped SSL-outproxy selector.
///
/// Mirrors the pinned `selectSSLProxy`/`noteProxyResult` contract with a
/// bounded adaptation: the hostname cache is an LRU with
/// [`SSL_CACHE_CAPACITY`] entries (reference uses `LHMCache(32)`), failure
/// memory is exactly the most recently failed proxy, and a single configured
/// proxy is returned directly without cache interaction. Selection never
/// touches the network; the handler resolves the returned I2P destination
/// through the accepted address-book path only.
#[derive(Debug, Clone)]
pub struct SslProxySelector {
    proxies: Vec<OutproxyTarget>,
    cache: HashMap<String, OutproxyTarget>,
    order: VecDeque<String>,
    last_failed: Option<OutproxyTarget>,
}

impl SslProxySelector {
    pub fn new(proxies: Vec<OutproxyTarget>) -> Self {
        Self {
            proxies,
            cache: HashMap::new(),
            order: VecDeque::new(),
            last_failed: None,
        }
    }

    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    fn cache_insert(&mut self, key: String, proxy: OutproxyTarget) {
        if self.cache.contains_key(&key) {
            self.order.retain(|existing| existing != &key);
        } else if self.cache.len() >= SSL_CACHE_CAPACITY {
            if let Some(oldest) = self.order.pop_front() {
                self.cache.remove(&oldest);
            }
        }
        self.order.push_back(key.clone());
        self.cache.insert(key, proxy);
    }

    fn cache_remove(&mut self, key: &str) {
        if self.cache.remove(key).is_some() {
            self.order.retain(|existing| existing != key);
        }
    }

    /// Select the I2P outproxy for a clearnet `host:port` key.
    /// Returns `None` when no SSL proxy is configured.
    pub fn select(&mut self, host_key: &str) -> Option<OutproxyTarget> {
        if self.proxies.is_empty() {
            return None;
        }
        if self.proxies.len() == 1 {
            return Some(self.proxies[0].clone());
        }
        if let Some(cached) = self.cache.get(host_key) {
            if self.proxies.contains(cached) {
                // Refresh LRU recency on hit.
                let cached = cached.clone();
                self.cache_insert(host_key.to_owned(), cached.clone());
                return Some(cached);
            }
        }
        let candidates: Vec<OutproxyTarget> = match &self.last_failed {
            Some(failed) if self.proxies.contains(failed) => {
                self.proxies.iter().filter(|proxy| *proxy != failed).cloned().collect()
            }
            _ => self.proxies.clone(),
        };
        // Bounded adaptation preserves the reference invariant: with at least
        // two configured proxies, removing at most one failure always leaves
        // at least one candidate, so no empty-random-range path exists.
        if candidates.is_empty() {
            return self.proxies.first().cloned();
        }
        let index = {
            use rand::RngExt;
            rand::rng().random_range(0..candidates.len())
        };
        let selected = candidates[index].clone();
        self.cache_insert(host_key.to_owned(), selected.clone());
        Some(selected)
    }

    /// Record an authoritative selected-proxy attempt outcome.
    /// Success clears a matching last-failed entry and pins the mapping;
    /// failure stores the proxy as last-failed and drops a matching cache
    /// entry. Must only be called after a real resolve/connect attempt, and
    /// the caller must not hold this lock across network I/O.
    pub fn note_result(&mut self, proxy: &OutproxyTarget, host_key: &str, ok: bool) {
        if ok {
            if self.last_failed.as_ref() == Some(proxy) {
                self.last_failed = None;
            }
            self.cache_insert(host_key.to_owned(), proxy.clone());
        } else {
            self.last_failed = Some(proxy.clone());
            if self.cache.get(host_key) == Some(proxy) {
                self.cache_remove(host_key);
            }
        }
    }

    #[allow(dead_code)]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    #[allow(dead_code)]
    pub fn last_failed(&self) -> Option<OutproxyTarget> {
        self.last_failed.clone()
    }
}

pub(crate) fn classify_target(
    host: &str,
    port: u16,
    outproxy: Option<OutproxyTarget>,
) -> io::Result<HttpTarget> {
    reject_local_target(host)?;
    if is_i2p_name(host) || crate::i2pcontrol::address_book_runtime::is_valid_full_destination(host)
    {
        return Ok(HttpTarget::I2p {
            destination: host.to_owned(),
            port,
        });
    }
    let Some(outproxy) = outproxy else {
        return Err(invalid("clearnet target requires an I2P outproxy"));
    };
    Ok(HttpTarget::Clearnet {
        host: host.to_owned(),
        port,
        outproxy,
    })
}

fn normalize_host(host: &str) -> io::Result<String> {
    if host.is_empty() || host.len() > 255 || host.chars().any(char::is_control) {
        return Err(invalid("invalid host"));
    }
    Ok(host.trim_end_matches('.').to_ascii_lowercase())
}

fn reject_local_target(host: &str) -> io::Result<()> {
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return Err(invalid("local target forbidden"));
    }
    if let Ok(address) = host.parse::<IpAddr>() {
        let forbidden = match address {
            IpAddr::V4(address) => {
                address.is_loopback()
                    || address.is_private()
                    || address.is_link_local()
                    || address.is_unspecified()
            }
            IpAddr::V6(address) => {
                address.is_loopback()
                    || address.is_unique_local()
                    || address.is_unicast_link_local()
                    || address.is_unspecified()
            }
        };
        if forbidden {
            return Err(invalid("local target forbidden"));
        }
    }
    Ok(())
}

fn is_i2p_name(host: &str) -> bool {
    host.ends_with(".i2p") && host.len() > ".i2p".len()
}

fn content_length(headers: &[Header]) -> io::Result<usize> {
    let mut lengths = Vec::new();
    for header in headers {
        if header.name == "transfer-encoding" {
            return Err(invalid("Transfer-Encoding is unsupported"));
        }
        if header.name == "content-length" {
            let value =
                header.value.parse::<usize>().map_err(|_| invalid("invalid Content-Length"))?;
            lengths.push(value);
        }
    }
    if lengths.windows(2).any(|values| values[0] != values[1]) {
        return Err(invalid("conflicting Content-Length"));
    }
    Ok(lengths.first().copied().unwrap_or(0))
}

fn should_strip(name: &str) -> bool {
    name == "connection"
        || name == "keep-alive"
        || name == "proxy-authenticate"
        || name == "proxy-authorization"
        || name == "proxy-connection"
        || name == "te"
        || name == "trailer"
        || name == "transfer-encoding"
        || name == "upgrade"
        || name == "forwarded"
        || name == "via"
        || name == "from"
        || name.starts_with("x-forwarded-")
        || name.starts_with("proxy-")
        || name == "x-requested-with"
        || name == "dnt"
}

fn referer_matches(value: &str, target_host: &str) -> bool {
    url::Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.eq_ignore_ascii_case(target_host)))
        .unwrap_or(false)
}

fn authority_with_port(host: &str, port: u16) -> String {
    if port == 80 {
        host.to_owned()
    } else if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn append_header(output: &mut Vec<u8>, name: &str, value: &str) {
    output.extend_from_slice(name.as_bytes());
    output.extend_from_slice(b": ");
    output.extend_from_slice(value.as_bytes());
    output.extend_from_slice(b"\r\n");
}

fn validate_header_block(input: &[u8]) -> io::Result<()> {
    if input.len() > MAX_HEADER_BYTES || !input.ends_with(b"\r\n\r\n") {
        return Err(invalid("invalid HTTP header block"));
    }
    if input.iter().any(|byte| {
        *byte == 0 || (*byte < 0x20 && *byte != b'\r' && *byte != b'\n') || *byte == 0x7f
    }) {
        return Err(invalid("HTTP control character"));
    }
    for line in input.split(|byte| *byte == b'\n') {
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            return Err(invalid("obs-fold is unsupported"));
        }
    }
    Ok(())
}

fn is_token_byte(byte: u8) -> bool {
    matches!(byte, b'!'..=b'~') && !b"()<>@,;:\\\"/[]?={} \t".contains(&byte)
}

fn parse_error(_: httparse::Error) -> io::Error {
    invalid("malformed HTTP request")
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, outproxy: Option<OutproxyTarget>) -> HttpClientRequest {
        HttpClientRequest::parse(input.as_bytes(), outproxy).unwrap()
    }

    #[test]
    fn direct_i2p_request_is_normalized_without_proxy_identity() {
        let request = request(
            "GET http://alias.i2p/path HTTP/1.1\r\nHost: alias.i2p\r\nUser-Agent: browser\r\nReferer: https://clear.example/\r\nForwarded: for=1\r\nProxy-Authorization: Basic secret\r\n\r\n",
            None,
        );
        let target = request.target(None).unwrap();
        let output = request
            .serialize(
                "alias.i2p",
                &target,
                &HttpClientPolicy {
                    allow_user_agent: false,
                    allow_referer: false,
                    allow_accept: false,
                    outproxy_authorization: None,
                },
            )
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Host: alias.i2p\r\n"));
        assert!(output.contains("User-Agent: Emissary-I2P/1.0\r\n"));
        assert!(!output.contains("Forwarded"));
        assert!(!output.contains("Proxy-Authorization"));
        assert!(!output.contains("Referer"));
    }

    #[test]
    fn privacy_flags_forward_only_the_requested_headers() {
        let request = request(
            "GET http://alias.i2p/path HTTP/1.1\r\nHost: alias.i2p\r\nUser-Agent: browser\r\nReferer: http://alias.i2p/from\r\nAccept: text/plain\r\nAccept-Language: en\r\n\r\n",
            None,
        );
        let target = request.target(None).unwrap();
        assert_eq!(request.host, "alias.i2p");
        assert!(request
            .headers
            .iter()
            .any(|header| { header.name == "referer" && header.value == "http://alias.i2p/from" }));
        assert!(referer_matches("http://alias.i2p/from", "alias.i2p"));
        let output = String::from_utf8(
            request
                .serialize(
                    "alias.i2p",
                    &target,
                    &HttpClientPolicy {
                        allow_user_agent: true,
                        allow_referer: true,
                        allow_accept: true,
                        outproxy_authorization: None,
                    },
                )
                .unwrap(),
        )
        .unwrap();
        assert!(output.contains("User-Agent: browser\r\n"));
        assert!(
            output.contains("referer: http://alias.i2p/from\r\n"),
            "{output}"
        );
        assert!(output.contains("accept: text/plain\r\n"));
        assert!(output.contains("accept-language: en\r\n"));
        assert!(!output.contains("Proxy-Authorization"));
    }

    #[test]
    fn outproxy_authorization_is_destination_scoped() {
        let request = request(
            "GET http://alias.i2p/path HTTP/1.1\r\nHost: alias.i2p\r\n\r\n",
            None,
        );
        let target = request.target(None).unwrap();
        let output = String::from_utf8(
            request
                .serialize(
                    "alias.i2p",
                    &target,
                    &HttpClientPolicy {
                        allow_user_agent: false,
                        allow_referer: false,
                        allow_accept: false,
                        outproxy_authorization: Some("Basic should-not-forward".to_owned()),
                    },
                )
                .unwrap(),
        )
        .unwrap();
        assert!(!output.contains("Proxy-Authorization"));
    }

    #[test]
    fn clearnet_without_outproxy_is_rejected() {
        let request = HttpClientRequest::parse(
            b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\n\r\n",
            None,
        );
        assert!(request.is_err());
    }

    #[test]
    fn local_and_private_targets_are_rejected() {
        for host in ["localhost", "127.0.0.1", "192.168.1.1", "[::1]"] {
            let input = format!("GET http://{host}/ HTTP/1.1\r\nHost: {host}\r\n\r\n");
            assert!(
                HttpClientRequest::parse(input.as_bytes(), None).is_err(),
                "{host}"
            );
        }
    }

    #[test]
    fn malformed_framing_and_obs_fold_are_rejected() {
        assert!(HttpClientRequest::parse(
            b"POST / HTTP/1.1\r\nHost: x.i2p\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\n",
            None,
        )
        .is_err());
        assert!(HttpClientRequest::parse(
            b"GET / HTTP/1.1\r\nHost: x.i2p\r\n X-Leak: yes\r\n\r\n",
            None,
        )
        .is_err());
    }
}
