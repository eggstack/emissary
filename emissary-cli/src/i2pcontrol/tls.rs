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

use std::{
    fs,
    io::BufReader,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use rcgen::{CertificateParams, KeyPair};
use rustls_pemfile::Item;
use tokio_rustls::rustls::{
    crypto::ring,
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};
use tracing;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use super::errors::I2pControlError;

const LOG_TARGET: &str = "emissary::i2pcontrol::tls";

/// Managed certificate directory name under the base path.
const MANAGED_CERT_DIR: &str = "i2pcontrol-certs";

/// Managed certificate filename.
const CERT_FILE: &str = "cert.pem";

/// Managed private key filename.
const KEY_FILE: &str = "key.pem";

/// TLS configuration for I2PControl.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Optional explicit certificate path.
    pub certificate: Option<PathBuf>,
    /// Optional explicit private key path.
    pub private_key: Option<PathBuf>,
}

impl TlsConfig {
    /// Returns true if this config provides explicit TLS material paths.
    pub fn is_explicit(&self) -> bool {
        self.certificate.is_some() || self.private_key.is_some()
    }

    /// Returns true only when both explicit certificate and private-key
    /// paths are present.
    ///
    /// M129 uses this to decide whether a non-loopback bind may proceed to
    /// explicit TLS loading. Partial material (certificate-only or
    /// key-only) never satisfies the remote-service requirement and must
    /// fail before any managed-TLS side effect.
    pub fn has_complete_explicit_material(&self) -> bool {
        self.certificate.is_some() && self.private_key.is_some()
    }
}

/// Load or generate TLS material and build a `ServerConfig`.
///
/// If explicit paths are provided, loads from those paths.
/// Otherwise, generates or reuses a managed self-signed certificate under `base_path`.
pub fn build_tls_config(
    tls: &TlsConfig,
    base_path: &Path,
) -> Result<Arc<ServerConfig>, I2pControlError> {
    let (certs, key) = if tls.is_explicit() {
        load_explicit_tls(tls)?
    } else {
        load_or_generate_managed_tls(base_path)?
    };

    let config = ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|e| I2pControlError::Tls(format!("TLS config error: {e}")))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| I2pControlError::Tls(format!("TLS cert/key error: {e}")))?;

    Ok(Arc::new(config))
}

/// Load TLS material from explicit operator-provided paths.
fn load_explicit_tls(
    tls: &TlsConfig,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), I2pControlError> {
    let cert_path = tls
        .certificate
        .as_ref()
        .ok_or_else(|| I2pControlError::Tls("Certificate path required".into()))?;
    let key_path = tls
        .private_key
        .as_ref()
        .ok_or_else(|| I2pControlError::Tls("Private key path required".into()))?;

    let certs = load_certs(cert_path)?;
    let key = load_key(key_path)?;

    Ok((certs, key))
}

/// Load or generate a managed self-signed certificate.
pub fn load_or_generate_managed_tls(
    base_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), I2pControlError> {
    let cert_dir = base_path.join(MANAGED_CERT_DIR);
    let cert_path = cert_dir.join(CERT_FILE);
    let key_path = cert_dir.join(KEY_FILE);

    ensure_managed_directory(&cert_dir)?;
    let cert_exists = validate_managed_file(&cert_path, "certificate")?;
    let key_exists = validate_managed_file(&key_path, "private key")?;
    if key_exists {
        ensure_managed_private_key_permissions(&key_path)?;
    }

    // Try to load existing certificate material
    if cert_exists && key_exists {
        match load_certs(&cert_path).and_then(|certs| load_key(&key_path).map(|key| (certs, key))) {
            Ok(result) => {
                tracing::info!(
                    target: LOG_TARGET,
                    "loaded existing managed TLS certificate",
                );
                return Ok(result);
            }
            Err(e) => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?e,
                    "existing TLS material invalid, regenerating",
                );
            }
        }
    }

    // Generate new self-signed certificate
    generate_managed_tls(&cert_dir, &cert_path, &key_path)
}

/// Generate a new self-signed certificate and save it.
fn generate_managed_tls(
    cert_dir: &Path,
    cert_path: &Path,
    key_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), I2pControlError> {
    ensure_managed_directory(cert_dir)?;
    validate_managed_file(cert_path, "certificate")?;
    validate_managed_file(key_path, "private key")?;

    let key_pair = KeyPair::generate()
        .map_err(|e| I2pControlError::Tls(format!("Failed to generate key pair: {e}")))?;

    let mut params = CertificateParams::new(vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ])
        .map_err(|e| I2pControlError::Tls(format!("Failed to create cert params: {e}")))?;

    params.distinguished_name.push(rcgen::DnType::CommonName, "Emissary I2PControl");

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| I2pControlError::Tls(format!("Failed to sign certificate: {e}")))?;

    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der = PrivateKeyDer::try_from(key_pair.serialize_der())
        .map_err(|e| I2pControlError::Tls(format!("Failed to serialize key: {e}")))?;

    // Publish each file through a same-directory temporary file. Renaming a temporary file
    // replaces a regular target without following a target symlink, while the explicit checks
    // reject pre-existing links and special files before any managed material is read or changed.
    write_managed_file(cert_path, cert.der(), "certificate")?;
    write_managed_file(key_path, &key_pair.serialize_der(), "private key")?;

    tracing::info!(
        target: LOG_TARGET,
        ?cert_path,
        "generated managed self-signed TLS certificate",
    );

    Ok((vec![cert_der], key_der))
}

fn ensure_managed_directory(path: &Path) -> Result<(), I2pControlError> {
    let existed = match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(I2pControlError::Tls(
                    "Managed TLS certificate directory is not a regular directory".into(),
                ));
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(I2pControlError::Tls(format!(
                "Failed to inspect managed TLS certificate directory: {error}"
            )))
        }
    };

    if !existed {
        fs::create_dir_all(path)
            .map_err(|e| I2pControlError::Tls(format!("Failed to create cert directory: {e}")))?;
    }

    #[cfg(unix)]
    restrict_managed_directory_permissions(path)?;

    validate_managed_directory(path)?;
    Ok(())
}

fn validate_managed_directory(path: &Path) -> Result<(), I2pControlError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|e| I2pControlError::Tls(format!("Failed to inspect cert directory: {e}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(I2pControlError::Tls(
            "Managed TLS certificate directory is not a regular directory".into(),
        ));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(I2pControlError::Tls(
            "Managed TLS certificate directory permissions are not owner-only".into(),
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn restrict_managed_directory_permissions(path: &Path) -> Result<(), I2pControlError> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| {
        I2pControlError::Tls(format!(
            "Failed to restrict cert directory permissions: {e}"
        ))
    })?;
    validate_managed_directory(path)
}

fn validate_managed_file(path: &Path, label: &str) -> Result<bool, I2pControlError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(I2pControlError::Tls(format!(
                    "Managed TLS {label} path is a symlink"
                )));
            }
            if !metadata.is_file() {
                return Err(I2pControlError::Tls(format!(
                    "Managed TLS {label} path is not a regular file"
                )));
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(I2pControlError::Tls(format!(
            "Failed to inspect managed TLS {label}: {error}"
        ))),
    }
}

fn ensure_managed_private_key_permissions(path: &Path) -> Result<(), I2pControlError> {
    if !validate_managed_file(path, "private key")? {
        return Ok(());
    }

    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            I2pControlError::Tls(format!("Failed to restrict managed TLS private key: {e}"))
        })?;
        let metadata = fs::symlink_metadata(path).map_err(|e| {
            I2pControlError::Tls(format!("Failed to inspect managed TLS private key: {e}"))
        })?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o777 != 0o600
        {
            return Err(I2pControlError::Tls(
                "Managed TLS private key permissions are not owner-only".into(),
            ));
        }
    }

    Ok(())
}

fn open_managed_file(path: &Path, label: &str) -> Result<fs::File, std::io::Error> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if label == "private key" {
        options.mode(0o600);
    }
    options.open(path)
}

fn write_managed_file(path: &Path, bytes: &[u8], label: &str) -> Result<(), I2pControlError> {
    let _ = validate_managed_file(path, label)?;
    let temporary = path.with_extension("tmp");
    if let Ok(metadata) = fs::symlink_metadata(&temporary) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(I2pControlError::Tls(format!(
                "Managed TLS {label} temporary path is not a regular file"
            )));
        }
        fs::remove_file(&temporary).map_err(|e| {
            I2pControlError::Tls(format!("Failed to replace managed TLS {label}: {e}"))
        })?;
    }

    let mut file = open_managed_file(&temporary, label)
        .map_err(|e| I2pControlError::Tls(format!("Failed to create managed TLS {label}: {e}")))?;
    file.write_all(bytes).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        I2pControlError::Tls(format!("Failed to write managed TLS {label}: {e}"))
    })?;
    if label == "private key" {
        ensure_managed_private_key_permissions(&temporary).inspect_err(|_| {
            let _ = fs::remove_file(&temporary);
        })?;
    }
    file.sync_all().map_err(|e| {
        let _ = fs::remove_file(&temporary);
        I2pControlError::Tls(format!("Failed to sync managed TLS {label}: {e}"))
    })?;
    if let Err(error) = validate_managed_file(path, label) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    #[cfg(not(unix))]
    if fs::symlink_metadata(path).is_ok() {
        fs::remove_file(path).map_err(|e| {
            let _ = fs::remove_file(&temporary);
            I2pControlError::Tls(format!("Failed to replace managed TLS {label}: {e}"))
        })?;
    }
    fs::rename(&temporary, path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        I2pControlError::Tls(format!("Failed to publish managed TLS {label}: {e}"))
    })?;
    Ok(())
}

/// Load certificates from a file (tries DER first, then PEM).
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, I2pControlError> {
    let data = fs::read(path)
        .map_err(|e| I2pControlError::Tls(format!("Failed to open certificate file: {e}")))?;

    // Try PEM first (has header)
    if data.starts_with(b"-----") {
        let mut reader = BufReader::new(data.as_slice());
        let mut certs = Vec::new();
        for item in rustls_pemfile::read_all(&mut reader).flatten() {
            if let Item::X509Certificate(cert) = item {
                certs.push(cert);
            }
        }
        if !certs.is_empty() {
            return Ok(certs);
        }
    }

    // Default: treat as DER
    let cert = CertificateDer::from(data);
    Ok(vec![cert])
}

/// Load a private key from a file (tries DER first, then PEM).
fn load_key(path: &Path) -> Result<PrivateKeyDer<'static>, I2pControlError> {
    let data = fs::read(path)
        .map_err(|e| I2pControlError::Tls(format!("Failed to open private key file: {e}")))?;

    // Try PEM first (has header)
    if data.starts_with(b"-----") {
        let mut reader = BufReader::new(data.as_slice());
        for item in rustls_pemfile::read_all(&mut reader).flatten() {
            match item {
                Item::Pkcs1Key(k) => return Ok(PrivateKeyDer::Pkcs1(k)),
                Item::Sec1Key(k) => return Ok(PrivateKeyDer::Sec1(k)),
                Item::Pkcs8Key(k) => return Ok(PrivateKeyDer::Pkcs8(k)),
                _ => {}
            }
        }
    }

    // Default: treat as PKCS8 DER (rcgen generates PKCS8)
    Ok(PrivateKeyDer::Pkcs8(data.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn managed_tls_generates_and_loads() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        // First call generates
        let (certs1, _) = load_or_generate_managed_tls(base).unwrap();
        assert!(!certs1.is_empty());

        // Second call loads same material
        let (certs2, _) = load_or_generate_managed_tls(base).unwrap();
        assert_eq!(certs1[0].as_ref(), certs2[0].as_ref());
    }

    #[test]
    fn managed_tls_recovers_from_invalid_cert() {
        let dir = tempdir().unwrap();
        let base = dir.path();
        let cert_dir = base.join(MANAGED_CERT_DIR);
        fs::create_dir_all(&cert_dir).unwrap();

        // Write invalid cert
        fs::write(cert_dir.join(CERT_FILE), "not a cert").unwrap();
        fs::write(cert_dir.join(KEY_FILE), "not a key").unwrap();

        // Should regenerate
        let result = load_or_generate_managed_tls(base);
        assert!(result.is_ok());
    }

    #[test]
    fn tls_config_is_explicit() {
        let c1 = TlsConfig {
            certificate: None,
            private_key: None,
        };
        assert!(!c1.is_explicit());

        let c2 = TlsConfig {
            certificate: Some(PathBuf::from("/cert")),
            private_key: None,
        };
        assert!(c2.is_explicit());
    }

    #[test]
    fn tls_complete_material_requires_both_paths() {
        let none = TlsConfig {
            certificate: None,
            private_key: None,
        };
        assert!(!none.has_complete_explicit_material());

        let cert_only = TlsConfig {
            certificate: Some(PathBuf::from("/cert")),
            private_key: None,
        };
        assert!(!cert_only.has_complete_explicit_material());

        let key_only = TlsConfig {
            certificate: None,
            private_key: Some(PathBuf::from("/key")),
        };
        assert!(!key_only.has_complete_explicit_material());

        let complete = TlsConfig {
            certificate: Some(PathBuf::from("/cert")),
            private_key: Some(PathBuf::from("/key")),
        };
        assert!(complete.has_complete_explicit_material());
    }

    #[cfg(unix)]
    #[test]
    fn managed_private_key_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        load_or_generate_managed_tls(dir.path()).unwrap();
        let mode = fs::metadata(dir.path().join(MANAGED_CERT_DIR).join(KEY_FILE))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        let directory_mode =
            fs::metadata(dir.path().join(MANAGED_CERT_DIR)).unwrap().permissions().mode() & 0o777;
        assert_eq!(directory_mode, 0o700);
    }

    #[cfg(unix)]
    #[test]
    fn permissive_managed_material_is_repaired_and_reused() {
        let dir = tempdir().unwrap();
        let cert_dir = dir.path().join(MANAGED_CERT_DIR);
        load_or_generate_managed_tls(dir.path()).unwrap();
        let cert_before = fs::read(cert_dir.join(CERT_FILE)).unwrap();
        let key_before = fs::read(cert_dir.join(KEY_FILE)).unwrap();

        fs::set_permissions(&cert_dir, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(cert_dir.join(KEY_FILE), fs::Permissions::from_mode(0o644)).unwrap();

        load_or_generate_managed_tls(dir.path()).unwrap();
        assert_eq!(fs::read(cert_dir.join(CERT_FILE)).unwrap(), cert_before);
        assert_eq!(fs::read(cert_dir.join(KEY_FILE)).unwrap(), key_before);
        assert_eq!(
            fs::metadata(&cert_dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(cert_dir.join(KEY_FILE)).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let cert_after_repair = fs::read(cert_dir.join(CERT_FILE)).unwrap();
        let key_after_repair = fs::read(cert_dir.join(KEY_FILE)).unwrap();
        load_or_generate_managed_tls(dir.path()).unwrap();
        assert_eq!(
            fs::read(cert_dir.join(CERT_FILE)).unwrap(),
            cert_after_repair
        );
        assert_eq!(fs::read(cert_dir.join(KEY_FILE)).unwrap(), key_after_repair);
    }

    #[cfg(unix)]
    #[test]
    fn managed_private_key_requests_owner_only_mode_at_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(KEY_FILE);
        let _file = open_managed_file(&path, "private key").unwrap();

        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[cfg(unix)]
    #[test]
    fn managed_symlinks_fail_closed_without_touching_targets() {
        use std::os::unix::fs::symlink;

        for (file, target_name) in [(KEY_FILE, "outside-key"), (CERT_FILE, "outside-cert")] {
            let dir = tempdir().unwrap();
            let cert_dir = dir.path().join(MANAGED_CERT_DIR);
            fs::create_dir_all(&cert_dir).unwrap();
            let target = dir.path().join(target_name);
            fs::write(&target, b"operator material").unwrap();
            symlink(&target, cert_dir.join(file)).unwrap();

            let error = load_or_generate_managed_tls(dir.path()).unwrap_err();
            assert!(error.to_string().contains("symlink"));
            assert_eq!(fs::read(target).unwrap(), b"operator material");
        }
    }

    #[tokio::test]
    async fn managed_certificate_validates_all_loopback_server_names() {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
        use tokio::net::TcpListener;
        use tokio_rustls::rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
        use tokio_rustls::{TlsAcceptor, TlsConnector};

        let dir = tempdir().unwrap();
        let (certs, key) = load_or_generate_managed_tls(dir.path()).unwrap();
        let server_config = Arc::new(
            ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(certs.clone(), key)
                .unwrap(),
        );
        let mut roots = RootCertStore::empty();
        roots.add(certs[0].clone()).unwrap();
        let client_config = Arc::new(
            ClientConfig::builder_with_provider(Arc::new(ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        );

        for server_name in [
            ServerName::try_from("localhost").unwrap(),
            ServerName::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST).into()),
            ServerName::IpAddress(IpAddr::V6(Ipv6Addr::LOCALHOST).into()),
        ] {
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
            let address = listener.local_addr().unwrap();
            let acceptor = TlsAcceptor::from(Arc::clone(&server_config));
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                acceptor.accept(stream).await.unwrap();
            });
            let connector = TlsConnector::from(Arc::clone(&client_config));
            connector
                .connect(server_name, tokio::net::TcpStream::connect(address).await.unwrap())
                .await
                .unwrap();
            server.await.unwrap();
        }
    }
}
