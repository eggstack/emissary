// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![cfg_attr(not(feature = "i2pcontrol"), allow(dead_code))]

use crate::config::AddressBookConfig;

use emissary_core::{
    crypto::{base32_encode, base64_decode, base64_encode},
    primitives::Destination,
    runtime::AddressBook,
};
use futures::channel::oneshot;
use parking_lot::RwLock;
use reqwest::{
    header::{
        HeaderMap, HeaderValue, CONNECTION, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED,
    },
    Client, Proxy, StatusCode,
};

use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    time::Duration,
};

#[cfg(feature = "i2pcontrol")]
use std::sync::Mutex;

#[cfg(feature = "i2pcontrol")]
use tokio::sync::mpsc;

/// Logging target for the file
const LOG_TARGET: &str = "emissary::address-book";

/// Backoff if downloading the hosts file fails.
const RETRY_BACKOFF: Duration = Duration::from_secs(30);

/// How many times each subscription is tried before giving up.
const SUBSCRIPTION_NUM_RETRIES: usize = 5usize;

#[cfg(feature = "i2pcontrol")]
fn build_refresh_client(settings: &RuntimeRefreshSettings) -> Result<Client, reqwest::Error> {
    let host = if settings.proxy_host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]", settings.proxy_host)
    } else {
        settings.proxy_host.clone()
    };
    Client::builder()
        .proxy(Proxy::http(format!(
            "http://{host}:{}",
            settings.proxy_port
        ))?)
        .http1_title_case_headers()
        .build()
}

fn build_legacy_client(http_host: &str, http_port: u16) -> Client {
    Client::builder()
        .proxy(Proxy::http(format!("http://{http_host}:{http_port}")).expect("to succeed"))
        .http1_title_case_headers()
        .build()
        .expect("to succeed")
}

#[cfg(feature = "i2pcontrol")]
#[derive(Clone)]
struct RuntimeRefreshContext {
    address_book_path: PathBuf,
    addresses: Arc<RwLock<HashMap<String, String>>>,
    hook: Arc<dyn AddressBookRuntimeHook>,
    control: Arc<RuntimeSubscriptionControl>,
}

#[cfg(feature = "i2pcontrol")]
pub(crate) struct AddressBookRuntimeContext {
    pub(crate) path: PathBuf,
    pub(crate) addresses: Arc<RwLock<HashMap<String, String>>>,
    pub(crate) serialized: Arc<RwLock<String>>,
}

#[cfg(feature = "i2pcontrol")]
pub(crate) struct RuntimeSubscriptionCommand {
    pub(crate) subscriptions: Vec<String>,
    pub(crate) response: oneshot::Sender<Result<(), String>>,
}

#[cfg(feature = "i2pcontrol")]
#[derive(Clone, Debug)]
pub(crate) struct RuntimeRefreshSettings {
    pub(crate) interval: Duration,
    pub(crate) proxy_host: String,
    pub(crate) proxy_port: u16,
    pub(crate) subscriptions_path: PathBuf,
    pub(crate) etags_path: PathBuf,
    pub(crate) last_modified_path: PathBuf,
    pub(crate) should_publish: bool,
}

#[cfg(feature = "i2pcontrol")]
impl RuntimeRefreshSettings {
    pub(crate) fn defaults(address_book_path: &Path, proxy_host: &str, proxy_port: u16) -> Self {
        Self {
            interval: Duration::from_secs(24 * 60 * 60),
            proxy_host: proxy_host.to_owned(),
            proxy_port,
            subscriptions_path: address_book_path.join("subscriptions"),
            etags_path: address_book_path.join("etags"),
            last_modified_path: address_book_path.join("last_modified"),
            should_publish: true,
        }
    }
}

#[cfg(feature = "i2pcontrol")]
pub(crate) struct RuntimeSubscriptionControl {
    pub(crate) sender: tokio::sync::mpsc::Sender<RuntimeSubscriptionCommand>,
    pub(crate) active: RwLock<Vec<String>>,
    pub(crate) settings: RwLock<RuntimeRefreshSettings>,
    pub(crate) started: std::sync::atomic::AtomicBool,
    pub(crate) receiver: Mutex<Option<tokio::sync::mpsc::Receiver<RuntimeSubscriptionCommand>>>,
}

#[cfg(feature = "i2pcontrol")]
pub(crate) trait AddressBookRuntimeHook: Send + Sync {
    fn initial_subscriptions(&self, configured: &[String]) -> Vec<String>;
    fn refresh_settings(&self, defaults: RuntimeRefreshSettings) -> RuntimeRefreshSettings;
    fn subscription_control(&self) -> Arc<RuntimeSubscriptionControl>;
    fn commit_subscriptions(
        &self,
        subscriptions: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
    fn merge_downloaded(
        &self,
        addresses: HashMap<String, (String, String)>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn publish_artifacts(&self) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn resolve_base32(&self, hostname: &str) -> Option<String>;
    fn resolve_base64(&self, hostname: &str) -> Option<String>;
    fn legacy_publish(&self, entry: String, destination: String);
    fn legacy_remove(&self, hostname: &str);
}

#[cfg(feature = "i2pcontrol")]
impl RuntimeRefreshContext {
    fn settings(&self) -> RuntimeRefreshSettings {
        self.control.settings.read().clone()
    }

    async fn save_to_disk(&self, addresses: HashMap<String, (String, String)>) {
        let persisted_addresses = addresses.clone();
        let (addresses, destinations): (HashMap<_, _>, HashMap<_, _>) = addresses
            .into_iter()
            .map(|(hostname, (base32, base64))| ((hostname.clone(), base32), (hostname, base64)))
            .unzip();

        {
            let base32_addresses = {
                let mut inner = self.addresses.write();
                inner.extend(addresses);
                inner
                    .iter()
                    .map(|(hostname, address)| format!("{hostname}={address}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if let Err(error) =
                tokio::fs::write(self.address_book_path.join("addresses"), base32_addresses).await
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?error,
                    "failed to write base32 addresses to disk",
                );
            }
        }

        match tokio::fs::create_dir_all(self.address_book_path.join("destinations")).await {
            Err(error) => tracing::warn!(
                target: LOG_TARGET,
                ?error,
                "failed to create directory for destinations",
            ),
            Ok(_) => {
                for (hostname, destination) in destinations {
                    if let Err(error) = tokio::fs::write(
                        self.address_book_path.join("destinations").join(format!("{hostname}.txt")),
                        destination,
                    )
                    .await
                    {
                        tracing::warn!(
                            target: LOG_TARGET,
                            %hostname,
                            ?error,
                            "failed to store destination to disk",
                        );
                    }
                }
            }
        }

        self.hook.merge_downloaded(persisted_addresses).await;
        self.hook.publish_artifacts().await;
    }

    async fn refresh(
        &self,
        client: &Client,
        addresses: &mut HashMap<String, (String, String)>,
        host_modified_times: &mut HashMap<String, Modified>,
        subscriptions: &[String],
    ) {
        for subscription in subscriptions {
            AddressBookManager::download_with_retries(
                subscription,
                client,
                addresses,
                host_modified_times,
            )
            .await;
        }
        self.save_to_disk(std::mem::take(addresses)).await;
        let settings = self.settings();
        let legacy = HostModified::from(host_modified_times.clone());
        if let Ok(raw) = toml::to_string(&legacy) {
            if let Err(error) = write_refresh_file(&settings.etags_path, &raw).await {
                tracing::warn!(target: LOG_TARGET, ?error, "could not write address book etags");
            }
            if let Err(error) = write_refresh_file(&settings.last_modified_path, &raw).await {
                tracing::warn!(target: LOG_TARGET, ?error, "could not write address book last-modified metadata");
            }
        }
    }
}

#[cfg(feature = "i2pcontrol")]
async fn write_refresh_file(path: &Path, contents: &str) -> std::io::Result<()> {
    const MAX_REFRESH_FILE_BYTES: usize = 4 * 1024 * 1024;
    if contents.len() > MAX_REFRESH_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "refresh metadata exceeds its size limit",
        ));
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = path.with_extension("m096-tmp");
    tokio::fs::write(&temporary, contents).await?;
    let file = tokio::fs::File::open(&temporary).await?;
    file.sync_all().await?;
    tokio::fs::rename(&temporary, path).await
}

/// Used when requesting address books from servers. This should reduce load to servers whose
/// address books haven't changed since our last lookup.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
struct Modified {
    etag: String,
    last_modified: String,
}

/// The on-disk representation of our hosts modified times.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HostModified {
    host_modified_times: Vec<(String, Modified)>,
}

impl From<HashMap<String, Modified>> for HostModified {
    fn from(value: HashMap<String, Modified>) -> Self {
        Self {
            host_modified_times: value.into_iter().collect::<Vec<(String, Modified)>>(),
        }
    }
}

impl From<HostModified> for HashMap<String, Modified> {
    fn from(value: HostModified) -> HashMap<String, Modified> {
        HashMap::from_iter(value.host_modified_times)
    }
}

#[cfg(feature = "i2pcontrol")]
async fn load_runtime_host_modified_times(
    settings: &RuntimeRefreshSettings,
    legacy_path: &Path,
) -> HashMap<String, Modified> {
    for path in [&settings.etags_path, &settings.last_modified_path] {
        if let Ok(raw) = tokio::fs::read_to_string(path).await {
            if raw.len() <= 4 * 1024 * 1024 {
                if let Ok(metadata) = toml::from_str::<HostModified>(&raw) {
                    return metadata.into();
                }
            }
        }
    }
    let legacy_path = legacy_path.join("host_modified_times");
    match tokio::fs::read_to_string(legacy_path).await {
        Ok(raw) => toml::from_str::<HostModified>(&raw).map(Into::into).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Only modified responses are passed on.
enum Response {
    /// etags/last-modified saved the server from doing work.
    NotModified,
    /// The response body and the etag/last-modified info to save for subsequent requests.
    Modified { modified: Modified, body: String },
}

/// When a request fails, we remove the cache entry and sleep for `RETRY_BACKOFF`.
enum Error {
    /// There's an error _sending_ an HTTP request.
    Client,
    /// The HTTP request was a failure (4xx, 5xx).
    Server,
    /// We couldn't decode the response.
    Encoding,
    /// We couldn't get a response.
    Response,
}

/// Address book.
pub struct AddressBookManager {
    /// Path to address book.
    address_book_path: PathBuf,

    /// Hostname -> Base32 address mappings
    addresses: Arc<RwLock<HashMap<String, String>>>,

    /// URL from which the primary `hosts.txt` is downloaded.
    hosts_url: Option<String>,

    /// Serialized base32 addresses.
    serialized: Arc<RwLock<String>>,

    /// Additional subscriptions.
    subscriptions: Vec<String>,

    /// Optional neutral runtime hook. The legacy manager owns downloading and
    /// lookup; the hook only supplies an independently owned observation and
    /// subscription capability.
    #[cfg(feature = "i2pcontrol")]
    runtime_hook: Option<Arc<dyn AddressBookRuntimeHook>>,
}

impl AddressBookManager {
    /// Create new [`AddressBookManager`].
    pub async fn new(base_path: PathBuf, config: AddressBookConfig) -> Self {
        let path = base_path.join("addressbook");
        let (addresses, serialized) = Self::load_legacy_state(&path).await;
        let addresses = Arc::new(RwLock::new(addresses));
        let serialized = Arc::new(RwLock::new(serialized));
        Self {
            address_book_path: path,
            addresses,
            hosts_url: config.default,
            serialized,
            subscriptions: config.subscriptions.unwrap_or_default(),
            #[cfg(feature = "i2pcontrol")]
            runtime_hook: None,
        }
    }

    #[cfg(feature = "i2pcontrol")]
    pub(crate) fn runtime_context(&self) -> AddressBookRuntimeContext {
        AddressBookRuntimeContext {
            path: self.address_book_path.clone(),
            addresses: Arc::clone(&self.addresses),
            serialized: Arc::clone(&self.serialized),
        }
    }

    #[cfg(feature = "i2pcontrol")]
    pub(crate) fn with_runtime_hook(mut self, hook: Arc<dyn AddressBookRuntimeHook>) -> Self {
        self.runtime_hook = Some(hook);
        self
    }

    async fn load_legacy_state(path: &Path) -> (HashMap<String, String>, String) {
        match tokio::fs::read_to_string(path.join("addresses")).await {
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    path = %path.join("addresses").display(),
                    error = ?error.kind(),
                    "failed to load base32 addresses from disk",
                );

                (HashMap::new(), String::new())
            }
            Ok(content) => (
                content
                    .lines()
                    .filter_map(|line| {
                        line.split_once('=').map(|(key, value)| (key.to_owned(), value.to_owned()))
                    })
                    .collect(),
                content,
            ),
        }
    }

    /// Get handle to [`AddressBook`].
    pub fn handle(&self) -> Arc<AddressBookHandle> {
        Arc::new(AddressBookHandle {
            address_book_path: Arc::from(self.address_book_path.to_str().expect("to succeed")),
            addresses: Arc::clone(&self.addresses),
            serialized: Arc::clone(&self.serialized),
            #[cfg(feature = "i2pcontrol")]
            runtime_hook: self.runtime_hook.as_ref().map(Arc::clone),
        })
    }

    /// Attempt to download `hosts.txt` from `url`.
    async fn download(
        client: &Client,
        url: &str,
        modified: Option<&Modified>,
    ) -> Result<Response, Error> {
        let mut headers = HeaderMap::from_iter([(CONNECTION, HeaderValue::from_static("close"))]);
        // Consider refactoring some of this. if-let chaings may make this nicer.
        if let Some(modified) = modified {
            if !modified.etag.is_empty() {
                if let Ok(etag) = HeaderValue::from_str(&modified.etag) {
                    headers.insert(IF_NONE_MATCH, etag);
                }
            }

            if !modified.last_modified.is_empty() {
                if let Ok(last_modified) = HeaderValue::from_str(&modified.last_modified) {
                    headers.insert(IF_MODIFIED_SINCE, last_modified);
                }
            }
        }
        let response = match client.get(url.to_string()).headers(headers).send().await {
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?url,
                    ?error,
                    "failed to fetch hosts.txt"
                );
                return Err(Error::Client);
            }
            Ok(response) => response,
        };
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(Response::NotModified);
        }
        if !response.status().is_success() {
            tracing::warn!(
                target: LOG_TARGET,
                ?url,
                status = ?response.status(),
                "request to address book server failed",
            );
            return Err(Error::Server);
        }
        let modified = Modified {
            etag: response
                .headers()
                .get(ETAG)
                .and_then(|e| e.to_str().ok())
                .map(String::from)
                .unwrap_or_default(),
            last_modified: response
                .headers()
                .get(LAST_MODIFIED)
                .and_then(|l| l.to_str().ok())
                .map(String::from)
                .unwrap_or_default(),
        };
        match response.bytes().await {
            Ok(response) => match std::str::from_utf8(&response) {
                Ok(response) => Ok(Response::Modified {
                    modified,
                    body: response.to_owned(),
                }),
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        ?url,
                        ?error,
                        "failed to convert `hosts.txt` to utf-8",
                    );
                    Err(Error::Encoding)
                }
            },
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?url,
                    ?error,
                    "failed to get response from address book server"
                );
                Err(Error::Response)
            }
        }
    }

    /// Parse `hosts` into base64 destinations and base32 addresses and merge the results into
    /// `addresses`.
    ///
    /// Addresses already present in `addresses` will be ignored.
    #[allow(dead_code)]
    async fn parse_and_merge(
        &self,
        addresses: &mut HashMap<String, (String, String)>,
        hosts: String,
    ) {
        Self::parse_and_merge_impl(addresses, hosts).await;
    }

    async fn parse_and_merge_impl(
        addresses: &mut HashMap<String, (String, String)>,
        hosts: String,
    ) {
        for line in hosts.lines() {
            let Some((hostname, base64_destination)) = line.split_once('=') else {
                tracing::warn!(
                    target: LOG_TARGET,
                    %line,
                    "ignoring invalid address",
                );
                continue;
            };
            let hostname = hostname.trim().to_string();

            if addresses.contains_key(&hostname) {
                tracing::trace!(
                    target: LOG_TARGET,
                    %hostname,
                    "skipping an already-existing address",
                );
                continue;
            }

            let base64_destination = match base64_destination.find("#!") {
                Some(index) => base64_destination[..index].trim().to_string(),
                None => base64_destination.trim().to_string(),
            };

            let Some(decoded) = base64_decode(&base64_destination) else {
                tracing::warn!(
                    target: LOG_TARGET,
                    %hostname,
                    "ignoring invalid base64-encoded destination",
                );
                continue;
            };

            match Destination::parse(&decoded) {
                Err(error) => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        %hostname,
                        ?error,
                        "ignoring invalid destination",
                    );
                    continue;
                }
                Ok(destination) => {
                    let base32_address = base32_encode(destination.id().to_vec());
                    addresses.insert(hostname, (base32_address, base64_destination));
                }
            }
        }
    }

    /// Loads the modified info for each endpoint we request. In case we can't load it, we return an
    /// empty `HashMap` so the whole process doesn't fail.
    ///
    /// If we can't load it for `std::io::Error` reasons, then we probably won't be able to save
    /// either.
    async fn load_host_modified_times(&self) -> HashMap<String, Modified> {
        let path = self.address_book_path.join("host_modified_times");
        let raw = match tokio::fs::read_to_string(path).await {
            Ok(r) => r,
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?error,
                    "could not read host_modified_times",
                );
                return HashMap::new();
            }
        };

        match toml::from_str::<HostModified>(&raw) {
            Ok(h) => h.into(),
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?error,
                    "could not deserialize host_modified_times",
                );
                HashMap::new()
            }
        }
    }

    /// Serializes the host modified times as a `Vec<(String, Modified)>`. Warns on error instead of
    /// failing.
    async fn save_host_modified_times(&self, host_modified_times: HashMap<String, Modified>) {
        let path = self.address_book_path.join("host_modified_times");
        let modified = HostModified::from(host_modified_times);
        let raw = match toml::to_string(&modified) {
            Ok(r) => r,
            Err(error) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?error,
                    "could not serialize host_modified_times",
                );
                return;
            }
        };

        if let Err(error) = tokio::fs::write(path, raw).await {
            tracing::warn!(
                target: LOG_TARGET,
                ?error,
                "could not write host_modified_times",
            );
        }
    }

    /// Start event loop for [`AddressBookManager`].
    ///
    /// Before the address book subscription download starts, [`AddressBook`] waits on
    /// `http_proxy_ready_rx` which the HTTP proxy sends a signal to once it's ready.
    pub async fn run(
        self,
        http_port: u16,
        http_host: String,
        http_proxy_ready_rx: oneshot::Receiver<()>,
    ) {
        if let Err(error) = http_proxy_ready_rx.await {
            tracing::error!(
                target: LOG_TARGET,
                ?error,
                "http proxy failed to start, cannot start address book",
            );
            return;
        }

        #[cfg(feature = "i2pcontrol")]
        let subscriptions = self.runtime_hook.as_ref().map_or_else(
            || self.subscriptions.clone(),
            |hook| hook.initial_subscriptions(&self.subscriptions),
        );
        #[cfg(not(feature = "i2pcontrol"))]
        let subscriptions = self.subscriptions.clone();

        tracing::info!(
            target: LOG_TARGET,
            ?http_port,
            ?http_host,
            subscriptions = ?subscriptions,
            "create address book",
        );

        #[cfg(feature = "i2pcontrol")]
        let (client, runtime_settings) = if let Some(hook) = &self.runtime_hook {
            let control = hook.subscription_control();
            let defaults =
                RuntimeRefreshSettings::defaults(&self.address_book_path, &http_host, http_port);
            let settings = hook.refresh_settings(defaults);
            *control.settings.write() = settings.clone();
            (
                build_refresh_client(&settings).expect("to build address book client"),
                Some(settings),
            )
        } else {
            (build_legacy_client(&http_host, http_port), None)
        };
        #[cfg(not(feature = "i2pcontrol"))]
        let client = build_legacy_client(&http_host, http_port);

        #[cfg(feature = "i2pcontrol")]
        let mut host_modified_times = if let Some(settings) = runtime_settings.as_ref() {
            load_runtime_host_modified_times(settings, &self.address_book_path).await
        } else {
            self.load_host_modified_times().await
        };
        #[cfg(not(feature = "i2pcontrol"))]
        let mut host_modified_times = self.load_host_modified_times().await;

        let mut addresses = HashMap::<String, (String, String)>::new();

        if let Some(url) = &self.hosts_url {
            Self::download_with_retries(url, &client, &mut addresses, &mut host_modified_times)
                .await;

            // save hosts to disk at this point as subscriptions might contain .i2p addresses
            // which the http proxy must be able to resolve to .b32.i2p addresses
            self.save_to_disk(addresses.clone()).await;
        } else {
            tracing::debug!(
                target: LOG_TARGET,
                "default address book download skipped",
            );
        };

        for subscription in &subscriptions {
            Self::download_with_retries(
                subscription,
                &client,
                &mut addresses,
                &mut host_modified_times,
            )
            .await;
        }

        self.save_to_disk(addresses).await;
        self.save_host_modified_times(host_modified_times).await;

        #[cfg(feature = "i2pcontrol")]
        if let Some(hook) = self.runtime_hook.as_ref() {
            let control = hook.subscription_control();
            let receiver = control.receiver.lock().ok().and_then(|mut receiver| receiver.take());
            if let Some(receiver) = receiver {
                control.started.store(true, std::sync::atomic::Ordering::Release);
                self.run_subscription_control(hook.clone(), control.clone(), receiver).await;
                control.started.store(false, std::sync::atomic::Ordering::Release);
            }
        }
    }

    #[cfg(feature = "i2pcontrol")]
    async fn run_subscription_control(
        &self,
        hook: Arc<dyn AddressBookRuntimeHook>,
        control: Arc<RuntimeSubscriptionControl>,
        mut receiver: mpsc::Receiver<RuntimeSubscriptionCommand>,
    ) {
        let context = RuntimeRefreshContext {
            address_book_path: self.address_book_path.clone(),
            addresses: Arc::clone(&self.addresses),
            hook: Arc::clone(&hook),
            control: Arc::clone(&control),
        };
        let (refresh_sender, mut refresh_receiver) = mpsc::channel::<Vec<String>>(1);
        let (done_sender, mut done_receiver) = mpsc::channel(1);
        let worker_context = context.clone();
        let worker = tokio::spawn(async move {
            let mut addresses = HashMap::new();
            let mut host_modified_times = {
                let settings = worker_context.settings();
                load_runtime_host_modified_times(&settings, &worker_context.address_book_path).await
            };
            while let Some(subscriptions) = refresh_receiver.recv().await {
                let settings = worker_context.settings();
                let client = match build_refresh_client(&settings) {
                    Ok(client) => client,
                    Err(error) => {
                        tracing::warn!(target: LOG_TARGET, ?error, "address book proxy configuration unavailable");
                        if done_sender.send(()).await.is_err() {
                            break;
                        }
                        continue;
                    }
                };
                worker_context
                    .refresh(
                        &client,
                        &mut addresses,
                        &mut host_modified_times,
                        &subscriptions,
                    )
                    .await;
                if done_sender.send(()).await.is_err() {
                    break;
                }
            }
        });

        let mut refresh_busy = false;
        let mut pending_refresh = None;
        let mut next_refresh = tokio::time::Instant::now() + control.settings.read().interval;

        loop {
            tokio::select! {
                command = receiver.recv() => {
                    let Some(command) = command else { break };
                    self.handle_subscription_command(
                        Arc::clone(&hook),
                        command,
                        &mut refresh_busy,
                        &mut pending_refresh,
                        &refresh_sender,
                    )
                    .await;
                    next_refresh = tokio::time::Instant::now() + control.settings.read().interval;
                }
                done = done_receiver.recv(), if refresh_busy => {
                    if done.is_none() {
                        break;
                    }
                    refresh_busy = false;
                    if let Some(subscriptions) = pending_refresh.take() {
                        if refresh_sender.send(subscriptions).await.is_err() {
                            break;
                        }
                        refresh_busy = true;
                        next_refresh = tokio::time::Instant::now() + control.settings.read().interval;
                    } else {
                        next_refresh = tokio::time::Instant::now() + control.settings.read().interval;
                    }
                }
                _ = tokio::time::sleep_until(next_refresh), if !refresh_busy => {
                    let subscriptions = control.active.read().clone();
                    if refresh_sender.send(subscriptions).await.is_err() {
                        break;
                    }
                    refresh_busy = true;
                    next_refresh = tokio::time::Instant::now() + control.settings.read().interval;
                }
            }
        }

        worker.abort();
    }

    #[cfg(feature = "i2pcontrol")]
    async fn handle_subscription_command(
        &self,
        hook: Arc<dyn AddressBookRuntimeHook>,
        command: RuntimeSubscriptionCommand,
        refresh_busy: &mut bool,
        pending_refresh: &mut Option<Vec<String>>,
        refresh_sender: &mpsc::Sender<Vec<String>>,
    ) {
        let subscriptions = command.subscriptions;
        let result = hook.commit_subscriptions(subscriptions.clone()).await;
        if result.is_ok() {
            if *refresh_busy {
                *pending_refresh = Some(subscriptions);
            } else {
                match refresh_sender.try_send(subscriptions) {
                    Ok(()) => *refresh_busy = true,
                    Err(mpsc::error::TrySendError::Full(subscriptions)) => {
                        *pending_refresh = Some(subscriptions);
                        *refresh_busy = true;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        // The subscription owner has already committed and activated the
                        // replacement. Refresh is bounded follow-up work, so a closed worker
                        // cannot turn that completed mutation into an error response.
                        tracing::warn!(
                            target: LOG_TARGET,
                            "address book refresh worker was unavailable after subscription commit",
                        );
                    }
                }
            }
        }
        let _ = command.response.send(result);
    }

    /// A wrapper around `Self::download` which deals with retries, logging, etc.
    async fn download_with_retries(
        url: &String,
        client: &Client,
        addresses: &mut HashMap<String, (String, String)>,
        host_modified_times: &mut HashMap<String, Modified>,
    ) {
        for _ in 0..SUBSCRIPTION_NUM_RETRIES {
            match Self::download(client, url, host_modified_times.get(url)).await {
                Ok(Response::NotModified) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        %url,
                        "hosts.txt not changed since last download",
                    );
                    break;
                }
                Ok(Response::Modified { modified, body }) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        %url,
                        "hosts.txt downloaded",
                    );
                    Self::parse_and_merge_impl(addresses, body).await;
                    host_modified_times.insert(url.to_string(), modified);
                    break;
                }
                // These errors would have been logged by `Self::download` already.
                Err(_) => {
                    tokio::time::sleep(RETRY_BACKOFF).await;
                }
            }
        }
    }

    /// Save `addresses` to disk.
    ///
    /// Parses each destination in `addresses` into its .b32.i2p address, stores all .b32.i2p
    /// addresses along with their hostnames into a file and stores all .Base64-encoded destinations
    /// into a separate directory where each destination is indexed by their hostname.
    async fn save_to_disk(&self, addresses: HashMap<String, (String, String)>) {
        #[cfg(feature = "i2pcontrol")]
        let persisted_addresses = addresses.clone();
        let (addresses, destinations): (HashMap<_, _>, HashMap<_, _>) = addresses
            .into_iter()
            .map(|(hostname, (base32, base64))| ((hostname.clone(), base32), (hostname, base64)))
            .unzip();

        // store base32-encoded addresses on disk and update `addresses`
        {
            // update `addresses`
            {
                let mut inner = self.addresses.write();

                for (hostname, address) in addresses {
                    inner.insert(hostname.clone(), address.clone());
                }
            }

            // get full list of base32 addresses
            let base32_addresses = {
                let inner = self.addresses.read();

                inner.iter().fold(String::new(), |acc, (hostname, address)| {
                    format!("{hostname}={address}\n{acc}")
                })
            };

            if let Err(error) =
                tokio::fs::write(self.address_book_path.join("addresses"), base32_addresses).await
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?error,
                    "failed to write base32 addresses to disk",
                );
            }
        }

        // store base64-encoded destinations on disk
        match tokio::fs::create_dir_all(self.address_book_path.join("destinations")).await {
            Err(error) => tracing::warn!(
                target: LOG_TARGET,
                ?error,
                "failed to create directory for destinations",
            ),
            Ok(_) => {
                for (hostname, destination) in destinations {
                    if let Err(error) = tokio::fs::write(
                        self.address_book_path.join("destinations").join(format!("{hostname}.txt")),
                        destination,
                    )
                    .await
                    {
                        tracing::warn!(
                            target: LOG_TARGET,
                            %hostname,
                            ?error,
                            "failed to store destination to disk",
                        );
                    }
                }
            }
        }

        #[cfg(feature = "i2pcontrol")]
        if let Some(hook) = &self.runtime_hook {
            hook.merge_downloaded(persisted_addresses).await;
        }
    }
}

/// Address book handle.
#[derive(Clone)]
pub struct AddressBookHandle {
    /// Path to address book.
    address_book_path: Arc<str>,

    /// Hostname -> Base32 address mappings
    addresses: Arc<RwLock<HashMap<String, String>>>,

    /// Serialized list of Base32 addresses.
    #[allow(dead_code)]
    serialized: Arc<RwLock<String>>,

    /// Optional neutral runtime lookup/observation hook.
    #[cfg(feature = "i2pcontrol")]
    runtime_hook: Option<Arc<dyn AddressBookRuntimeHook>>,
}

impl AddressBookHandle {
    /// Add new host to address book.
    ///
    /// Only the base32 address of the host is stored on disk.
    #[allow(dead_code)]
    pub fn add_base32(&self, host: String, address: String) {
        self.addresses.write().insert(host.clone(), address.clone());

        // update the on-disk version of the base32 address list
        {
            let mut inner = self.serialized.write();
            inner.push_str(&format!("\n{host}={address}"));

            if let Err(error) =
                std::fs::write(format!("{}/addresses", self.address_book_path), &*inner)
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    %host,
                    %address,
                    error = ?error.kind(),
                    "failed to update on-disk base32 address list",
                );
            }
        }
    }

    /// Add new host to address book.
    ///
    /// Both the destination and the base32 address of the destination are stored on disk.
    #[allow(dead_code)]
    pub fn add_base64(&self, host: String, destination: Destination) {
        let base32_address = base32_encode(destination.id().to_vec());

        self.addresses.write().insert(host.clone(), base32_address.clone());

        // update the on-disk version of the base32 address list
        {
            let mut inner = self.serialized.write();
            inner.push_str(&format!("\n{host}={base32_address}"));

            if let Err(error) =
                std::fs::write(format!("{}/addresses", self.address_book_path), &*inner)
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    %host,
                    address = %base32_address,
                    error = ?error.kind(),
                    "failed to update on-disk base32 address list",
                );
            }
        }

        if let Err(error) = std::fs::write(
            format!("{}/destinations/{host}.txt", self.address_book_path),
            base64_encode(destination.serialize()),
        ) {
            tracing::warn!(
                target: LOG_TARGET,
                %host,
                error = ?error.kind(),
                "failed to write base64-encoded destination on disk",
            );
        }
        #[cfg(feature = "i2pcontrol")]
        if let Some(hook) = &self.runtime_hook {
            hook.legacy_publish(host, base64_encode(destination.serialize()));
        }
    }

    /// Remove `host` from address book.
    #[allow(dead_code)]
    pub fn remove(&self, host: &str) {
        self.addresses.write().remove(host);

        // update the on-disk version of the base32 address list
        {
            let mut inner = self.serialized.write();
            *inner = inner
                .lines()
                .filter(|line| !line.starts_with(&format!("{host}=")))
                .collect::<Vec<_>>()
                .join("\n");

            if let Err(error) =
                std::fs::write(format!("{}/addresses", self.address_book_path), &*inner)
            {
                tracing::warn!(
                    target: LOG_TARGET,
                    %host,
                    error = ?error.kind(),
                    "failed to update on-disk base32 address list",
                );
            }
        }

        // remove destination file for the host
        if let Err(error) = std::fs::remove_file(format!(
            "{}/destinations/{host}.txt",
            self.address_book_path
        )) {
            tracing::warn!(
                target: LOG_TARGET,
                %host,
                error = ?error.kind(),
                "failed to remove destination for host",
            );
        }
        #[cfg(feature = "i2pcontrol")]
        if let Some(hook) = &self.runtime_hook {
            hook.legacy_remove(host);
        }
    }
}

impl AddressBook for AddressBookHandle {
    fn resolve_base64(&self, host: String) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        let path = Arc::clone(&self.address_book_path);
        #[cfg(feature = "i2pcontrol")]
        let runtime_hook = self.runtime_hook.as_ref().map(Arc::clone);

        Box::pin(async move {
            #[cfg(feature = "i2pcontrol")]
            if let Some(hook) = runtime_hook {
                return hook.resolve_base64(&host);
            }
            if let Ok(destination) =
                tokio::fs::read_to_string(format!("{path}/destinations/{host}.txt")).await
            {
                return Some(destination);
            }
            None
        })
    }

    fn resolve_base32(&self, host: &str) -> Option<String> {
        #[cfg(feature = "i2pcontrol")]
        if let Some(hook) = &self.runtime_hook {
            return hook.resolve_base32(host);
        }
        self.addresses.read().get(host).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "i2pcontrol")]
    use crate::i2pcontrol::address_book_runtime::{
        base32_for_destination, new_controlled_manager, RuntimeAddressBookEntry,
        RuntimeAddressBookHandle, RuntimeAddressBookSnapshot, RuntimeAddressBookType,
    };
    #[cfg(feature = "i2pcontrol")]
    use std::{collections::BTreeMap, sync::atomic::Ordering};
    use tempfile::tempdir;

    #[cfg(feature = "i2pcontrol")]
    async fn controlled_manager(
        base_path: std::path::PathBuf,
        config: AddressBookConfig,
    ) -> (AddressBookManager, Arc<RuntimeAddressBookHandle>) {
        new_controlled_manager(base_path, config).await
    }

    #[cfg(feature = "i2pcontrol")]
    fn valid_destination(seed: u8) -> String {
        use emissary_core::crypto::SigningPrivateKey;
        use emissary_util::runtime::tokio::Runtime as TokioRuntime;

        let key = SigningPrivateKey::from_bytes(&[seed; 32]).unwrap();
        base64_encode(
            emissary_core::primitives::Destination::new::<TokioRuntime>(key.public()).serialize(),
        )
    }

    #[tokio::test]
    async fn save_only_destination() {
        let dir = tempdir().unwrap();
        let address_book = AddressBookManager::new(
            dir.keep(),
            AddressBookConfig {
                default: Some(String::from("url")),
                subscriptions: None,
            },
        )
        .await;

        let mut addresses = HashMap::<String, (String, String)>::new();
        let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

        address_book.parse_and_merge(&mut addresses, hosts).await;
        let addresses = addresses
            .into_iter()
            .map(|(hostname, (_, destination))| (hostname, destination))
            .collect::<HashMap<_, _>>();

        assert_eq!(
            addresses.get(&String::from("tracker2.postman.i2p")),
            Some(&String::from(
                    "lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDaJ50lUZg\
                    VPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaT\
                    Mn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQ\
                    US4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~\
                    sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfcKolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZ\
                    DU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA=="
                )
            )
        );

        assert_eq!(
            addresses.get(&String::from("psi.i2p")),
            Some(
                &String::from(
                    "a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH3\
                    3l4~mQjB8Hkt83l9qnNJPUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW\
                    9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpBZz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0Nt\
                    aOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPybmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MR\
                    VT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lzS9GWf2i-nk~~\
                    ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA"
                )
            )
        );

        assert_eq!(
            addresses.get(&String::from("zerobin.i2p")),
            Some(
                &String::from(
                    "Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVgDZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0Mj\
                    QDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8SWcxWdTbKU-O8IpE3O01Oc6j0fp1\
                    E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYhbfM7x56\
                    4PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZ\
                    nGKPEBD31hVLdJuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG\
                    0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYmAAAA"
                )
            )
        );

        assert_eq!(
            addresses.get(&String::from("zzz.i2p")),
            Some(
                &String::from(
                    "GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~\
                    YV8dmXl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0\
                    d8AdjB0E0864nOT~J9Fpnd2pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsd\
                    jU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~\
                    DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpne8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFF\
                    T3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA=="
                )
            )
        );
    }

    #[tokio::test]
    async fn host_lookup() {
        // create address book
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            dir
        };

        // load address book from disk
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = address_book.handle();

        // try to find base32 address and base64-encoded destination of a saved destination
        let host = handle.resolve_base32("zzz.i2p").expect("to find base32 address for zzz.i2p");
        assert_eq!(
            host,
            "lhbd7ojcaiofbfku7ixh47qj537g572zmhdc4oilvugzxdpdghua".to_string()
        );

        let destination = handle
            .resolve_base64("zzz.i2p".to_string())
            .await
            .expect("expected to find destination for zzz.i2p");
        assert_eq!(destination, "GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dmXl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpne8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==");

        // attempt to find base32 address for an unknown hostname
        assert!(handle.resolve_base32("test.i2p").is_none());
        assert!(handle.resolve_base64("test.i2p".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn save_to_disk() {
        let dir = tempdir().unwrap().path().to_owned();
        tokio::fs::create_dir_all(dir.join("addressbook")).await.unwrap();

        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: Some(String::from("url")),
                subscriptions: None,
            },
        )
        .await;

        let mut addresses = HashMap::<String, (String, String)>::new();
        let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

        address_book.parse_and_merge(&mut addresses, hosts).await;
        address_book.save_to_disk(addresses.clone()).await;

        // verify all base64 destinations have been saved under correct keys
        for (key, (_, value)) in addresses {
            let key = key.strip_suffix(".i2p").unwrap();

            let path = dir.join("addressbook/destinations").join(format!("{key}.i2p.txt"));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), value);
        }

        // verify all base32 addresses have been saved and correspond to correct hostnames
        let content = std::fs::read_to_string(dir.join("addressbook/addresses")).unwrap();
        let mut expected = HashMap::<&str, &str>::from_iter([
            (
                "psi.i2p",
                "avviiexdngd32ccoy4kuckvc3mkf53ycvzbz6vz75vzhv4tbpk5a",
            ),
            (
                "zerobin.i2p",
                "3564erslxzaoucqasxsjerk4jz2xril7j2cbzd4p7flpb4ut67hq",
            ),
            (
                "tracker2.postman.i2p",
                "6a4kxkg5wp33p25qqhgwl6sj4yh4xuf5b3p3qldwgclebchm3eea",
            ),
            (
                "zzz.i2p",
                "lhbd7ojcaiofbfku7ixh47qj537g572zmhdc4oilvugzxdpdghua",
            ),
        ]);

        for line in content.lines() {
            let (key, value) = line.split_once('=').unwrap();
            let expected_value = expected.remove(&key).unwrap();

            assert_eq!(expected_value, value);
        }
    }

    #[tokio::test]
    async fn redownload_address_book() {
        // create address book
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            dir
        };

        // load address book from disk
        //
        // this simulates a startup from disk
        {
            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: None,
                    subscriptions: None,
                },
            )
            .await;
            let handle = address_book.handle();

            // verify all hosts are found
            for host in ["psi.i2p", "zerobin.i2p", "tracker2.postman.i2p", "zzz.i2p"] {
                assert!(handle.resolve_base32(host).is_some());
                assert!(handle.resolve_base64(host.to_string()).await.is_some());
            }

            // download more hosts from some other subscriptions
            //
            // verify that the new destinations are stored on disk, theb base32 address book is
            // updated and that `AddressBookHandle` has the new base32 addresses
            let new_hosts = "echo.idk.i2p=75Mgd8DeYkNAdk6mrF0rOt3WuM6bYi3k7n3BRIp-np5-5-py1YTmBJSRq9EXL1YrweU\
                pal~ZT~46~-lMW~zKHA46iTWsZzKFMEKFdfoM2fSgZDlEaoalW4QzkQjbOVO0atkEK15uXO2WiCRgfUab0Gyp-EcGkdQq\
                8FuF~sPLlm4xa90cmT6G~s1SeaNLII8DPli-XqZppSGOSArMSEYsxZPNfKs5UxvEeCrbUjBLCGhy2K3x926Af9bZk1ITI\
                W~K70xo-rSD0SbDXO9vT4e7fiq21Q2LicAofn~Y4MwPz6jr0CUZQAHxwCy1N3OR0XYFsTI4XS1LZ3zGwVpEjCfVD1mQhC\
                LzzIjRHgGB6Cr3xj3FntBhv0d9pBuogUjOkLEnmFHpj5ONE7l~3aUsR6vRYTUDfwnxXnIi9KF-PPyOAACXCz~T06QXVfa\
                qzTj8tyr40cUqKIZHquaMZrAkqqa26KM3l0fu755UVo43MfD3f9zo3erjo~UDI~oP1bYyEU3LBQAEAAcAAA==\n3chele\
                ctricboogaloo.i2p=W5br4iApDNRQ-IY0yZQqrv-BqIj5Rk1XytcDUHcTpGHiyeaEPlP3wRr~zMTKrlCh78IRI26m8Gw\
                jE2VKdoMMwl9j9GdikQX4p4J0912IcRZbFGVIkibe36KgqkKJ9-s1gv4yzDVjEG2ZE6CMn-NZJxyp3a7Ypa1UFDLATdep\
                KFhHZRK3EN9fcUfMy2EAMwgihnd4JVZTytSVOcKfUrhIXf1lKmgr9qN9uXv38q2mtYMzbnWNe6TlB9GWrAZnsREB1kEZM\
                ts28leSGB~kYJHSlxQQvgOA0Rt9s3xkvP~EThUmyIIDZ9SlBh8iek6E1wreHfRhsCA9qTdCcs5XivxG0CMJR4F-Q55AlS\
                FWthsgJrUrb9x4RB6fTANv5bvZH3yBMQvZZxgqu4sN6MbPVaW2X7x3ldq3~UwG~HXvdvA7wP-sBq3uEshPCuVhJD8ojUD\
                qm9prLIVuLWt2pIiVbz-l4P-jx769A0OoN8-Z6j5b7zLNdd~QX0NPfXVMp1V~Z4fFBQAEAAcAAA==\ngts.varikvalef\
                or.i2p=cqwoYXfztQTzDKaPMsElmgfflDLLRBBvQSTgHfhBKnGkT-lKzL659h480A9YQuZfmKSzaplidYTvMPIrS4lhh4\
                m7ybvkt3Dv9bLd8QR7k14gw~arshKakr6HgkpUXU8uwA6ns45bsLKe~PbASIGbPEIudRjtrvmRpXxqy~0mpjkXkhhm8RJ\
                x3CM7iO62ZyspCXTI2GV2rbvcxyD6kEcQd-YrU5tnimSUN8b1WqbkoQBvEn1JR~mn-KJzR4RUpKd~FgqPesMI2rM8dyl4\
                bQkHv3XrWU3YZ75bsXaht80Ii6rrcPrD3LCERIC43rII0Y1UQarpij2ZyC508ccveIbGroUWOsrPz8tLklnIuvZIor~2M\
                kWK9e2zymXMPU~8ZJDVhAFd2PSJs~TFBrwvzD2mvcG-ChvrS051Gv1ZPIOqDLGdTEBC7BV2OyDlzKu0VnHsiS3h2PwBJd\
                Xx9zk~HXAA1hwLCjuIwSYpNIQxrEaFEu2Eh6keqo-kKidL~Lcxk~hUBQAEAAcAAA==".to_string();

            let mut addresses = HashMap::<String, (String, String)>::new();
            address_book.parse_and_merge(&mut addresses, new_hosts).await;
            address_book.save_to_disk(addresses).await;

            // verify all old and new hosts are found
            for host in [
                "psi.i2p",
                "zerobin.i2p",
                "tracker2.postman.i2p",
                "zzz.i2p",
                "echo.idk.i2p",
                "3chelectricboogaloo.i2p",
                "gts.varikvalefor.i2p",
            ] {
                assert!(handle.resolve_base32(host).is_some());
                assert!(handle.resolve_base64(host.to_string()).await.is_some());
            }
        }

        // load address book from disk again and verify all hosts are still in the address book
        {
            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: None,
                    subscriptions: None,
                },
            )
            .await;
            let handle = address_book.handle();

            for host in [
                "psi.i2p",
                "zerobin.i2p",
                "tracker2.postman.i2p",
                "zzz.i2p",
                "echo.idk.i2p",
                "3chelectricboogaloo.i2p",
                "gts.varikvalefor.i2p",
            ] {
                assert!(handle.resolve_base32(host).is_some());
                assert!(handle.resolve_base64(host.to_string()).await.is_some());
            }
        }
    }

    #[tokio::test]
    async fn add_base32_address() {
        // create address book
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            dir
        };

        // simulate new boot and add new base32 address to address book
        {
            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: None,
                    subscriptions: None,
                },
            )
            .await;
            let handle = address_book.handle();

            handle.add_base32(
                "i2pd.i2p".to_string(),
                "4bpcp4fmvyr46vb4kqjvtxlst6puz4r3dld24umooiy5mesxzspa".to_string(),
            );

            // verify that base32 address for i2pd is found
            assert_eq!(
                handle.resolve_base32("i2pd.i2p"),
                Some(String::from(
                    "4bpcp4fmvyr46vb4kqjvtxlst6puz4r3dld24umooiy5mesxzspa"
                ))
            );
        }

        // simulate another boot and verify i2pd.i2p has been stored on disk
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

        assert_eq!(
            address_book.handle().resolve_base32("i2pd.i2p"),
            Some(String::from(
                "4bpcp4fmvyr46vb4kqjvtxlst6puz4r3dld24umooiy5mesxzspa"
            ))
        );
    }

    #[tokio::test]
    async fn add_base64_address() {
        // create address book
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            dir
        };

        // simulate new boot and add new base32 address to address book
        let base64_destination = {
            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: None,
                    subscriptions: None,
                },
            )
            .await;
            let handle = address_book.handle();

            let base64_destination = "75Mgd8DeYkNAdk6mrF0rOt3WuM6bYi3k7n3BRIp-np5-5-py1YTmBJSRq9EXL1YrweU\
                pal~ZT~46~-lMW~zKHA46iTWsZzKFMEKFdfoM2fSgZDlEaoalW4QzkQjbOVO0atkEK15uXO2WiCRgfUab0Gyp-\
                EcGkdQq8FuF~sPLlm4xa90cmT6G~s1SeaNLII8DPli-XqZppSGOSArMSEYsxZPNfKs5UxvEeCrbUjBLCGhy2K3\
                x926Af9bZk1ITIW~K70xo-rSD0SbDXO9vT4e7fiq21Q2LicAofn~Y4MwPz6jr0CUZQAHxwCy1N3OR0XYFsTI4X\
                S1LZ3zGwVpEjCfVD1mQhCLzzIjRHgGB6Cr3xj3FntBhv0d9pBuogUjOkLEnmFHpj5ONE7l~3aUsR6vRYTUDfwn\
                xXnIi9KF-PPyOAACXCz~T06QXVfaqzTj8tyr40cUqKIZHquaMZrAkqqa26KM3l0fu755UVo43MfD3f9zo3erjo\
                ~UDI~oP1bYyEU3LBQAEAAcAAA==";
            let deserialized = base64_decode(base64_destination).unwrap();
            let destination = Destination::parse(&deserialized).unwrap();

            handle.add_base64("echo.idk.i2p".to_string(), destination);

            // verify that base32 address for echo.idk is found
            assert_eq!(
                handle.resolve_base32("echo.idk.i2p"),
                Some(String::from(
                    "63sgpiu6f33arldcxkbjsn3jgf6asyx3onjmz6j6gsk7hgbiehkq"
                ))
            );
            assert_eq!(
                handle.resolve_base64("echo.idk.i2p".to_string()).await,
                Some(base64_destination.to_string())
            );

            base64_destination
        };

        // simulate another boot and verify i2pd.i2p has been stored on disk
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = address_book.handle();

        assert_eq!(
            handle.resolve_base32("echo.idk.i2p"),
            Some(String::from(
                "63sgpiu6f33arldcxkbjsn3jgf6asyx3onjmz6j6gsk7hgbiehkq"
            ))
        );
        assert_eq!(
            handle.resolve_base64("echo.idk.i2p".to_string()).await,
            Some(base64_destination.to_string())
        );
    }

    #[tokio::test]
    async fn remove_host() {
        // create address book and remove zzz.i2p from it
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
                J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
                E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
                umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
                KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
                O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
                gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
                n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
                PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
                1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
                Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
                p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
                PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
                Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
                bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
                S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
                DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
                WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
                bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
                JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
                AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
                Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
                pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
                Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
                e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            let handle = address_book.handle();
            assert!(handle.resolve_base32("zzz.i2p").is_some());
            assert!(handle.resolve_base64("zzz.i2p".to_string()).await.is_some());

            // remove host and verify it's no longer found
            handle.remove("zzz.i2p");

            assert!(handle.resolve_base32("zzz.i2p").is_none());
            assert!(handle.resolve_base64("zzz.i2p".to_string()).await.is_none());

            dir
        };

        // simulate another boot and verify zzz.i2p is not found in the address book
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = address_book.handle();

        assert!(handle.resolve_base32("zzz.i2p").is_none());
        assert!(handle.resolve_base64("zzz.i2p".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn duplicate_entries_ignored() {
        // create address book
        let dir = {
            let dir = tempdir().unwrap().keep();
            tokio::fs::create_dir_all(&dir.join("addressbook")).await.unwrap();

            let address_book = AddressBookManager::new(
                dir.clone(),
                AddressBookConfig {
                    default: Some(String::from("url")),
                    subscriptions: None,
                },
            )
            .await;

            let mut addresses = HashMap::<String, (String, String)>::new();
            let hosts = "zerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVgDZRuaToRlXIFW4k5J1ccTzGzMxR518Bk\
                CAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8SWcxWdTbKU-O8IpE3O\
                01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYhbf\
                M7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZ\
                nGKPEBD31hVLdJuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l\
                ~BvPl2OKQIBhXIxJLOOavGyqmmYmAAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-e\
                OD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dmXl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH\
                6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6\
                Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6Uri3usE3XWD7Pw6P8qVYF39jUIq4\
                OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpne8cuesn3NDXIp\
                NnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

            address_book.parse_and_merge(&mut addresses, hosts).await;
            address_book.save_to_disk(addresses).await;

            dir
        };

        // simulate another boot and verify zzz.i2p is not found in the address book
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

        // read original values from disk and verify only zerobin.i2p and zzz.i2p exist
        let original_values =
            tokio::fs::read_to_string(dir.join("addressbook/addresses")).await.unwrap();

        assert_eq!(original_values.matches("tracker2.postman.i2p").count(), 0);
        assert_eq!(original_values.matches("psi.i2p").count(), 0);
        assert_eq!(original_values.matches("zerobin.i2p").count(), 1);
        assert_eq!(original_values.matches("zzz.i2p").count(), 1);

        // add new hosts, tracker2.postman.i2p is new and zerobin.i2p and zzz.i2p already
        // exist in the address book
        let mut addresses = HashMap::<String, (String, String)>::new();
        let hosts = "tracker2.postman.i2p=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHhn853zjVXrJBgwlB9sO57KakBDa\
            J50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~PUJQxRwceaTMn96FcVenwdXqle\
            E16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG1kOIBeDD3XF8BWb6K3GOOoyjc1\
            umYKpur3G~FxBuqtHAsDRICrsRuil8qK~whOvj8uNTv~ohZnTZHxTLgi~sDyo98BwJ-4Y4NMSuF4GLzcgLypcR1D1WY2tDqMKRYFVyLE~MTPVjRRgXfc\
            KolykQ666~Go~A~~CNV4qc~zlO6F4bsUhVZDU7WJ7mxCAwqaMiJsL-NgIkb~SMHNxIzaE~oy0agHJMBQAEAAcAAA==#!oldsig=i02RMv3Hy86NGhVo2\
            O3byIf6xXqWrzrRibSabe5dmNfRRQPZO9L25A==#date=1598641102#action=adddest#sig=cB-mY~sp1uuEmcQJqremV1D6EDWCe3IwPv4lBiGAX\
            gKRYc5MLBBzYvJXtXmOawpfLKeNM~v5fWlXYsDfKf5nDA==#olddest=lnQ6yoBTxQuQU8EQ1FlF395ITIQF-HGJxUeFvzETLFnoczNjQvKDbtSB7aHh\
            n853zjVXrJBgwlB9sO57KakBDaJ50lUZgVPhjlI19TgJ-CxyHhHSCeKx5JzURdEW-ucdONMynr-b2zwhsx8VQCJwCEkARvt21YkOyQDaB9IdV8aTAmP~\
            PUJQxRwceaTMn96FcVenwdXqleE16fI8CVFOV18jbJKrhTOYpTtcZKV4l1wNYBDwKgwPx5c0kcrRzFyw5~bjuAKO~GJ5dR7BQsL7AwBoQUS4k1lwoYrG\
            1kOIBeDD3XF8BWb6K3GOOoyjc1umYKpur3G~FxBuqtHAsDRICkEbKUqJ9mPYQlTSujhNxiRIW-oLwMtvayCFci99oX8MvazPS7~97x0Gsm-onEK1Td9n\
            Bdmq30OqDxpRtXBimbzkLbR1IKObbg9HvrKs3L-kSyGwTUmHG9rSQSoZEvFMA-S0EXO~o4g21q1oikmxPMhkeVwQ22VHB0-LZJfmLr4SAAAA\npsi.i2\
            p=a11l91etedRW5Kl2GhdDI9qiRBbDRAQY6TWJb8KlSc0P9WUrEviABAAltqDU1DFJrRhMAZg5i6rWGszkJrF-pWLQK9JOH33l4~mQjB8Hkt83l9qnNJ\
            PUlGlh9yIfBY40CQ0Ermy8gzjHLayUpypDJFv2V6rHLwxAQeaXJu8YXbyvCucEu9i6HVO49akXW9YSxcZEqxK04wZnjBqhHGlVbehleMqTx9nkd0pUpB\
            Zz~vIaG9matUSHinopEo6Wegml9FEz~FEaQpPknKuMAGGSNFVJb0NtaOQSAocAOg1nLKh80v232Y8sJOHG63asSJoBa6bGwjIHftsqD~lEmVV4NkgNPy\
            bmvsD1SCbMQ2ExaCXFPVQV-yJhIAPN9MRVT9cSBT2GCq-vpMwdJ5Nf0iPR3M-Ak961JUwWXPYTL79toXCgxDX2~nZ5QFRV490YNnfB7LQu10G89wG8lz\
            S9GWf2i-nk~~ez0Lq0dH7qQokFXdUkPc7bvSrxqkytrbd-h8O8AAAA\nzerobin.i2p=Jf64hlpW8ILKZGDe61ljHU5wzmUYwN2klOyhM2iR-8VkUEVg\
            DZRuaToRlXIFW4k5J1ccTzGzMxR518BkCAE3jCFIyrbF0MjQDuXO5cwmqfBFWrIv72xgKDizu3HytE4vOF2M730rv8epSNPAJg6OpyXkf5UQW96kgL8S\
            WcxWdTbKU-O8IpE3O01Oc6j0fp1E4wVOci7qIL8UEloNN~mulgka69MkR0uEtXWOXd6wvBjLNrZgdZi7XtT4QlDjx13jr7RGpZBJAUkk~8gLqgJwoUYh\
            bfM7x564PIn3IlMXHK5AKRVxAbCQ5GkS8KdkvNL7FsQ~EiElGzZId4wenraHMHL0destUDmuwGdHKA7YdtovXD~OnaBvIbl36iuIduZnGKPEBD31hVLd\
            JuVId9RND7lQy5BZJHQss5HSxMWTszAnWJDwmxqzMHHCiL6BMpZnkz8znwPDSkUwEs3P6-ba7mDKKt8EPCG0nM6l~BvPl2OKQIBhXIxJLOOavGyqmmYm\
            AAAA\nzzz.i2p=GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dm\
            Xl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2\
            pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6\
            Uri3usE3XWD7Pw6P8qVYF39jUIq4OiNMwPnNYzy2N4mDMQdsdHO3LUVh~DEppOy9AAmEoHDjjJxt2BFBbGxfdpZCpENkwvmZeYUyNCCzASqTOOlNzdpn\
            e8cuesn3NDXIpNnqEE6Oe5Qm5YOJykrX~Vx~cFFT3QzDGkIjjxlFBsjUJyYkFjBQAEAAcAAA==".to_string();

        address_book.parse_and_merge(&mut addresses, hosts).await;
        address_book.save_to_disk(addresses).await;

        // read updated values from disk, verify all hosts exist and that there aren't duplicate
        let new_values =
            tokio::fs::read_to_string(dir.join("addressbook/addresses")).await.unwrap();

        assert_eq!(new_values.matches("tracker2.postman.i2p").count(), 1);
        assert_eq!(new_values.matches("psi.i2p").count(), 1);
        assert_eq!(new_values.matches("zerobin.i2p").count(), 1);
        assert_eq!(new_values.matches("zzz.i2p").count(), 1);

        // verify `addresses` have also been updated
        assert!(address_book.addresses.read().get("tracker2.postman.i2p").is_some());
        assert!(address_book.addresses.read().get("psi.i2p").is_some());
        assert!(address_book.addresses.read().get("zerobin.i2p").is_some());
        assert!(address_book.addresses.read().get("zzz.i2p").is_some());

        // create new address book with updated entries and verify all hosts are there
        let address_book_new = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

        assert!(address_book_new.addresses.read().get("tracker2.postman.i2p").is_some());
        assert!(address_book_new.addresses.read().get("psi.i2p").is_some());
        assert!(address_book_new.addresses.read().get("zerobin.i2p").is_some());
        assert!(address_book_new.addresses.read().get("zzz.i2p").is_some());
    }

    #[tokio::test]
    async fn simple_save_load_host_modified() {
        let dir = tempdir().unwrap().path().to_owned();
        let addressbook_dir = dir.join("addressbook");
        tokio::fs::create_dir_all(addressbook_dir).await.unwrap();
        let address_book = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: Some(String::from("url")),
                subscriptions: None,
            },
        )
        .await;
        let loaded = HashMap::from_iter([(
            "foo".to_string(),
            Modified {
                etag: "bar".to_string(),
                last_modified: "baz".to_string(),
            },
        )]);
        address_book.save_host_modified_times(loaded.clone()).await;
        let saved = address_book.load_host_modified_times().await;
        assert_eq!(loaded, saved);
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn runtime_control_mutation_is_visible_and_restart_safe() {
        use emissary_core::crypto::SigningPrivateKey;
        use emissary_util::runtime::tokio::Runtime as TokioRuntime;

        let dir = tempdir().unwrap().keep();
        let destination = {
            let key = SigningPrivateKey::from_bytes(&[0xa; 32]).unwrap();
            base64_encode(
                emissary_core::primitives::Destination::new::<TokioRuntime>(key.public())
                    .serialize(),
            )
        };
        let expected_base32 = base32_encode(
            Destination::parse(base64_decode(&destination).unwrap()).unwrap().id().to_vec(),
        );
        let (manager, control) = controlled_manager(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = manager.handle();
        control
            .runtime_add(
                RuntimeAddressBookType::Private,
                RuntimeAddressBookEntry {
                    hostname: "runtime.i2p".to_string(),
                    destination: destination.clone(),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            control.runtime_list(RuntimeAddressBookType::Private).await.unwrap().len(),
            1
        );
        assert_eq!(
            handle.resolve_base64("runtime.i2p".to_string()).await,
            Some(destination)
        );
        assert_eq!(handle.resolve_base32("runtime.i2p"), Some(expected_base32));

        drop(manager);
        let (manager, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = manager.handle();
        assert!(handle.resolve_base32("runtime.i2p").is_some());
        assert!(handle.resolve_base64("runtime.i2p".to_string()).await.is_some());
        control
            .runtime_delete(RuntimeAddressBookType::Private, "runtime.i2p")
            .await
            .unwrap();
        assert!(handle.resolve_base32("runtime.i2p").is_none());
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn set_subscriptions_queue_failure_preserves_prior_state() {
        let dir = tempdir().unwrap().keep();
        let (_manager, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;

        let result = control
            .runtime_set_subscriptions(vec!["https://example.i2p/hosts.txt".to_string()])
            .await;
        assert!(result.is_err());
        assert!(control.runtime_subscriptions().await.unwrap().is_empty());
        assert!(control.runtime_active_subscriptions().is_empty());
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn set_subscriptions_worker_failure_after_commit_returns_success() {
        let dir = tempdir().unwrap().keep();
        let (manager, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let replacement = vec!["https://committed.example/hosts.txt".to_string()];
        let (response_sender, response_receiver) = oneshot::channel();
        let (refresh_sender, refresh_receiver) = mpsc::channel(1);
        drop(refresh_receiver);

        let mut refresh_busy = false;
        let mut pending_refresh = None;
        manager
            .handle_subscription_command(
                manager.runtime_hook.clone().unwrap(),
                RuntimeSubscriptionCommand {
                    subscriptions: replacement.clone(),
                    response: response_sender,
                },
                &mut refresh_busy,
                &mut pending_refresh,
                &refresh_sender,
            )
            .await;

        assert!(response_receiver.await.unwrap().is_ok());
        assert_eq!(control.runtime_active_subscriptions(), replacement);
        assert_eq!(control.runtime_subscriptions().await.unwrap(), replacement);
    }

    #[cfg(feature = "i2pcontrol")]
    async fn running_address_book_manager() -> (
        Arc<RuntimeAddressBookHandle>,
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let dir = tempdir().unwrap().keep();
        let (manager, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let proxy = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request).await;
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .await;
            }
        });
        let (ready_sender, ready_receiver) = oneshot::channel();
        let task = tokio::spawn(manager.run(port, "127.0.0.1".to_string(), ready_receiver));
        ready_sender.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            while !control.owner.subscription_control().started.load(Ordering::Acquire) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("address book subscription worker should start");
        (control, task, proxy)
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn set_subscriptions_updates_active_runtime_sources() {
        let (control, manager_task, proxy_task) = running_address_book_manager().await;
        let subscriptions = vec!["http://sub.example/hosts.txt".to_string()];
        let mut result = Err(String::new());
        for _ in 0..100 {
            result = control.runtime_set_subscriptions(subscriptions.clone()).await;
            if result.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        result.unwrap();
        assert_eq!(control.runtime_active_subscriptions(), subscriptions);
        assert_eq!(
            control.runtime_subscriptions().await.unwrap(),
            subscriptions
        );
        manager_task.abort();
        proxy_task.abort();
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn concurrent_subscription_replacements_publish_complete_generation() {
        let (control, manager_task, proxy_task) = running_address_book_manager().await;
        let first = vec!["http://first.example/hosts.txt".to_string()];
        let second = vec![
            "http://second.example/hosts.txt".to_string(),
            "https://third.example/hosts.txt".to_string(),
        ];
        let (first_result, second_result) = tokio::join!(
            control.runtime_set_subscriptions(first.clone()),
            control.runtime_set_subscriptions(second.clone()),
        );
        first_result.unwrap();
        second_result.unwrap();
        let active = control.runtime_active_subscriptions();
        assert!(active == first || active == second);
        assert_eq!(control.runtime_subscriptions().await.unwrap(), active);
        manager_task.abort();
        proxy_task.abort();
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn set_subscriptions_restart_restores_accepted_sources() {
        let dir = tempdir().unwrap().keep();
        let (manager, control) = controlled_manager(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        control
            .owner
            .mutate(|state| {
                state.subscriptions = vec!["https://restored.i2p/hosts.txt".to_string()];
                Ok(())
            })
            .await
            .unwrap();
        drop(manager);

        let (_restarted, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        assert_eq!(
            control.runtime_active_subscriptions(),
            vec!["https://restored.i2p/hosts.txt"]
        );
        assert_eq!(
            control.runtime_subscriptions().await.unwrap(),
            vec!["https://restored.i2p/hosts.txt"]
        );
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn runtime_owner_rejects_persistence_failure_without_runtime_change() {
        use emissary_core::crypto::SigningPrivateKey;
        use emissary_util::runtime::tokio::Runtime as TokioRuntime;

        let dir = tempdir().unwrap().keep();
        let stable_destination = {
            let key = SigningPrivateKey::from_bytes(&[0xb; 32]).unwrap();
            base64_encode(
                emissary_core::primitives::Destination::new::<TokioRuntime>(key.public())
                    .serialize(),
            )
        };
        let new_destination = {
            let key = SigningPrivateKey::from_bytes(&[0xc; 32]).unwrap();
            base64_encode(
                emissary_core::primitives::Destination::new::<TokioRuntime>(key.public())
                    .serialize(),
            )
        };
        let (manager, control) = controlled_manager(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = manager.handle();
        control
            .runtime_add(
                RuntimeAddressBookType::Private,
                RuntimeAddressBookEntry {
                    hostname: "stable.i2p".to_string(),
                    destination: stable_destination.clone(),
                },
            )
            .await
            .unwrap();

        let state_path = dir.join("addressbook/control-state.json");
        tokio::fs::remove_file(&state_path).await.unwrap();
        tokio::fs::create_dir(&state_path).await.unwrap();
        let result = control
            .runtime_update(
                RuntimeAddressBookType::Private,
                RuntimeAddressBookEntry {
                    hostname: "stable.i2p".to_string(),
                    destination: new_destination,
                },
            )
            .await;
        assert!(result.is_err());
        assert_eq!(
            handle.resolve_base64("stable.i2p".to_string()).await,
            Some(stable_destination)
        );
    }

    #[tokio::test]
    async fn inactive_address_book_ignores_and_preserves_control_state() {
        let dir = tempdir().unwrap().keep();
        let addressbook = dir.join("addressbook");
        tokio::fs::create_dir_all(&addressbook).await.unwrap();
        let control_state = addressbook.join("control-state.json");
        let stale_state = br#"{"private":{"stale.i2p":{"hostname":"stale.i2p","destination":"stale-destination"}},"local":{},"router":{},"published":{},"subscriptions":[],"configuration":{}}"#;
        tokio::fs::write(&control_state, stale_state).await.unwrap();

        let manager = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let handle = manager.handle();

        assert!(handle.resolve_base32("stale.i2p").is_none());
        assert_eq!(handle.resolve_base64("stale.i2p".to_string()).await, None);

        let downloaded = HashMap::from_iter([(
            "legacy.i2p".to_string(),
            (
                "legacy-base32".to_string(),
                "legacy-destination".to_string(),
            ),
        )]);
        manager.save_to_disk(downloaded).await;

        assert_eq!(tokio::fs::read(&control_state).await.unwrap(), stale_state);
        assert!(!addressbook.join(".control-state.json.tmp").exists());
        assert_eq!(
            handle.resolve_base32("legacy.i2p"),
            Some("legacy-base32".to_string())
        );
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn control_state_survives_disable_and_restores_on_reenable() {
        use emissary_core::crypto::SigningPrivateKey;
        use emissary_util::runtime::tokio::Runtime as TokioRuntime;

        let dir = tempdir().unwrap().keep();
        let retained_destination = {
            let key = SigningPrivateKey::from_bytes(&[0xd; 32]).unwrap();
            base64_encode(
                emissary_core::primitives::Destination::new::<TokioRuntime>(key.public())
                    .serialize(),
            )
        };
        let (manager, control) = controlled_manager(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        control
            .runtime_add(
                RuntimeAddressBookType::Private,
                RuntimeAddressBookEntry {
                    hostname: "retained.i2p".to_string(),
                    destination: retained_destination.clone(),
                },
            )
            .await
            .unwrap();
        let state_path = dir.join("addressbook/control-state.json");
        let retained = tokio::fs::read(&state_path).await.unwrap();
        drop(manager);

        let disabled = AddressBookManager::new(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let disabled_handle = disabled.handle();
        assert!(disabled_handle.resolve_base32("retained.i2p").is_none());
        assert_eq!(tokio::fs::read(&state_path).await.unwrap(), retained);
        drop(disabled);

        let (enabled, enabled_control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        let enabled_handle = enabled.handle();
        assert_eq!(
            enabled_control
                .runtime_lookup(RuntimeAddressBookType::Private, "retained.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            retained_destination
        );
        assert_eq!(
            enabled_handle.resolve_base64("retained.i2p".to_string()).await,
            Some(retained_destination)
        );
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn active_owner_update_and_delete_override_legacy_destination_file() {
        let dir = tempdir().unwrap().keep();
        let old_destination = valid_destination(20);
        let new_destination = valid_destination(21);
        let old_base32 = base32_for_destination(&old_destination);
        let new_base32 = base32_for_destination(&new_destination);
        let (manager, control) = controlled_manager(
            dir.clone(),
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        manager
            .save_to_disk(HashMap::from([(
                "coherent.i2p".to_string(),
                (old_base32, old_destination.clone()),
            )]))
            .await;
        let handle = manager.handle();

        control
            .runtime_update(
                RuntimeAddressBookType::Published,
                RuntimeAddressBookEntry {
                    hostname: "coherent.i2p".to_string(),
                    destination: new_destination.clone(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            handle.resolve_base64("coherent.i2p".to_string()).await,
            Some(new_destination)
        );
        assert_eq!(handle.resolve_base32("coherent.i2p"), Some(new_base32));

        assert!(control
            .runtime_delete(RuntimeAddressBookType::Published, "coherent.i2p")
            .await
            .unwrap());
        assert_eq!(
            handle.resolve_base64("coherent.i2p".to_string()).await,
            None
        );
        assert_eq!(handle.resolve_base32("coherent.i2p"), None);
        assert!(dir.join("addressbook/destinations/coherent.i2p.txt").exists());
    }

    #[cfg(feature = "i2pcontrol")]
    #[tokio::test]
    async fn active_download_repairs_a_persisted_base32_seed() {
        let dir = tempdir().unwrap().keep();
        let destination = valid_destination(22);
        let base32 = base32_for_destination(&destination);
        let state = RuntimeAddressBookSnapshot {
            published: BTreeMap::from([(
                "download.i2p".to_string(),
                RuntimeAddressBookEntry {
                    hostname: "download.i2p".to_string(),
                    destination: base32.clone(),
                },
            )]),
            ..RuntimeAddressBookSnapshot::default()
        };
        tokio::fs::create_dir_all(dir.join("addressbook")).await.unwrap();
        tokio::fs::write(
            dir.join("addressbook/control-state.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .await
        .unwrap();

        let (manager, control) = controlled_manager(
            dir,
            AddressBookConfig {
                default: None,
                subscriptions: None,
            },
        )
        .await;
        manager
            .save_to_disk(HashMap::from([(
                "download.i2p".to_string(),
                (base32, destination.clone()),
            )]))
            .await;
        assert_eq!(
            control
                .runtime_lookup(RuntimeAddressBookType::Published, "download.i2p")
                .await
                .unwrap()
                .unwrap()
                .destination,
            destination
        );
    }

    #[test]
    fn serde_json_is_feature_owned() {
        let manifest =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
        assert!(manifest.contains("serde_json = { version = \"1.0.140\", optional = true }"));
        assert!(manifest
            .lines()
            .any(|line| line.starts_with("i2pcontrol = [") && line.contains("\"serde_json\"")));
    }
}
