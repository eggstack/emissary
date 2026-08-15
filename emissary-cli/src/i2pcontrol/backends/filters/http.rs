//! Bounded HTTP/1 request normalization and response filtering for `httpserver`.
//!
//! This module intentionally implements only the HTTP boundary needed by the
//! accepted-stream server.  It never selects a local target and it never
//! forwards an input header block unchanged.

use std::{collections::HashSet, io, time::Duration};

use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

pub const REQUEST_LINE_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEADER_TIMEOUT: Duration = Duration::from_secs(15);
pub const BODY_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_REQUEST_LINE: usize = 8 * 1024;
pub const MAX_HEADER_LINE: usize = 8 * 1024;
pub const MAX_HEADER_COUNT: usize = 64;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;
pub const MAX_RESPONSE_LINE: usize = 8 * 1024;
pub const MAX_RESPONSE_HEADER_BYTES: usize = 32 * 1024;

// The largest destination representation accepted by the core reference
// parser is 391 serialized bytes (the key-certificate form), which is 524
// bytes in padded I2P Base64.  Keep the authenticated peer value below the
// old arbitrary 64 KiB allowance before it can become local HTTP metadata.
pub const MAX_TRUSTED_DESTINATION_TEXT: usize = 524;

const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

const PROXY_IDENTITY: &[&str] = &[
    "cf-connecting-ip",
    "fastly-client-ip",
    "forwarded",
    "via",
    "true-client-ip",
    "x-forwarded-for",
    "x-forwarded-host",
    "x-forwarded-server",
    "x-real-ip",
    "x-client-ip",
    "x-cluster-client-ip",
    "proxy",
    "proxy-connection",
];

const REQUEST_PRIVACY: &[&str] = &["priority", "sec-gpc"];

const I2P_IDENTITY: &[&str] = &[
    "x-i2p-desthash",
    "x-i2p-destb32",
    "x-i2p-destb64",
    "x-i2p-destination",
    "x-i2p-identity",
];

const RESPONSE_FINGERPRINTS: &[&str] = &[
    "age",
    "alt-svc",
    "date",
    "expires",
    "pragma",
    "referer",
    "server",
    "strict-transport-security",
    "via",
    "x-cache",
    "x-cache-hits",
    "x-cloud-trace-context",
    "x-contextid",
    "x-goog-generation",
    "x-goog-hash",
    "x-guploader-uploadid",
    "x-hacker",
    "x-nananana",
    "x-pantheon-styx-hostname",
    "x-powered-by",
    "x-runtime",
    "x-served-by",
    "x-styx-req-id",
    "proxy",
    "proxy-connection",
];

#[derive(Debug, Clone)]
pub struct HttpServerPolicy {
    pub website_host: String,
    pub block_access_in_proxies: bool,
    pub block_referers: bool,
    pub allow_referer: bool,
    pub block_user_agents: bool,
    pub allow_user_agent: bool,
    pub user_agents: Option<Vec<String>>,
    pub access_list: Option<Vec<String>>,
    pub access_option: AccessOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessOption {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedRequest {
    pub method: String,
    pub content_length: usize,
    pub head: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedResponse {
    pub content_length: Option<usize>,
    pub chunked: bool,
    pub head: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

pub async fn read_and_sanitize_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    peer_destination: &str,
    policy: &HttpServerPolicy,
) -> io::Result<SanitizedRequest> {
    validate_trusted_destination(peer_destination)?;
    let request_line =
        read_line_with_timeout(reader, REQUEST_LINE_TIMEOUT, MAX_REQUEST_LINE).await?;
    let (method, target, version) = parse_request_line(&request_line)?;
    let headers = tokio::time::timeout(HEADER_TIMEOUT, read_headers(reader, MAX_HEADER_BYTES))
        .await
        .map_err(|_| invalid(io::ErrorKind::TimedOut, "HTTP header timeout"))??;

    let host_count = headers.iter().filter(|header| eq(&header.name, "host")).count();
    if host_count > 1 {
        return Err(invalid(io::ErrorKind::InvalidData, "ambiguous host header"));
    }

    let proxy_seen = headers
        .iter()
        .any(|header| PROXY_IDENTITY.contains(&header.name.to_ascii_lowercase().as_str()));
    if policy.block_access_in_proxies && proxy_seen {
        return Err(invalid(
            io::ErrorKind::PermissionDenied,
            "proxy access blocked",
        ));
    }

    if !peer_allowed(peer_destination, policy) {
        return Err(invalid(io::ErrorKind::PermissionDenied, "access denied"));
    }

    let referer_seen = headers.iter().any(|header| eq(&header.name, "referer"));
    if (policy.block_referers || !policy.allow_referer) && referer_seen {
        return Err(invalid(io::ErrorKind::PermissionDenied, "referer blocked"));
    }

    let user_agent = headers
        .iter()
        .find(|header| eq(&header.name, "user-agent"))
        .map(|header| header.value.trim());
    if (policy.block_user_agents || !policy.allow_user_agent) && user_agent.is_some() {
        return Err(invalid(
            io::ErrorKind::PermissionDenied,
            "user-agent blocked",
        ));
    }
    if let Some(allowed) = &policy.user_agents {
        if user_agent.is_none_or(|value| !allowed.iter().any(|entry| value == entry)) {
            return Err(invalid(
                io::ErrorKind::PermissionDenied,
                "user-agent not allowed",
            ));
        }
    }

    if headers.iter().any(|header| {
        eq(&header.name, "connection")
            && split_tokens(&header.value).iter().any(|token| token == "upgrade")
    }) || headers.iter().any(|header| eq(&header.name, "upgrade"))
    {
        return Err(invalid(
            io::ErrorKind::Unsupported,
            "HTTP upgrade is unsupported",
        ));
    }

    let content_lengths = headers
        .iter()
        .filter(|header| eq(&header.name, "content-length"))
        .map(|header| parse_content_length(&header.value))
        .collect::<io::Result<Vec<_>>>()?;
    if content_lengths.windows(2).any(|values| values[0] != values[1]) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "conflicting content length",
        ));
    }
    let content_length = content_lengths.first().copied().unwrap_or(0);

    if headers.iter().any(|header| eq(&header.name, "transfer-encoding")) {
        return Err(invalid(
            io::ErrorKind::Unsupported,
            "transfer encoding is unsupported",
        ));
    }

    let mut output = Vec::with_capacity(request_line.len() + headers.len() * 32 + 4);
    output.extend_from_slice(&normalize_request_line(&method, &target, &version)?);
    let nominated = connection_nominated_headers(&headers);
    for header in headers {
        let name = header.name.to_ascii_lowercase();
        if HOP_BY_HOP.contains(&name.as_str())
            || nominated.contains(name.as_str())
            || PROXY_IDENTITY.contains(&name.as_str())
            || REQUEST_PRIVACY.contains(&name.as_str())
            || I2P_IDENTITY.contains(&name.as_str())
            || (eq(&name, "referer") && (policy.block_referers || !policy.allow_referer))
            || (eq(&name, "user-agent") && (policy.block_user_agents || !policy.allow_user_agent))
            || eq(&name, "host")
        {
            continue;
        }
        append_header(&mut output, &header.name, &header.value);
    }
    append_header(&mut output, "Host", &policy.website_host);
    append_header(&mut output, "X-I2P-DestB64", peer_destination);
    append_header(
        &mut output,
        "X-I2P-DestB32",
        &crate::i2pcontrol::address_book_runtime::base32_for_destination(peer_destination),
    );
    output.extend_from_slice(b"Connection: close\r\n\r\n");

    Ok(SanitizedRequest {
        method,
        content_length,
        head: output,
    })
}

pub async fn read_and_filter_response<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> io::Result<SanitizedResponse> {
    let status_line = read_line_with_timeout(reader, HEADER_TIMEOUT, MAX_RESPONSE_LINE).await?;
    let (version, status, reason) = parse_status_line(&status_line)?;
    let headers = tokio::time::timeout(
        HEADER_TIMEOUT,
        read_headers(reader, MAX_RESPONSE_HEADER_BYTES),
    )
    .await
    .map_err(|_| invalid(io::ErrorKind::TimedOut, "HTTP response header timeout"))??;

    let content_lengths = headers
        .iter()
        .filter(|header| eq(&header.name, "content-length"))
        .map(|header| parse_content_length(&header.value))
        .collect::<io::Result<Vec<_>>>()?;
    if content_lengths.windows(2).any(|values| values[0] != values[1]) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "conflicting response length",
        ));
    }
    let transfer_encoding = headers
        .iter()
        .find(|header| eq(&header.name, "transfer-encoding"))
        .map(|header| header.value.trim().to_ascii_lowercase());
    let chunked = match transfer_encoding.as_deref() {
        None => false,
        Some("chunked") if content_lengths.is_empty() => true,
        Some(_) => {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "ambiguous response framing",
            ));
        }
    };

    let mut output = Vec::with_capacity(status_line.len() + headers.len() * 32 + 4);
    output.extend_from_slice(format!("{version} {status} {reason}\r\n").as_bytes());
    let nominated = connection_nominated_headers(&headers);
    for header in headers {
        let name = header.name.to_ascii_lowercase();
        if eq(&name, "content-length") || eq(&name, "transfer-encoding") {
            // Re-emit only the validated framing selected above.  In
            // particular, removing Transfer-Encoding while forwarding raw
            // chunk bytes would change the response body interpretation.
            continue;
        }
        if RESPONSE_FINGERPRINTS.contains(&name.as_str())
            || HOP_BY_HOP.contains(&name.as_str())
            || nominated.contains(name.as_str())
        {
            continue;
        }
        append_header(&mut output, &header.name, &header.value);
    }
    if chunked {
        append_header(&mut output, "Transfer-Encoding", "chunked");
    } else if let Some(length) = content_lengths.first() {
        append_header(&mut output, "Content-Length", &length.to_string());
    }
    output.extend_from_slice(b"Connection: close\r\n\r\n");
    Ok(SanitizedResponse {
        content_length: content_lengths.first().copied(),
        chunked,
        head: output,
    })
}

pub async fn copy_body<R, W>(reader: &mut R, writer: &mut W, length: usize) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    if length == 0 {
        return Ok(());
    }
    let mut limited = reader.take(length as u64);
    let copied = tokio::time::timeout(BODY_TIMEOUT, tokio::io::copy(&mut limited, writer))
        .await
        .map_err(|_| invalid(io::ErrorKind::TimedOut, "HTTP body timeout"))??;
    if copied != length as u64 {
        return Err(invalid(io::ErrorKind::UnexpectedEof, "short HTTP body"));
    }
    Ok(())
}

pub async fn copy_response_body<R, W>(
    reader: &mut R,
    writer: &mut W,
    response: &SanitizedResponse,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    if response.chunked || response.content_length.is_none() {
        tokio::time::timeout(BODY_TIMEOUT, tokio::io::copy(reader, writer))
            .await
            .map_err(|_| invalid(io::ErrorKind::TimedOut, "HTTP response body timeout"))??;
    } else if let Some(length) = response.content_length {
        copy_body(reader, writer, length).await?;
    }
    writer.flush().await
}

fn peer_allowed(peer: &str, policy: &HttpServerPolicy) -> bool {
    let Some(entries) = &policy.access_list else {
        return true;
    };
    let b32 = crate::i2pcontrol::address_book_runtime::base32_for_destination(peer);
    let matches = entries.iter().any(|entry| entry == peer || entry.eq_ignore_ascii_case(&b32));
    match policy.access_option {
        AccessOption::Allow => matches,
        AccessOption::Deny => !matches,
    }
}

fn validate_trusted_destination(destination: &str) -> io::Result<()> {
    if destination.is_empty()
        || destination.len() > MAX_TRUSTED_DESTINATION_TEXT
        || destination
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "trusted peer identity is too large or malformed",
        ));
    }
    Ok(())
}

fn parse_request_line(line: &[u8]) -> io::Result<(String, String, String)> {
    let line = trim_crlf(line)?;
    if line.iter().any(|byte| *byte < 0x20 || *byte == 0x7f) {
        return Err(invalid(io::ErrorKind::InvalidData, "invalid request line"));
    }
    let mut parts = line.split(|byte| *byte == b' ');
    let method = nonempty_utf8(parts.next(), "invalid request method")?;
    let target = nonempty_utf8(parts.next(), "invalid request target")?;
    let version = nonempty_utf8(parts.next(), "invalid HTTP version")?;
    if parts.next().is_some() || !version.starts_with("HTTP/") || target.starts_with("//") {
        return Err(invalid(io::ErrorKind::InvalidData, "invalid request line"));
    }
    if !method.bytes().all(is_token_byte) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "invalid request method",
        ));
    }
    let target = normalize_target(&target)?;
    Ok((method, target, version))
}

fn normalize_request_line(method: &str, target: &str, version: &str) -> io::Result<Vec<u8>> {
    if version != "HTTP/1.0" && version != "HTTP/1.1" {
        return Err(invalid(
            io::ErrorKind::Unsupported,
            "HTTP version unsupported",
        ));
    }
    let line = format!("{method} {target} {version}\r\n");
    if line.len() > MAX_REQUEST_LINE {
        return Err(invalid(io::ErrorKind::InvalidData, "request line too long"));
    }
    Ok(line.into_bytes())
}

fn normalize_target(target: &str) -> io::Result<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        let parsed = url::Url::parse(target)
            .map_err(|_| invalid(io::ErrorKind::InvalidData, "invalid absolute target"))?;
        if parsed.host_str().is_none() || parsed.fragment().is_some() {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "invalid absolute target",
            ));
        }
        let mut path = parsed.path().to_owned();
        if path.is_empty() {
            path.push('/');
        }
        if let Some(query) = parsed.query() {
            path.push('?');
            path.push_str(query);
        }
        return Ok(path);
    }
    if !target.starts_with('/') || target.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "invalid request target",
        ));
    }
    Ok(target.to_owned())
}

fn parse_status_line(line: &[u8]) -> io::Result<(String, u16, String)> {
    let line = trim_crlf(line)?;
    let mut parts = line.splitn(3, |byte| *byte == b' ');
    let version = nonempty_utf8(parts.next(), "invalid response version")?;
    let status = nonempty_utf8(parts.next(), "invalid response status")?
        .parse::<u16>()
        .map_err(|_| invalid(io::ErrorKind::InvalidData, "invalid response status"))?;
    let reason = parts.next().unwrap_or_default();
    if version != "HTTP/1.0" && version != "HTTP/1.1" || !(100..=599).contains(&status) {
        return Err(invalid(io::ErrorKind::InvalidData, "invalid response line"));
    }
    if reason.iter().any(|byte| *byte < 0x20 || *byte == 0x7f) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "invalid response reason",
        ));
    }
    Ok((
        version,
        status,
        String::from_utf8_lossy(reason).into_owned(),
    ))
}

async fn read_line_with_timeout<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    timeout: Duration,
    max: usize,
) -> io::Result<Vec<u8>> {
    let mut line = Vec::new();
    tokio::time::timeout(timeout, reader.read_until(b'\n', &mut line))
        .await
        .map_err(|_| invalid(io::ErrorKind::TimedOut, "HTTP line timeout"))??;
    if line.is_empty() {
        return Err(invalid(io::ErrorKind::UnexpectedEof, "HTTP line missing"));
    }
    if line.len() > max {
        return Err(invalid(io::ErrorKind::InvalidData, "HTTP line too long"));
    }
    Ok(line)
}

async fn read_headers<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max_bytes: usize,
) -> io::Result<Vec<Header>> {
    let mut headers = Vec::new();
    let mut total = 0usize;
    loop {
        let mut line = Vec::new();
        reader.read_until(b'\n', &mut line).await?;
        total = total.saturating_add(line.len());
        if total > max_bytes || line.len() > MAX_HEADER_LINE {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "HTTP headers too large",
            ));
        }
        if line == b"\r\n" {
            return Ok(headers);
        }
        if line.is_empty() || !line.ends_with(b"\r\n") {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "HTTP header is not CRLF terminated",
            ));
        }
        if line[0] == b' ' || line[0] == b'\t' {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "HTTP obs-fold is unsupported",
            ));
        }
        let line = &line[..line.len() - 2];
        let Some(colon) = line.iter().position(|byte| *byte == b':') else {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "HTTP header has no colon",
            ));
        };
        if colon == 0 || !line[..colon].iter().all(|byte| is_token_byte(*byte)) {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "invalid HTTP header name",
            ));
        }
        if line[colon + 1..].iter().any(|byte| {
            *byte == b'\r' || *byte == b'\n' || *byte == 0 || *byte < 0x20 && *byte != b'\t'
        }) {
            return Err(invalid(
                io::ErrorKind::InvalidData,
                "invalid HTTP header value",
            ));
        }
        if headers.len() >= MAX_HEADER_COUNT {
            return Err(invalid(io::ErrorKind::InvalidData, "too many HTTP headers"));
        }
        headers.push(Header {
            name: String::from_utf8(line[..colon].to_vec())
                .map_err(|_| invalid(io::ErrorKind::InvalidData, "invalid HTTP header name"))?,
            value: String::from_utf8(line[colon + 1..].to_vec())
                .map_err(|_| invalid(io::ErrorKind::InvalidData, "invalid HTTP header value"))?
                .trim()
                .to_owned(),
        });
    }
}

fn connection_nominated_headers(headers: &[Header]) -> HashSet<String> {
    headers
        .iter()
        .filter(|header| eq(&header.name, "connection"))
        .flat_map(|header| split_tokens(&header.value))
        .collect()
}

fn split_tokens(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn parse_content_length(value: &str) -> io::Result<usize> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid(
            io::ErrorKind::InvalidData,
            "invalid content length",
        ));
    }
    value
        .parse::<usize>()
        .map_err(|_| invalid(io::ErrorKind::InvalidData, "content length overflow"))
}

fn append_header(output: &mut Vec<u8>, name: &str, value: &str) {
    output.extend_from_slice(name.as_bytes());
    output.extend_from_slice(b": ");
    output.extend_from_slice(value.as_bytes());
    output.extend_from_slice(b"\r\n");
}

fn trim_crlf(line: &[u8]) -> io::Result<&[u8]> {
    line.strip_suffix(b"\r\n").ok_or_else(|| {
        invalid(
            io::ErrorKind::InvalidData,
            "HTTP line is not CRLF terminated",
        )
    })
}

fn nonempty_utf8(value: Option<&[u8]>, message: &'static str) -> io::Result<String> {
    let value = value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid(io::ErrorKind::InvalidData, message))?;
    String::from_utf8(value.to_vec()).map_err(|_| invalid(io::ErrorKind::InvalidData, message))
}

fn is_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'#'..=b'\'' | b'*' | b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z'
            | b'^' | b'_' | b'`' | b'a'..=b'z' | b'|' | b'~'
    )
}

fn eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn invalid(kind: io::ErrorKind, message: &'static str) -> io::Error {
    io::Error::new(kind, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncWriteExt, BufReader};

    fn policy() -> HttpServerPolicy {
        HttpServerPolicy {
            website_host: "example.i2p".to_owned(),
            block_access_in_proxies: false,
            block_referers: false,
            allow_referer: true,
            block_user_agents: false,
            allow_user_agent: true,
            user_agents: None,
            access_list: None,
            access_option: AccessOption::Allow,
        }
    }

    async fn request(input: &[u8], policy: &HttpServerPolicy) -> io::Result<SanitizedRequest> {
        let (mut writer, reader) = duplex(64 * 1024);
        writer.write_all(input).await.unwrap();
        writer.shutdown().await.unwrap();
        let mut reader = BufReader::new(reader);
        read_and_sanitize_request(&mut reader, "peer-destination", policy).await
    }

    #[tokio::test]
    async fn normalizes_absolute_target_and_removes_spoofed_identity() {
        let result = request(
            b"GET http://evil.example/path?q=1 HTTP/1.1\r\nHost: evil.example\r\nx-i2p-destb64: attacker\r\nX-Forwarded-For: 10.0.0.1\r\n\r\n",
            &policy(),
        )
        .await
        .unwrap();
        let output = String::from_utf8(result.head).unwrap();
        assert!(output.starts_with("GET /path?q=1 HTTP/1.1\r\n"));
        assert!(!output.contains("evil.example"));
        assert!(!output.contains("attacker"));
        assert!(!output.contains("10.0.0.1"));
        assert!(output.contains("Host: example.i2p\r\n"));
        assert!(output.contains("X-I2P-DestB64: peer-destination\r\n"));
    }

    #[tokio::test]
    async fn rejects_smuggling_and_malformed_headers() {
        for input in [
            b"POST / HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\n".as_slice(),
            b"POST / HTTP/1.1\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\n"
                .as_slice(),
            b"GET / HTTP/1.1\r\n Folded: yes\r\n\r\n".as_slice(),
            b"GET / HTTP/1.1\r\nMissing\r\n\r\n".as_slice(),
        ] {
            assert!(request(input, &policy()).await.is_err());
        }
    }

    #[tokio::test]
    async fn applies_proxy_referer_user_agent_and_access_policy() {
        let mut configured = policy();
        configured.block_access_in_proxies = true;
        assert!(request(
            b"GET / HTTP/1.1\r\nForwarded: for=10.0.0.1\r\n\r\n",
            &configured,
        )
        .await
        .is_err());

        configured = policy();
        configured.block_referers = true;
        assert!(request(
            b"GET / HTTP/1.1\r\nReferer: https://clear.example/\r\n\r\n",
            &configured
        )
        .await
        .is_err());

        configured = policy();
        configured.user_agents = Some(vec!["safe-agent".to_owned()]);
        assert!(
            request(b"GET / HTTP/1.1\r\nUser-Agent: unsafe\r\n\r\n", &configured)
                .await
                .is_err()
        );

        configured = policy();
        configured.access_list = Some(vec!["other-peer".to_owned()]);
        assert!(request(b"GET / HTTP/1.1\r\n\r\n", &configured).await.is_err());
    }

    #[tokio::test]
    async fn preserves_allowed_end_to_end_headers() {
        let result = request(
            b"GET / HTTP/1.1\r\nReferer: https://example.i2p/\r\nUser-Agent: safe\r\n\r\n",
            &policy(),
        )
        .await
        .unwrap();
        let output = String::from_utf8(result.head).unwrap();
        assert!(output.contains("Referer: https://example.i2p/\r\n"));
        assert!(output.contains("User-Agent: safe\r\n"));
    }

    #[tokio::test]
    async fn response_filter_removes_fingerprints_and_hop_by_hop_headers() {
        let (mut writer, reader) = duplex(64 * 1024);
        writer
            .write_all(b"HTTP/1.1 200 OK\r\nDaTe: clock\r\nServer: secret\r\nX-Powered-By: framework\r\nContent-Type: text/plain\r\nSet-Cookie: safe=1\r\nContent-Length: 5\r\n\r\nhello")
            .await
            .unwrap();
        writer.shutdown().await.unwrap();
        let mut reader = BufReader::new(reader);
        let response = read_and_filter_response(&mut reader).await.unwrap();
        let output = String::from_utf8(response.head).unwrap();
        assert!(!output.contains("clock"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("framework"));
        assert!(output.contains("Content-Type: text/plain\r\n"));
        assert!(output.contains("Set-Cookie: safe=1\r\n"));
        assert!(output.contains("Content-Length: 5\r\n"));
        assert_eq!(response.content_length, Some(5));
    }

    #[tokio::test]
    async fn removes_every_adopted_response_fingerprint_case_insensitively() {
        let mut input = b"HTTP/1.1 200 OK\r\n".to_vec();
        for (index, name) in RESPONSE_FINGERPRINTS.iter().enumerate() {
            input.extend_from_slice(
                format!("{}: secret-{index}\r\n", name.to_ascii_uppercase()).as_bytes(),
            );
        }
        input.extend_from_slice(b"Content-Length: 0\r\nContent-Type: text/plain\r\n\r\n");

        let (mut writer, reader) = duplex(64 * 1024);
        writer.write_all(&input).await.unwrap();
        writer.shutdown().await.unwrap();
        let mut reader = BufReader::new(reader);
        let response = read_and_filter_response(&mut reader).await.unwrap();
        let output = String::from_utf8(response.head).unwrap();
        for index in 0..RESPONSE_FINGERPRINTS.len() {
            assert!(!output.contains(&format!("secret-{index}")));
        }
        assert!(output.contains("Content-Length: 0\r\n"));
        assert!(output.contains("Content-Type: text/plain\r\n"));
    }

    #[tokio::test]
    async fn removes_proxy_identity_and_adopted_request_privacy_headers() {
        let result = request(
            b"GET / HTTP/1.1\r\nForwarded: for=10.0.0.1\r\nVia: proxy\r\nX-Forwarded-For: 10.0.0.2\r\nX-Forwarded-Host: evil\r\nX-Forwarded-Server: evil\r\nProxy: evil\r\nX-Real-IP: 10.0.0.3\r\nx-client-ip: 10.0.0.4\r\nTrue-Client-IP: 10.0.0.5\r\nCF-Connecting-IP: 10.0.0.6\r\nFastly-Client-IP: 10.0.0.7\r\nX-Cluster-Client-IP: 10.0.0.8\r\nPriority: u=1\r\nSec-GPC: 1\r\n\r\n",
            &policy(),
        )
        .await
        .unwrap();
        let output = String::from_utf8(result.head).unwrap().to_ascii_lowercase();
        for marker in [
            "x-real-ip",
            "x-client-ip",
            "true-client-ip",
            "cf-connecting-ip",
            "fastly-client-ip",
            "x-cluster-client-ip",
            "x-forwarded-for",
            "x-forwarded-host",
            "x-forwarded-server",
            "forwarded",
            "via",
            "proxy",
            "priority",
            "sec-gpc",
        ] {
            assert!(
                !output.contains(marker),
                "unexpected forwarded header: {marker}"
            );
        }
    }

    #[tokio::test]
    async fn preserves_valid_chunked_response_framing() {
        let (mut writer, reader) = duplex(64 * 1024);
        writer
            .write_all(
                b"HTTP/1.1 200 OK\r\nSeRvEr: secret\r\nTransfer-Encoding: ChUnKeD\r\n\r\n5\r\nhello\r\n0\r\n\r\n",
            )
            .await
            .unwrap();
        writer.shutdown().await.unwrap();
        let mut reader = BufReader::new(reader);
        let response = read_and_filter_response(&mut reader).await.unwrap();
        let output = String::from_utf8(response.head).unwrap();
        assert!(!output.contains("secret"));
        assert!(output.contains("Transfer-Encoding: chunked\r\n"));
        assert!(response.chunked);
        let mut body = Vec::new();
        reader.read_to_end(&mut body).await.unwrap();
        assert_eq!(body, b"5\r\nhello\r\n0\r\n\r\n");
    }

    #[tokio::test]
    async fn rejects_over_bound_trusted_identity_before_building_request() {
        let (mut writer, reader) = duplex(64 * 1024);
        writer.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
        writer.shutdown().await.unwrap();
        let mut reader = BufReader::new(reader);
        let identity = "a".repeat(MAX_TRUSTED_DESTINATION_TEXT + 1);
        let error = read_and_sanitize_request(&mut reader, &identity, &policy()).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
