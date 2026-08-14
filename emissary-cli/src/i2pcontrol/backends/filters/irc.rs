//! Bounded IRC anonymity filtering shared by IRC-family client frontends.

use std::fmt;

use tokio::io::{
    self, AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
    BufReader,
};

/// Maximum accepted IRC line body, with compatibility headroom over RFC 2812's 512 bytes.
pub const MAX_IRC_LINE: usize = 2048;
const SAFE_PING_TOKEN: &[u8] = b"emissary-ping";
const PART_REASON: &[u8] = b"leaving";
const QUIT_REASON: &[u8] = b"leaving";

/// Direction of a line crossing the anonymity boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrcDirection {
    ClientToNetwork,
    NetworkToClient,
}

/// Result of applying the IRC policy to one complete line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterDecision {
    /// Forward a normalized or rewritten line.
    Forward(Vec<u8>),
    /// Silently discard the line.
    Drop,
    /// Close the connection because the line is malformed or unsafe to continue.
    Close,
}

/// Per-connection state for IRC protocol normalization.
#[derive(Debug, Default)]
pub struct IrcFilter {
    awaiting_client_pong: bool,
    awaiting_server_pong: bool,
}

impl IrcFilter {
    /// Apply the client-to-network policy.
    pub fn client_to_network(&mut self, line: &[u8]) -> FilterDecision {
        self.apply(IrcDirection::ClientToNetwork, line)
    }

    /// Apply the network-to-client policy.
    pub fn network_to_client(&mut self, line: &[u8]) -> FilterDecision {
        self.apply(IrcDirection::NetworkToClient, line)
    }

    fn apply(&mut self, direction: IrcDirection, line: &[u8]) -> FilterDecision {
        let Some(message) = ParsedMessage::parse(line) else {
            return FilterDecision::Close;
        };
        let command = message.command.as_str();

        match direction {
            IrcDirection::ClientToNetwork => self.client_to_network_message(&message, command),
            IrcDirection::NetworkToClient => self.network_to_client_message(&message, command),
        }
    }

    fn client_to_network_message(
        &mut self,
        message: &ParsedMessage,
        command: &str,
    ) -> FilterDecision {
        match command {
            "USER" => rewrite_user(message, b"0"),
            "PING" => {
                if message.params.is_empty() {
                    return FilterDecision::Close;
                }
                self.awaiting_server_pong = true;
                forward_with_params("PING", &[SAFE_PING_TOKEN], true)
            }
            "PONG" => {
                if !self.awaiting_client_pong {
                    return FilterDecision::Drop;
                }
                self.awaiting_client_pong = false;
                forward_with_params("PONG", &[SAFE_PING_TOKEN], true)
            }
            "PART" => {
                if message.params.is_empty() {
                    return FilterDecision::Close;
                }
                forward_with_params("PART", &[message.params[0].as_slice(), PART_REASON], true)
            }
            "QUIT" => forward_with_params("QUIT", &[QUIT_REASON], true),
            "PRIVMSG" | "NOTICE" => filter_message_text(message),
            command if CLIENT_COMMANDS.contains(&command) => FilterDecision::Forward(message.raw()),
            _ => FilterDecision::Drop,
        }
    }

    fn network_to_client_message(
        &mut self,
        message: &ParsedMessage,
        command: &str,
    ) -> FilterDecision {
        match command {
            "PING" => {
                if message.params.is_empty() {
                    return FilterDecision::Close;
                }
                self.awaiting_client_pong = true;
                forward_with_params("PING", &[SAFE_PING_TOKEN], true)
            }
            "PONG" => {
                if !self.awaiting_server_pong {
                    return FilterDecision::Drop;
                }
                self.awaiting_server_pong = false;
                forward_with_params("PONG", &[SAFE_PING_TOKEN], true)
            }
            "PRIVMSG" | "NOTICE" => filter_message_text(message),
            command if command.as_bytes().iter().all(u8::is_ascii_digit) && command.len() == 3 => {
                FilterDecision::Forward(message.raw())
            }
            command if SERVER_COMMANDS.contains(&command) => FilterDecision::Forward(message.raw()),
            _ => FilterDecision::Drop,
        }
    }
}

fn filter_message_text(message: &ParsedMessage) -> FilterDecision {
    let Some(text) = message.params.last() else {
        return FilterDecision::Close;
    };
    match classify_ctcp(text) {
        CtcpDecision::Ordinary | CtcpDecision::Action => FilterDecision::Forward(message.raw()),
        CtcpDecision::Blocked | CtcpDecision::Malformed => FilterDecision::Drop,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CtcpDecision {
    Ordinary,
    Action,
    Blocked,
    Malformed,
}

fn classify_ctcp(text: &[u8]) -> CtcpDecision {
    let markers = text.iter().filter(|&&byte| byte == 0x01).count();
    if markers == 0 {
        return CtcpDecision::Ordinary;
    }
    if markers != 2 || !text.starts_with(b"\x01") || !text.ends_with(b"\x01") {
        return CtcpDecision::Malformed;
    }
    let inner = &text[1..text.len() - 1];
    let Some(token) = inner.split(|&byte| byte == b' ').next() else {
        return CtcpDecision::Malformed;
    };
    if token.eq_ignore_ascii_case(b"ACTION") {
        return CtcpDecision::Action;
    }
    if token.eq_ignore_ascii_case(b"DCC") {
        return CtcpDecision::Blocked;
    }
    CtcpDecision::Blocked
}

fn rewrite_user(message: &ParsedMessage, hostname: &[u8]) -> FilterDecision {
    if message.params.len() < 4 {
        return FilterDecision::Close;
    }
    forward_with_params(
        "USER",
        &[
            message.params[0].as_slice(),
            message.params[1].as_slice(),
            hostname,
            message.params[3].as_slice(),
        ],
        true,
    )
}

fn forward_with_params(command: &str, params: &[&[u8]], trailing_last: bool) -> FilterDecision {
    let mut output = Vec::with_capacity(MAX_IRC_LINE.min(128));
    output.extend_from_slice(command.as_bytes());
    for (index, param) in params.iter().enumerate() {
        output.push(b' ');
        if trailing_last && index + 1 == params.len() {
            output.push(b':');
        }
        output.extend_from_slice(param);
    }
    output.extend_from_slice(b"\r\n");
    if output.len() > MAX_IRC_LINE + 2 {
        FilterDecision::Close
    } else {
        FilterDecision::Forward(output)
    }
}

const CLIENT_COMMANDS: &[&str] = &[
    "PASS",
    "NICK",
    "CAP",
    "AUTHENTICATE",
    "JOIN",
    "MODE",
    "TOPIC",
    "AWAY",
    "WHO",
    "WHOIS",
    "WHOWAS",
    "LIST",
    "INVITE",
    "KICK",
    "NOTICE",
    "PRIVMSG",
    "VERSION",
    "LUSERS",
    "MOTD",
    "TIME",
    "NAMES",
    "ISON",
    "OPER",
    "REHASH",
    "RULES",
    "STATS",
    "LINKS",
    "MAP",
    "INFO",
    "USERS",
    "SUMMON",
    "WALLOPS",
    "SQUIT",
    "ERROR",
    "QUIT",
    "PART",
    "PONG",
    "PING",
    "USER",
];

const SERVER_COMMANDS: &[&str] = &[
    "CAP",
    "AUTHENTICATE",
    "MODE",
    "JOIN",
    "NICK",
    "QUIT",
    "PART",
    "ERROR",
    "KICK",
    "TOPIC",
    "PROTOCTL",
    "AWAY",
    "ACCOUNT",
    "CHGHOST",
    "NOTICE",
    "PRIVMSG",
    "PONG",
    "PING",
    "ERROR",
    "INVITE",
    "WALLOPS",
    "JOIN",
    "REPLY",
    "FAIL",
    "WARN",
    "NOTE",
    "TAGMSG",
    "BATCH",
];

#[derive(Debug)]
struct ParsedMessage {
    command: String,
    params: Vec<Vec<u8>>,
    body: Vec<u8>,
}

impl ParsedMessage {
    fn parse(line: &[u8]) -> Option<Self> {
        if line.len() > MAX_IRC_LINE + 2 {
            return None;
        }
        let mut body = line;
        if body.ends_with(b"\r\n") {
            body = &body[..body.len() - 2];
        } else if body.ends_with(b"\n") || body.ends_with(b"\r") {
            body = &body[..body.len() - 1];
        }
        if body.is_empty() || body.len() > MAX_IRC_LINE {
            return None;
        }
        if body
            .iter()
            .any(|&byte| (byte < 0x20 && byte != 0x01) || byte == 0x7f || byte == 0)
        {
            return None;
        }

        let mut position = 0;
        skip_spaces(body, &mut position);
        if body.get(position) == Some(&b'@') {
            let end = body[position..].iter().position(|&byte| byte == b' ')? + position;
            position = end;
            skip_spaces(body, &mut position);
        }
        if body.get(position) == Some(&b':') {
            let end = body[position..].iter().position(|&byte| byte == b' ')? + position;
            position = end;
            skip_spaces(body, &mut position);
        }
        let command_start = position;
        while position < body.len() && body[position] != b' ' {
            position += 1;
        }
        let command_bytes = body.get(command_start..position)?;
        if command_bytes.is_empty()
            || command_bytes.len() > 16
            || !command_bytes.iter().all(u8::is_ascii_alphanumeric)
        {
            return None;
        }
        let command = String::from_utf8(command_bytes.to_ascii_uppercase()).ok()?;
        let mut params = Vec::new();
        while position < body.len() {
            skip_spaces(body, &mut position);
            if position >= body.len() {
                break;
            }
            if body[position] == b':' {
                params.push(body[position + 1..].to_vec());
                break;
            }
            let start = position;
            while position < body.len() && body[position] != b' ' {
                position += 1;
            }
            params.push(body[start..position].to_vec());
        }
        Some(Self {
            command,
            params,
            body: body.to_vec(),
        })
    }

    fn raw(&self) -> Vec<u8> {
        let mut output = self.body.clone();
        output.extend_from_slice(b"\r\n");
        output
    }
}

fn skip_spaces(body: &[u8], position: &mut usize) {
    while *position < body.len() && body[*position] == b' ' {
        *position += 1;
    }
}

/// Apply one complete line without retaining connection state.
#[allow(dead_code)]
pub fn filter_line(direction: IrcDirection, line: &[u8]) -> FilterDecision {
    let mut filter = IrcFilter::default();
    match direction {
        IrcDirection::ClientToNetwork => filter.client_to_network(line),
        IrcDirection::NetworkToClient => filter.network_to_client(line),
    }
}

/// Return the normalized command and bounded parameters from one IRC line.
pub fn command_and_params(line: &[u8]) -> Option<(String, Vec<Vec<u8>>)> {
    let message = ParsedMessage::parse(line)?;
    Some((message.command, message.params))
}

/// Normalize a validated line without applying directional policy.
pub fn normalize_line(line: &[u8]) -> Option<Vec<u8>> {
    Some(ParsedMessage::parse(line)?.raw())
}

/// Rewrite a server-side registration USER line with a trusted hostname.
pub fn rewrite_server_user(line: &[u8], hostname: &[u8]) -> Option<Vec<u8>> {
    let message = ParsedMessage::parse(line)?;
    if message.command != "USER" {
        return None;
    }
    match rewrite_user(&message, hostname) {
        FilterDecision::Forward(line) => Some(line),
        _ => None,
    }
}

/// Read at most `max_line_len` bytes while retaining the reader's buffered data.
pub async fn read_bounded_line<R>(reader: &mut R, max_line_len: usize) -> io::Result<Option<Vec<u8>>>
where
    R: AsyncBufRead + Unpin,
{
    let mut line = Vec::with_capacity(max_line_len.min(256));
    let bytes = reader
        .take((max_line_len as u64).saturating_add(1))
        .read_until(b'\n', &mut line)
        .await?;
    if bytes == 0 {
        return Ok(None);
    }
    if !line.ends_with(b"\n") || line.len() > max_line_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "IRC line exceeds configured bound",
        ));
    }
    Ok(Some(line))
}

/// Relay one IRC client connection with independent per-connection filter state.
pub async fn relay_client_stream<L, R>(local: L, remote: R) -> io::Result<()>
where
    L: AsyncRead + AsyncWrite + Unpin,
    R: AsyncRead + AsyncWrite + Unpin,
{
    let (local_read, mut local_write) = io::split(local);
    let (remote_read, mut remote_write) = io::split(remote);
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(IrcFilter::default()));
    let outbound_state = std::sync::Arc::clone(&state);
    let inbound_state = std::sync::Arc::clone(&state);

    let outbound = async move {
        let mut reader = BufReader::new(local_read);
        relay_lines(
            &mut reader,
            &mut remote_write,
            outbound_state,
            IrcDirection::ClientToNetwork,
        )
        .await
    };
    let inbound = async move {
        let mut reader = BufReader::new(remote_read);
        relay_lines(
            &mut reader,
            &mut local_write,
            inbound_state,
            IrcDirection::NetworkToClient,
        )
        .await
    };

    tokio::select! {
        result = outbound => result,
        result = inbound => result,
    }
}

async fn relay_lines<R, W>(
    reader: &mut BufReader<R>,
    writer: &mut W,
    state: std::sync::Arc<tokio::sync::Mutex<IrcFilter>>,
    direction: IrcDirection,
) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let Some(line) = read_bounded_line(reader, MAX_IRC_LINE + 2).await? else {
            return Ok(());
        };
        let decision = {
            let mut filter = state.lock().await;
            match direction {
                IrcDirection::ClientToNetwork => filter.client_to_network(&line),
                IrcDirection::NetworkToClient => filter.network_to_client(&line),
            }
        };
        match decision {
            FilterDecision::Forward(line) => writer.write_all(&line).await?,
            FilterDecision::Drop => {}
            FilterDecision::Close => return Ok(()),
        }
    }
}

impl fmt::Display for IrcDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ClientToNetwork => "client-to-network",
            Self::NetworkToClient => "network-to-client",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn forward(direction: IrcDirection, line: &[u8]) -> Vec<u8> {
        match filter_line(direction, line) {
            FilterDecision::Forward(line) => line,
            other => panic!("expected forward, got {other:?}"),
        }
    }

    #[test]
    fn tags_and_prefix_are_parsed_before_command() {
        assert_eq!(
            forward(
                IrcDirection::ClientToNetwork,
                b"@label=one :nick!user@host PRIVMSG #room :hello\r\n"
            ),
            b"@label=one :nick!user@host PRIVMSG #room :hello\r\n"
        );
    }

    #[test]
    fn user_hostname_is_replaced() {
        assert_eq!(
            forward(
                IrcDirection::ClientToNetwork,
                b"USER alice 0 local-ip :Alice\r\n"
            ),
            b"USER alice 0 0 :Alice\r\n"
        );
    }

    #[test]
    fn ping_and_pong_use_connection_local_state() {
        let mut filter = IrcFilter::default();
        assert_eq!(
            filter.client_to_network(b"PING :192.0.2.9\r\n"),
            FilterDecision::Forward(b"PING :emissary-ping\r\n".to_vec())
        );
        assert_eq!(
            filter.network_to_client(b":server PONG :192.0.2.9\r\n"),
            FilterDecision::Forward(b"PONG :emissary-ping\r\n".to_vec())
        );
        let mut other = IrcFilter::default();
        assert_eq!(
            other.network_to_client(b":server PONG :token\r\n"),
            FilterDecision::Drop
        );
    }

    #[test]
    fn part_and_quit_reasons_are_neutralized() {
        assert_eq!(
            forward(
                IrcDirection::ClientToNetwork,
                b"PART #room :meet me at 192.0.2.1\r\n"
            ),
            b"PART #room :leaving\r\n"
        );
        assert_eq!(
            forward(
                IrcDirection::ClientToNetwork,
                b"QUIT :my home address is here\r\n"
            ),
            b"QUIT :leaving\r\n"
        );
    }

    #[test]
    fn ordinary_messages_and_action_pass() {
        assert!(matches!(
            filter_line(IrcDirection::ClientToNetwork, b"PRIVMSG #room :hello\r\n"),
            FilterDecision::Forward(_)
        ));
        assert_eq!(
            forward(
                IrcDirection::ClientToNetwork,
                b"PRIVMSG #room :\x01ACTION waves\x01\r\n"
            ),
            b"PRIVMSG #room :\x01ACTION waves\x01\r\n"
        );
    }

    #[test]
    fn unsupported_ctcp_and_dcc_are_blocked() {
        for body in [
            b"\x01VERSION\x01".as_slice(),
            b"\x01TIME\x01".as_slice(),
            b"\x01DCC CHAT 1 2 3\x01".as_slice(),
            b"\x01DCC SEND file 1 2 3\x01".as_slice(),
            b"\x01DCC RESUME file 1 2\x01".as_slice(),
            b"\x01DCC ACCEPT file 1 2\x01".as_slice(),
        ] {
            let mut line = b"PRIVMSG #room :".to_vec();
            line.extend_from_slice(body);
            line.extend_from_slice(b"\r\n");
            assert_eq!(
                filter_line(IrcDirection::ClientToNetwork, &line),
                FilterDecision::Drop
            );
        }
    }

    #[test]
    fn malformed_ctcp_and_unknown_commands_fail_closed() {
        assert_eq!(
            filter_line(
                IrcDirection::ClientToNetwork,
                b"PRIVMSG #room :\x01ACTION unclosed\r\n"
            ),
            FilterDecision::Drop
        );
        assert_eq!(
            filter_line(IrcDirection::ClientToNetwork, b"UNREVIEWED secret\r\n"),
            FilterDecision::Drop
        );
        assert_eq!(
            filter_line(
                IrcDirection::ClientToNetwork,
                b"PRIVMSG #room :bad\x00text\r\n"
            ),
            FilterDecision::Close
        );
    }

    #[test]
    fn overlong_lines_close() {
        let mut line = b"PRIVMSG #room :".to_vec();
        line.extend(std::iter::repeat_n(b'x', MAX_IRC_LINE));
        line.extend_from_slice(b"\r\n");
        assert_eq!(
            filter_line(IrcDirection::ClientToNetwork, &line),
            FilterDecision::Close
        );
    }

    #[tokio::test]
    async fn bidirectional_relay_filters_both_sides_with_isolated_state() {
        let (local_client, local_relay) = tokio::io::duplex(4096);
        let (remote_relay, remote_server) = tokio::io::duplex(4096);
        let relay = tokio::spawn(relay_client_stream(local_relay, remote_relay));
        let (mut local_read, mut local_write) = tokio::io::split(local_client);
        let (mut remote_read, mut remote_write) = tokio::io::split(remote_server);
        let mut local_reader = BufReader::new(&mut local_read);
        let mut remote_reader = BufReader::new(&mut remote_read);

        local_write.write_all(b"USER alice 0 192.0.2.1 :Alice\r\n").await.unwrap();
        let mut line = Vec::new();
        remote_reader.read_until(b'\n', &mut line).await.unwrap();
        assert_eq!(line, b"USER alice 0 0 :Alice\r\n");

        remote_write.write_all(b":server PING :192.0.2.9\r\n").await.unwrap();
        line.clear();
        local_reader.read_until(b'\n', &mut line).await.unwrap();
        assert_eq!(line, b"PING :emissary-ping\r\n");
        local_write.write_all(b"PONG :192.0.2.9\r\n").await.unwrap();
        line.clear();
        remote_reader.read_until(b'\n', &mut line).await.unwrap();
        assert_eq!(line, b"PONG :emissary-ping\r\n");

        drop(local_write);
        drop(remote_write);
        let _ = relay.await;
    }
}
