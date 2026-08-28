//! Bounded, request-independent RouterInfo news source.
//!
//! The reference RouterInfo implementation exposes the HTML rendered by
//! `NewsFeedHelper` from the signed I2P `news.su3` feed. This module owns the
//! optional I2PControl fetch/cache boundary; RouterInfo only reads its
//! immutable current snapshot.

#![allow(dead_code)]

use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use emissary_util::su3::Su3;

use super::router_info::InspectionError;

const LOG_TARGET: &str = "emissary::i2pcontrol::news";

/// The pinned Java router's default news source, reached through the local
/// I2P HTTP proxy rather than through a clearnet client.
pub(crate) const DEFAULT_NEWS_URL: &str =
    "http://tc73n4kivdroccekirco7rhgxdg5f3cjvbaapabupeyzrqwv5guq.b32.i2p/news.su3";
const MAX_COMPRESSED_BYTES: usize = 2 * 1024 * 1024;
const MAX_RENDERED_BYTES: usize = 1024 * 1024;
const MAX_ENTRIES: usize = 128;
const MAX_XML_DEPTH: usize = 32;
const MAX_XML_NODES: usize = 4096;
const MAX_FIELD_BYTES: usize = 64 * 1024;
const REFRESH_INTERVAL: Duration = Duration::from_secs(36 * 60 * 60);
const RETRY_INTERVAL: Duration = Duration::from_secs(15 * 60);
const MAX_STALENESS: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NewsSnapshotError {
    Unavailable,
    Stale,
}

impl NewsSnapshotError {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Unavailable => "no validated router news generation",
            Self::Stale => "router news generation is stale",
        }
    }
}

struct NewsState {
    current: Option<(String, Instant)>,
}

/// One bounded news source and its background refresh owner.
pub(crate) struct RouterNewsSource {
    state: Arc<RwLock<NewsState>>,
    abort: tokio::task::AbortHandle,
}

impl RouterNewsSource {
    /// Start one source. The caller must provide the already-configured local
    /// HTTP proxy; no direct DNS/clearnet path is created here.
    pub(crate) fn start(proxy_host: &str, proxy_port: u16) -> Result<Arc<Self>, String> {
        if proxy_host.is_empty() || proxy_port == 0 {
            return Err("router news requires a configured HTTP proxy".to_owned());
        }

        let proxy = reqwest::Proxy::http(format!("http://{proxy_host}:{proxy_port}"))
            .map_err(|_| "router news proxy configuration is invalid".to_owned())?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| "router news client could not be initialized".to_owned())?;

        let state = Arc::new(RwLock::new(NewsState { current: None }));
        let task_state = Arc::clone(&state);
        let task = tokio::spawn(async move {
            refresh_loop(client, task_state).await;
        });

        Ok(Arc::new(Self {
            state,
            abort: task.abort_handle(),
        }))
    }

    /// Return the last completely validated generation without performing I/O.
    pub(crate) fn snapshot(&self) -> Result<String, NewsSnapshotError> {
        let state = self.state.read().map_err(|_| NewsSnapshotError::Unavailable)?;
        snapshot_state(&state)
    }
}

fn snapshot_state(state: &NewsState) -> Result<String, NewsSnapshotError> {
    let Some((news, published_at)) = &state.current else {
        return Err(NewsSnapshotError::Unavailable);
    };
    if published_at.elapsed() > MAX_STALENESS {
        return Err(NewsSnapshotError::Stale);
    }
    Ok(news.clone())
}

impl Drop for RouterNewsSource {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

async fn refresh_loop(client: reqwest::Client, state: Arc<RwLock<NewsState>>) {
    let mut delay = Duration::ZERO;
    loop {
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        match fetch_and_render(&client).await {
            Ok(news) => {
                if let Ok(mut state) = state.write() {
                    state.current = Some((news, Instant::now()));
                }
                delay = REFRESH_INTERVAL;
            }
            Err(error) => {
                tracing::warn!(target: LOG_TARGET, error = %error, "router news refresh failed");
                delay = RETRY_INTERVAL;
            }
        }
    }
}

async fn fetch_and_render(client: &reqwest::Client) -> Result<String, String> {
    let response = client
        .get(DEFAULT_NEWS_URL)
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()
        .await
        .map_err(|_| "news source request failed".to_owned())?;
    if !response.status().is_success() {
        return Err("news source returned an unsuccessful response".to_owned());
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_COMPRESSED_BYTES as u64)
    {
        return Err("news source response exceeds its size bound".to_owned());
    }

    let mut body = Vec::new();
    let mut response = response;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "news source response could not be read".to_owned())?
    {
        if body.len().saturating_add(chunk.len()) > MAX_COMPRESSED_BYTES {
            return Err("news source response exceeds its size bound".to_owned());
        }
        body.extend_from_slice(&chunk);
    }

    let xml =
        Su3::parse_news(&body, true).ok_or_else(|| "news source failed validation".to_owned())?;
    render_news(&xml)
}

fn render_news(xml: &[u8]) -> Result<String, String> {
    if xml.len() > 4 * 1024 * 1024 {
        return Err("news XML exceeds its size bound".to_owned());
    }
    let mut parser = XmlParser::new(xml);
    let root = parser.parse_document()?;
    if root.name != "feed" {
        return Err("news XML has no feed root".to_owned());
    }

    let entries = root.children.iter().filter_map(XmlPart::element).collect::<Vec<_>>();
    validate_release_metadata(&entries)?;
    let entries = entries.into_iter().filter(|node| node.name == "entry").collect::<Vec<_>>();
    if entries.len() > MAX_ENTRIES {
        return Err("news feed contains too many entries".to_owned());
    }

    let mut output = String::new();
    for entry in entries {
        render_entry(entry, &mut output)?;
        if output.len() > MAX_RENDERED_BYTES {
            return Err("rendered news exceeds its size bound".to_owned());
        }
    }
    Ok(output)
}

fn validate_release_metadata(children: &[&XmlNode]) -> Result<(), String> {
    let releases = children.iter().filter(|node| node.name == "i2p:release").collect::<Vec<_>>();
    if releases.is_empty() {
        return Err("news feed has no release metadata".to_owned());
    }
    for release in releases {
        if child_text(release, "i2p:version").is_none_or(|version| version.is_empty()) {
            return Err("news release has no version".to_owned());
        }
        let updates = release
            .children
            .iter()
            .filter_map(XmlPart::element)
            .filter(|node| node.name == "i2p:update")
            .collect::<Vec<_>>();
        if updates.is_empty()
            || updates.iter().any(|update| {
                attr(update, "type").is_none_or(str::is_empty)
                    || !update.children.iter().filter_map(XmlPart::element).any(|source| {
                        matches!(
                            source.name.as_str(),
                            "i2p:url" | "i2p:clearnet" | "i2p:clearnetssl" | "i2p:torrent"
                        ) && attr(source, "href").is_some_and(|href| !href.is_empty())
                    })
            })
        {
            return Err("news release has incomplete update metadata".to_owned());
        }
    }
    Ok(())
}

fn render_entry(entry: &XmlNode, output: &mut String) -> Result<(), String> {
    let title = child_text(entry, "title").ok_or_else(|| "news entry has no title".to_owned())?;
    let content = child(entry, "content").ok_or_else(|| "news entry has no content".to_owned())?;
    if attr(content, "type") != Some("xhtml") {
        return Err("news entry content is not XHTML".to_owned());
    }
    if title.len() > MAX_FIELD_BYTES {
        return Err("news entry title exceeds its size bound".to_owned());
    }
    if xhtml_size(content) > MAX_FIELD_BYTES {
        return Err("news entry content exceeds its size bound".to_owned());
    }
    let updated = child_text(entry, "updated").unwrap_or_default();
    let date = updated
        .is_empty()
        .then_some(String::new())
        .or_else(|| format_news_date(&updated))
        .ok_or_else(|| "news entry has an invalid update time".to_owned())?;
    let link = child(entry, "link").and_then(|node| attr(node, "href"));
    if let Some(link) = link {
        validate_link(link)?;
    }
    let author = child(entry, "author").and_then(|node| child_text(node, "name"));

    output.push_str("<div class=\"newsentry\"><h3>");
    if !date.is_empty() {
        output.push_str("<span class=\"newsDate\">");
        push_escaped(output, &date);
        output.push_str("</span> ");
    }
    if let Some(link) = link {
        output.push_str("<a href=\"");
        push_escaped_attr(output, link);
        output.push_str("\">");
    }
    push_escaped(output, &title);
    if link.is_some() {
        output.push_str("</a>");
    }
    if let Some(author) = author {
        if author.len() > MAX_FIELD_BYTES {
            return Err("news entry author exceeds its size bound".to_owned());
        }
        output.push_str(" <span class=\"newsAuthor\" title=\"Post author\"><i>");
        push_escaped(output, &author);
        output.push_str("</i></span>\n");
    }
    output.push_str("</h3>\n<div class=\"newscontent\">\n");
    for part in &content.children {
        render_xhtml(part, output)?;
    }
    output.push_str("\n</div></div>\n");
    Ok(())
}

fn xhtml_size(part: &XmlNode) -> usize {
    part.children
        .iter()
        .map(|part| match part {
            XmlPart::Text(text) => text.len(),
            XmlPart::Element(node) => {
                node.name.len()
                    + node.attrs.iter().map(|(name, value)| name.len() + value.len()).sum::<usize>()
                    + node
                        .children
                        .iter()
                        .map(|child| match child {
                            XmlPart::Text(text) => text.len(),
                            XmlPart::Element(node) => xhtml_size(node),
                        })
                        .sum::<usize>()
            }
        })
        .sum()
}

fn render_xhtml(part: &XmlPart, output: &mut String) -> Result<(), String> {
    match part {
        XmlPart::Text(text) => push_escaped(output, text),
        XmlPart::Element(node) => {
            const ALLOWED: &[&str] = &[
                "a",
                "b",
                "br",
                "div",
                "i",
                "p",
                "span",
                "font",
                "blockquote",
                "hr",
                "del",
                "ins",
                "em",
                "strong",
                "mark",
                "sub",
                "sup",
                "tt",
                "code",
                "strike",
                "s",
                "u",
                "h4",
                "h5",
                "h6",
                "ol",
                "ul",
                "li",
                "dl",
                "dt",
                "dd",
                "table",
                "tr",
                "td",
                "th",
            ];
            if !ALLOWED.contains(&node.name.as_str()) {
                return Err("news content contains a disallowed element".to_owned());
            }
            for (name, value) in &node.attrs {
                let lower = name.to_ascii_lowercase();
                if lower.starts_with("on") || lower == "style" || lower == "src" {
                    return Err("news content contains an active attribute".to_owned());
                }
                if lower == "href" {
                    validate_link(value)?;
                }
            }
            output.push('<');
            output.push_str(&node.name);
            for (name, value) in &node.attrs {
                output.push(' ');
                output.push_str(name);
                output.push_str("=\"");
                push_escaped_attr(output, value);
                output.push('"');
            }
            if node.children.is_empty() {
                output.push_str(" />");
            } else {
                output.push('>');
                for child in &node.children {
                    render_xhtml(child, output)?;
                }
                output.push_str("</");
                output.push_str(&node.name);
                output.push('>');
            }
        }
    }
    Ok(())
}

fn validate_link(link: &str) -> Result<(), String> {
    if link.len() > MAX_FIELD_BYTES
        || link.to_ascii_lowercase().starts_with("javascript:")
        || link.to_ascii_lowercase().starts_with("data:")
    {
        return Err("news content contains an unsafe link".to_owned());
    }
    Ok(())
}

fn format_news_date(value: &str) -> Option<String> {
    if value.len() < 20 {
        return None;
    }
    let date = value.get(..10)?;
    let bytes = date.as_bytes();
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }
    let year = date.get(..4)?.parse::<u16>().ok()?;
    let month = date.get(5..7)?.parse::<u8>().ok()?;
    let day = date.get(8..10)?.parse::<u8>().ok()?;
    let time = value.as_bytes();
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || time.get(10) != Some(&b'T')
        || time.get(13) != Some(&b':')
        || time.get(16) != Some(&b':')
        || !time.get(11..13)?.iter().all(u8::is_ascii_digit)
        || !time.get(14..16)?.iter().all(u8::is_ascii_digit)
        || !time.get(17..19)?.iter().all(u8::is_ascii_digit)
    {
        return None;
    }
    let hour = value.get(11..13)?.parse::<u8>().ok()?;
    let minute = value.get(14..16)?.parse::<u8>().ok()?;
    let second = value.get(17..19)?.parse::<u8>().ok()?;
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let suffix = value.get(19..)?;
    if suffix == "Z" {
        // RFC 3339 UTC without a fractional second.
    } else if let Some(fraction) = suffix.strip_prefix('.') {
        let timezone = fraction.find(['Z', '+', '-'])?;
        if timezone == 0 || !fraction[..timezone].bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        validate_timezone(&fraction[timezone..])?;
    } else {
        validate_timezone(suffix)?;
    }
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    Some(format!("{} {}, {year}", months[month as usize - 1], day))
}

fn validate_timezone(value: &str) -> Option<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 6
        || !matches!(bytes[0], b'+' | b'-')
        || bytes[3] != b':'
        || !bytes[1..3].iter().all(u8::is_ascii_digit)
        || !bytes[4..6].iter().all(u8::is_ascii_digit)
        || bytes[1..3].iter().fold(0, |value, digit| value * 10 + digit - b'0') > 23
        || bytes[4..6].iter().fold(0, |value, digit| value * 10 + digit - b'0') > 59
    {
        return None;
    }
    Some(())
}

fn child<'a>(node: &'a XmlNode, name: &str) -> Option<&'a XmlNode> {
    node.children
        .iter()
        .filter_map(XmlPart::element)
        .find(|child| child.name == name)
}

fn child_text(node: &XmlNode, name: &str) -> Option<String> {
    let child = child(node, name)?;
    if child.children.iter().all(|part| matches!(part, XmlPart::Text(_))) {
        let mut text = String::new();
        for part in &child.children {
            if let XmlPart::Text(value) = part {
                text.push_str(value);
            }
        }
        return Some(text.trim().to_owned());
    }
    None
}

fn attr<'a>(node: &'a XmlNode, name: &str) -> Option<&'a str> {
    node.attrs.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

fn push_escaped(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(ch),
        }
    }
}

fn push_escaped_attr(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            _ => output.push(ch),
        }
    }
}

struct XmlNode {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<XmlPart>,
}

enum XmlPart {
    Text(String),
    Element(XmlNode),
}

impl XmlPart {
    fn element(&self) -> Option<&XmlNode> {
        match self {
            Self::Element(node) => Some(node),
            Self::Text(_) => None,
        }
    }
}

struct XmlParser<'a> {
    input: &'a [u8],
    position: usize,
    nodes: usize,
}

impl<'a> XmlParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            position: 0,
            nodes: 0,
        }
    }

    fn parse_document(&mut self) -> Result<XmlNode, String> {
        self.skip_misc()?;
        let root = self.parse_element(0)?;
        self.skip_misc()?;
        if self.position != self.input.len() {
            return Err("trailing data after news XML".to_owned());
        }
        Ok(root)
    }

    fn parse_element(&mut self, depth: usize) -> Result<XmlNode, String> {
        if depth > MAX_XML_DEPTH || self.nodes >= MAX_XML_NODES {
            return Err("news XML nesting/node bound exceeded".to_owned());
        }
        self.expect_byte(b'<')?;
        let name = self.parse_name()?;
        let mut attrs = Vec::new();
        loop {
            self.skip_ascii_whitespace();
            if self.consume(b"/>") {
                self.nodes += 1;
                return Ok(XmlNode {
                    name,
                    attrs,
                    children: Vec::new(),
                });
            }
            if self.consume(b">") {
                break;
            }
            let attr_name = self.parse_name()?;
            if attrs.iter().any(|(existing, _)| existing == &attr_name) {
                return Err("news XML contains duplicate attributes".to_owned());
            }
            self.skip_ascii_whitespace();
            self.expect_byte(b'=')?;
            self.skip_ascii_whitespace();
            let quote =
                self.next_byte().ok_or_else(|| "truncated news XML attribute".to_owned())?;
            if quote != b'\'' && quote != b'"' {
                return Err("news XML attribute is not quoted".to_owned());
            }
            let value = self.parse_quoted(quote)?;
            attrs.push((attr_name, value));
        }

        let mut children = Vec::new();
        loop {
            if self.consume(b"</") {
                let close_name = self.parse_name()?;
                self.skip_ascii_whitespace();
                self.expect_byte(b'>')?;
                if close_name != name {
                    return Err("news XML closing tag does not match".to_owned());
                }
                self.nodes += 1;
                return Ok(XmlNode {
                    name,
                    attrs,
                    children,
                });
            }
            if self.position >= self.input.len() {
                return Err("truncated news XML element".to_owned());
            }
            if self.input[self.position] == b'<' {
                if self.consume(b"<!--") {
                    self.skip_until(b"-->")?;
                } else if self.consume(b"<?") {
                    self.skip_until(b"?>")?;
                } else if self.consume(b"<![CDATA[") {
                    let text = self.take_until(b"]]>")?;
                    children.push(XmlPart::Text(
                        String::from_utf8(text).map_err(|_| "news XML is not UTF-8".to_owned())?,
                    ));
                } else if self.input.get(self.position + 1) == Some(&b'!') {
                    return Err("news XML declaration is not allowed here".to_owned());
                } else {
                    children.push(XmlPart::Element(self.parse_element(depth + 1)?));
                }
            } else {
                let end = self.input[self.position..]
                    .iter()
                    .position(|byte| *byte == b'<')
                    .map_or(self.input.len(), |offset| self.position + offset);
                let text = self.decode_entities(&self.input[self.position..end])?;
                if !text.is_empty() {
                    children.push(XmlPart::Text(text));
                }
                self.position = end;
            }
        }
    }

    fn skip_misc(&mut self) -> Result<(), String> {
        loop {
            self.skip_ascii_whitespace();
            if self.consume(b"<?xml") {
                self.skip_until(b"?>")?;
            } else if self.consume(b"<!--") {
                self.skip_until(b"-->")?;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let start = self.position;
        while let Some(byte) = self.input.get(self.position) {
            if byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.') {
                self.position += 1;
            } else {
                break;
            }
        }
        if start == self.position {
            return Err("news XML has an invalid name".to_owned());
        }
        String::from_utf8(self.input[start..self.position].to_vec())
            .map_err(|_| "news XML name is not UTF-8".to_owned())
    }

    fn parse_quoted(&mut self, quote: u8) -> Result<String, String> {
        let start = self.position;
        while let Some(byte) = self.input.get(self.position) {
            if *byte == quote {
                let value = self.decode_entities(&self.input[start..self.position])?;
                self.position += 1;
                return Ok(value);
            }
            if *byte == b'<' {
                return Err("news XML attribute contains '<'".to_owned());
            }
            self.position += 1;
        }
        Err("truncated news XML attribute".to_owned())
    }

    fn decode_entities(&self, bytes: &[u8]) -> Result<String, String> {
        let raw = std::str::from_utf8(bytes).map_err(|_| "news XML is not UTF-8".to_owned())?;
        let mut output = String::with_capacity(raw.len());
        let mut rest = raw;
        while let Some(start) = rest.find('&') {
            output.push_str(&rest[..start]);
            let end =
                rest[start..].find(';').ok_or_else(|| "unterminated XML entity".to_owned())?
                    + start;
            let entity = &rest[start + 1..end];
            let decoded = match entity {
                "amp" => '&',
                "lt" => '<',
                "gt" => '>',
                "quot" => '"',
                "apos" => '\'',
                _ if entity.starts_with("#x") => char::from_u32(
                    u32::from_str_radix(&entity[2..], 16)
                        .map_err(|_| "invalid XML entity".to_owned())?,
                )
                .ok_or_else(|| "invalid XML entity".to_owned())?,
                _ if entity.starts_with('#') => char::from_u32(
                    entity[1..].parse::<u32>().map_err(|_| "invalid XML entity".to_owned())?,
                )
                .ok_or_else(|| "invalid XML entity".to_owned())?,
                _ => return Err("unknown XML entity".to_owned()),
            };
            output.push(decoded);
            rest = &rest[end + 1..];
        }
        output.push_str(rest);
        Ok(output)
    }

    fn skip_until(&mut self, marker: &[u8]) -> Result<(), String> {
        self.take_until(marker)?;
        Ok(())
    }

    fn take_until(&mut self, marker: &[u8]) -> Result<Vec<u8>, String> {
        let start = self.position;
        let relative = self.input[self.position..]
            .windows(marker.len())
            .position(|window| window == marker)
            .ok_or_else(|| "truncated news XML section".to_owned())?;
        let end = self.position + relative;
        self.position = end + marker.len();
        Ok(self.input[start..end].to_vec())
    }

    fn skip_ascii_whitespace(&mut self) {
        while self.input.get(self.position).is_some_and(u8::is_ascii_whitespace) {
            self.position += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.next_byte() == Some(expected) {
            Ok(())
        } else {
            Err("malformed news XML".to_owned())
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.input.get(self.position).copied();
        self.position = self.position.saturating_add(1);
        byte
    }

    fn consume(&mut self, expected: &[u8]) -> bool {
        if self
            .input
            .get(self.position..)
            .is_some_and(|remaining| remaining.starts_with(expected))
        {
            self.position += expected.len();
            true
        } else {
            false
        }
    }
}

impl From<NewsSnapshotError> for InspectionError {
    fn from(error: NewsSnapshotError) -> Self {
        InspectionError::UnavailableReason {
            group: super::router_info::InspectionGroup::Retained,
            reason: error.reason(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(content: &str) -> Vec<u8> {
        format!(
            r#"<feed xmlns="http://www.w3.org/2005/Atom" xmlns:i2p="http://geti2p.net/en/docs/spec/updates">
<entry><title>Example &amp; news</title><link href="http://example.i2p/news"/><id>urn:uuid:1</id><updated>2026-08-28T00:00:00Z</updated><author><name>author</name></author><content type="xhtml"><div xmlns="http://www.w3.org/1999/xhtml">{content}</div></content></entry>
<i2p:release date="2026-08-28T00:00:00Z"><i2p:version>1.0</i2p:version><i2p:update type="su3"><i2p:url href="http://example.i2p/update.su3"/></i2p:update></i2p:release></feed>"#
        )
        .into_bytes()
    }

    #[test]
    fn renders_reference_news_shape_and_escapes_fields() {
        let rendered = render_news(&feed("<p>Hello &amp; <strong>world</strong>.</p>")).unwrap();
        assert!(rendered.contains("<span class=\"newsDate\">Aug 28, 2026</span>"));
        assert!(rendered.contains("Example &amp; news"));
        assert!(rendered.contains("<strong>world</strong>"));
    }

    #[test]
    fn rejects_active_content_and_malformed_xml() {
        assert!(render_news(&feed("<script>alert(1)</script>")).is_err());
        assert!(render_news(&feed("<p>unterminated")).is_err());
    }

    #[test]
    fn rejects_unsafe_links_and_oversized_entries() {
        assert!(render_news(&feed("<a href=\"javascript:alert(1)\">bad</a>")).is_err());
        assert!(render_news(&feed(&"x".repeat(MAX_FIELD_BYTES + 1))).is_err());
    }

    #[test]
    fn snapshot_is_local_and_staleness_is_truthful() {
        let now = Instant::now();
        let missing = NewsState { current: None };
        assert_eq!(
            snapshot_state(&missing),
            Err(NewsSnapshotError::Unavailable)
        );

        let current = NewsState {
            current: Some(("<news />".to_owned(), now)),
        };
        assert_eq!(snapshot_state(&current), Ok("<news />".to_owned()));

        let stale = NewsState {
            current: Some((
                "<old-news />".to_owned(),
                now - MAX_STALENESS - Duration::from_secs(1),
            )),
        };
        assert_eq!(snapshot_state(&stale), Err(NewsSnapshotError::Stale));
    }
}
