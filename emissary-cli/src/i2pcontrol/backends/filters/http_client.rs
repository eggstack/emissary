//! Bounded outbound HTTP proxy parsing and anonymity normalization.
//!
//! This filter deliberately has no socket or DNS capability.  It classifies a
//! request into an I2P destination or an explicitly configured I2P outproxy,
//! then serializes a new request with proxy identity removed.

use std::{io, net::IpAddr, time::Duration};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub const REQUEST_LINE_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEADER_TIMEOUT: Duration = Duration::from_secs(15);
pub const BODY_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_REQUEST_LINE: usize = 8 * 1024;
pub const MAX_HEADER_LINE: usize = 8 * 1024;
pub const MAX_HEADER_COUNT: usize = 64;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
    pub allow_user_agent: bool,
    pub allow_referer: bool,
    pub allow_accept: bool,
    pub outproxy_authorization: Option<String>,
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
        if !matches!(
            method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
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
        let (target_host, target_port, path, absolute_scheme) = if raw_target.starts_with('/') {
            let (host, port) = parse_authority(&host_authority)?;
            (host, port.unwrap_or(80), raw_target.to_owned(), None)
        } else {
            let url =
                url::Url::parse(raw_target).map_err(|_| invalid("invalid absolute target"))?;
            let scheme = url.scheme().to_ascii_lowercase();
            if scheme != "http" {
                return Err(invalid("only HTTP proxy requests are supported"));
            }
            let host = url.host_str().ok_or_else(|| invalid("absolute target has no host"))?;
            let port = url.port().unwrap_or(80);
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
        if absolute_scheme.is_some()
            && !authority_matches(&host_authority, &target_host, target_port)?
        {
            return Err(invalid("Host does not match request target"));
        }
        let host = normalize_host(&target_host)?;
        let _target = classify_target(&host, target_port, outproxy)?;
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
            headers,
        })
    }

    pub fn target(&self, outproxy: Option<OutproxyTarget>) -> io::Result<HttpTarget> {
        classify_target(&self.host, self.port, outproxy)
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
    let (authority_host, authority_port) = parse_authority(authority)?;
    Ok(authority_host.eq_ignore_ascii_case(host) && authority_port.unwrap_or(80) == port)
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
            IpAddr::V4(address) =>
                address.is_loopback()
                    || address.is_private()
                    || address.is_link_local()
                    || address.is_unspecified(),
            IpAddr::V6(address) =>
                address.is_loopback()
                    || address.is_unique_local()
                    || address.is_unicast_link_local()
                    || address.is_unspecified(),
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
