// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Keylime Authors

use crate::{error::Error, permissions, tpm};
use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment,
    File, FileFormat, Map, Source, Value,
};
use glob::glob;
use keylime::algorithms::{
    EncryptionAlgorithm, HashAlgorithm, SignAlgorithm,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub static CONFIG_VERSION: &str = "2.0";
pub static DEFAULT_UUID: &str = "d432fbb3-d2f1-4a97-9ef7-75bd81c00000";
pub static DEFAULT_IP: &str = "127.0.0.1";
pub static DEFAULT_PORT: u32 = 9002;
pub static DEFAULT_CONTACT_IP: &str = "127.0.0.1";
pub static DEFAULT_CONTACT_PORT: u32 = 9002;
pub static DEFAULT_REGISTRAR_IP: &str = "127.0.0.1";
pub static DEFAULT_REGISTRAR_PORT: u32 = 8890;
pub static DEFAULT_ENABLE_AGENT_MTLS: bool = true;
pub static DEFAULT_KEYLIME_DIR: &str = "/var/lib/keylime";
pub static DEFAULT_SERVER_KEY: &str = "server-private.pem";
pub static DEFAULT_SERVER_CERT: &str = "server-cert.crt";
pub static DEFAULT_SERVER_KEY_PASSWORD: &str = "";
// The DEFAULT_TRUSTED_CLIENT_CA is relative from KEYLIME_DIR
pub static DEFAULT_TRUSTED_CLIENT_CA: &str = "cv_ca/cacert.crt";
pub static DEFAULT_ENC_KEYNAME: &str = "derived_tci_key";
pub static DEFAULT_DEC_PAYLOAD_FILE: &str = "decrypted_payload";
pub static DEFAULT_SECURE_SIZE: &str = "1m";
pub static DEFAULT_TPM_OWNERPASSWORD: &str = "";
pub static DEFAULT_EXTRACT_PAYLOAD_ZIP: bool = true;
pub static DEFAULT_ENABLE_REVOCATION_NOTIFICATIONS: bool = true;
pub static DEFAULT_REVOCATION_ACTIONS_DIR: &str = "/usr/libexec/keylime";
pub static DEFAULT_REVOCATION_NOTIFICATION_IP: &str = "127.0.0.1";
pub static DEFAULT_REVOCATION_NOTIFICATION_PORT: u32 = 8992;
// Note: The revocation certificate name is generated inside the Python tenant and the
// certificate(s) can be generated by running the tenant with the --cert flag. For more
// information, check the README: https://github.com/keylime/keylime/#using-keylime-ca
pub static DEFAULT_REVOCATION_CERT: &str = "RevocationNotifier-cert.crt";
pub static DEFAULT_REVOCATION_ACTIONS: &str = "";
pub static DEFAULT_PAYLOAD_SCRIPT: &str = "autorun.sh";
pub static DEFAULT_ENABLE_INSECURE_PAYLOAD: bool = false;
pub static DEFAULT_ALLOW_PAYLOAD_REVOCATION_ACTIONS: bool = true;
pub static DEFAULT_TPM_HASH_ALG: &str = "sha256";
pub static DEFAULT_TPM_ENCRYPTION_ALG: &str = "rsa";
pub static DEFAULT_TPM_SIGNING_ALG: &str = "rsassa";
pub static DEFAULT_EK_HANDLE: &str = "generate";
pub static DEFAULT_RUN_AS: &str = "keylime:tss";
pub static DEFAULT_AGENT_DATA_PATH: &str = "agent_data.json";
pub static DEFAULT_CONFIG: &str = "/etc/keylime/agent.conf";
pub static DEFAULT_CONFIG_SYS: &str = "/usr/etc/keylime/agent.conf";

impl Source for KeylimeConfig {
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let agent: Map<String, Value> = Map::from([
            ("version".to_string(), self.agent.version.to_string().into()),
            ("uuid".to_string(), self.agent.uuid.to_string().into()),
            ("ip".to_string(), self.agent.ip.to_string().into()),
            ("port".to_string(), Value::from(self.agent.port)),
            (
                "contact_ip".to_string(),
                if let Some(ref s) = self.agent.contact_ip {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "contact_port".to_string(),
                if let Some(ref s) = self.agent.contact_port {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "registrar_ip".to_string(),
                self.agent.registrar_ip.to_string().into(),
            ),
            (
                "registrar_port".to_string(),
                self.agent.registrar_port.into(),
            ),
            (
                "enable_agent_mtls".to_string(),
                self.agent.enable_agent_mtls.into(),
            ),
            (
                "keylime_dir".to_string(),
                self.agent.keylime_dir.to_string().into(),
            ),
            (
                "server_key".to_string(),
                if let Some(ref s) = self.agent.server_key {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "server_key_password".to_string(),
                if let Some(ref s) = self.agent.server_key_password {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "server_cert".to_string(),
                if let Some(ref s) = self.agent.server_cert {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "trusted_client_ca".to_string(),
                if let Some(ref s) = self.agent.trusted_client_ca {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "enc_keyname".to_string(),
                self.agent.enc_keyname.to_string().into(),
            ),
            (
                "dec_payload_file".to_string(),
                self.agent.dec_payload_file.to_string().into(),
            ),
            (
                "secure_size".to_string(),
                self.agent.secure_size.to_string().into(),
            ),
            (
                "tpm_ownerpassword".to_string(),
                if let Some(ref s) = self.agent.tpm_ownerpassword {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "extract_payload_zip".to_string(),
                self.agent.extract_payload_zip.into(),
            ),
            (
                "enable_revocation_notifications".to_string(),
                self.agent.enable_revocation_notifications.into(),
            ),
            (
                "revocation_actions_dir".to_string(),
                if let Some(ref s) = self.agent.revocation_actions_dir {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "revocation_notification_ip".to_string(),
                if let Some(ref s) = self.agent.revocation_notification_ip {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "revocation_notification_port".to_string(),
                if let Some(ref s) = self.agent.revocation_notification_port {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "revocation_cert".to_string(),
                if let Some(ref s) = self.agent.revocation_cert {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "revocation_actions".to_string(),
                if let Some(ref s) = self.agent.revocation_actions {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "payload_script".to_string(),
                self.agent.payload_script.to_string().into(),
            ),
            (
                "enable_insecure_payload".to_string(),
                self.agent.enable_insecure_payload.to_string().into(),
            ),
            (
                "allow_payload_revocation_actions".to_string(),
                self.agent.allow_payload_revocation_actions.into(),
            ),
            (
                "tpm_hash_alg".to_string(),
                self.agent.tpm_hash_alg.to_string().into(),
            ),
            (
                "tpm_encryption_alg".to_string(),
                self.agent.tpm_encryption_alg.to_string().into(),
            ),
            (
                "tpm_signing_alg".to_string(),
                self.agent.tpm_signing_alg.to_string().into(),
            ),
            (
                "ek_handle".to_string(),
                if let Some(ref s) = self.agent.ek_handle {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "run_as".to_string(),
                if let Some(ref s) = self.agent.run_as {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
            (
                "agent_data_path".to_string(),
                if let Some(ref s) = self.agent.agent_data_path {
                    s.to_string().into()
                } else {
                    "".into()
                },
            ),
        ]);

        Ok(Map::from([("agent".to_string(), agent.into())]))
    }

    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct KeylimeConfig {
    pub agent: AgentConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct AgentConfig {
    pub version: String,
    pub uuid: String,
    pub ip: String,
    pub port: u32,
    pub contact_ip: Option<String>,
    pub contact_port: Option<u32>,
    pub registrar_ip: String,
    pub registrar_port: u32,
    pub enable_agent_mtls: bool,
    pub keylime_dir: String,
    pub server_key: Option<String>,
    pub server_cert: Option<String>,
    pub server_key_password: Option<String>,
    pub trusted_client_ca: Option<String>,
    pub enc_keyname: String,
    pub dec_payload_file: String,
    pub secure_size: String,
    pub tpm_ownerpassword: Option<String>,
    pub extract_payload_zip: bool,
    pub enable_revocation_notifications: bool,
    pub revocation_actions_dir: Option<String>,
    pub revocation_notification_ip: Option<String>,
    pub revocation_notification_port: Option<u32>,
    pub revocation_cert: Option<String>,
    pub revocation_actions: Option<String>,
    pub payload_script: String,
    pub enable_insecure_payload: bool,
    pub allow_payload_revocation_actions: bool,
    pub tpm_hash_alg: String,
    pub tpm_encryption_alg: String,
    pub tpm_signing_alg: String,
    pub ek_handle: Option<String>,
    pub run_as: Option<String>,
    pub agent_data_path: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        // In case the process is executed by privileged user
        let run_as = if permissions::get_euid() == 0 {
            Some(DEFAULT_RUN_AS.to_string())
        } else {
            None
        };

        AgentConfig {
            version: CONFIG_VERSION.to_string(),
            ip: DEFAULT_IP.to_string(),
            port: DEFAULT_PORT,
            registrar_ip: DEFAULT_REGISTRAR_IP.to_string(),
            registrar_port: DEFAULT_REGISTRAR_PORT,
            uuid: DEFAULT_UUID.to_string(),
            contact_ip: Some(DEFAULT_CONTACT_IP.to_string()),
            contact_port: Some(DEFAULT_CONTACT_PORT),
            tpm_hash_alg: DEFAULT_TPM_HASH_ALG.to_string(),
            tpm_encryption_alg: DEFAULT_TPM_ENCRYPTION_ALG.to_string(),
            tpm_signing_alg: DEFAULT_TPM_SIGNING_ALG.to_string(),
            agent_data_path: Some("default".to_string()),
            enable_revocation_notifications:
                DEFAULT_ENABLE_REVOCATION_NOTIFICATIONS,
            revocation_cert: Some("default".to_string()),
            revocation_notification_ip: Some(
                DEFAULT_REVOCATION_NOTIFICATION_IP.to_string(),
            ),
            revocation_notification_port: Some(
                DEFAULT_REVOCATION_NOTIFICATION_PORT,
            ),
            secure_size: DEFAULT_SECURE_SIZE.to_string(),
            payload_script: DEFAULT_PAYLOAD_SCRIPT.to_string(),
            dec_payload_file: DEFAULT_DEC_PAYLOAD_FILE.to_string(),
            enc_keyname: DEFAULT_ENC_KEYNAME.to_string(),
            extract_payload_zip: DEFAULT_EXTRACT_PAYLOAD_ZIP,
            server_key: Some("default".to_string()),
            server_key_password: Some(
                DEFAULT_SERVER_KEY_PASSWORD.to_string(),
            ),
            server_cert: Some("default".to_string()),
            trusted_client_ca: Some("default".to_string()),
            revocation_actions: Some(DEFAULT_REVOCATION_ACTIONS.to_string()),
            revocation_actions_dir: Some(
                DEFAULT_REVOCATION_ACTIONS_DIR.to_string(),
            ),
            allow_payload_revocation_actions:
                DEFAULT_ALLOW_PAYLOAD_REVOCATION_ACTIONS,
            keylime_dir: DEFAULT_KEYLIME_DIR.to_string(),
            enable_agent_mtls: DEFAULT_ENABLE_AGENT_MTLS,
            enable_insecure_payload: DEFAULT_ENABLE_INSECURE_PAYLOAD,
            run_as,
            tpm_ownerpassword: Some(DEFAULT_TPM_OWNERPASSWORD.to_string()),
            ek_handle: Some(DEFAULT_EK_HANDLE.to_string()),
        }
    }
}

impl Default for KeylimeConfig {
    fn default() -> Self {
        let c = KeylimeConfig {
            agent: AgentConfig::default(),
        };

        // The default config should never fail to translate keywords
        config_translate_keywords(&c).unwrap() //#[allow_ci]
    }
}

fn config_get_file_setting() -> Result<ConfigBuilder<DefaultState>, Error> {
    let default_config = KeylimeConfig::default();

    Ok(Config::builder()
        // Default values
        .add_source(default_config)
        // Add system configuration file
        .add_source(
            File::new(DEFAULT_CONFIG_SYS, FileFormat::Toml).required(false),
        )
        // Add system configuration snippets
        .add_source(
            glob("/usr/etc/keylime/agent.conf.d/*")
                .map_err(Error::GlobPattern)?
                .filter_map(|entry| entry.ok())
                .map(|path| {
                    File::new(&path.display().to_string(), FileFormat::Toml)
                        .required(false)
                })
                .collect::<Vec<_>>(),
        )
        .add_source(
            File::new(DEFAULT_CONFIG, FileFormat::Toml).required(false),
        )
        // Add user configuration snippets
        .add_source(
            glob("/etc/keylime/agent.conf.d/*")
                .map_err(Error::GlobPattern)?
                .filter_map(|entry| entry.ok())
                .map(|path| {
                    File::new(&path.display().to_string(), FileFormat::Toml)
                        .required(false)
                })
                .collect::<Vec<_>>(),
        )
        // Add environment variables overrides
        .add_source(
            Environment::with_prefix("KEYLIME")
                .separator("_")
                .prefix_separator("_"),
        ))
}

fn config_get_setting() -> Result<ConfigBuilder<DefaultState>, Error> {
    if let Ok(env_cfg) = env::var("KEYLIME_AGENT_CONFIG") {
        if !env_cfg.is_empty() {
            let path = Path::new(&env_cfg);
            if (path.exists()) {
                return Ok(Config::builder()
                    .add_source(
                        File::new(&env_cfg, FileFormat::Toml).required(true),
                    )
                    // Add environment variables overrides
                    .add_source(
                        Environment::with_prefix("KEYLIME")
                            .prefix_separator("_")
                            .separator("_"),
                    ));
            } else {
                warn!("Configuration set in KEYLIME_AGENT_CONFIG environment variable not found");
                return Err(Error::Configuration("Configuration set in KEYLIME_AGENT_CONFIG environment variable not found".to_string()));
            }
        }
    }
    config_get_file_setting()
}

/// Replace the options that support keywords with the final value
fn config_translate_keywords(
    config: &KeylimeConfig,
) -> Result<KeylimeConfig, Error> {
    let uuid = get_uuid(&config.agent.uuid);

    let env_keylime_dir = env::var("KEYLIME_DIR").ok();
    let keylime_dir = match env_keylime_dir {
        Some(ref dir) => {
            if !dir.is_empty() {
                dir.to_string()
            } else {
                config.agent.keylime_dir.to_string()
            }
        }
        None => config.agent.keylime_dir.to_string(),
    };

    let mut agent_data_path = config_get_file_path(
        &config.agent.agent_data_path,
        &keylime_dir,
        DEFAULT_AGENT_DATA_PATH,
    );

    let mut server_key = config_get_file_path(
        &config.agent.server_key,
        &keylime_dir,
        DEFAULT_SERVER_KEY,
    );

    let mut server_cert = config_get_file_path(
        &config.agent.server_cert,
        &keylime_dir,
        DEFAULT_SERVER_CERT,
    );

    let mut trusted_client_ca = config_get_file_path(
        &config.agent.trusted_client_ca,
        &keylime_dir,
        DEFAULT_TRUSTED_CLIENT_CA,
    );

    let mut revocation_cert = config_get_file_path(
        &config.agent.revocation_cert,
        &keylime_dir,
        &format!("secure/unzipped/{DEFAULT_REVOCATION_CERT}"),
    );

    let tpm_ownerpassword = match config.agent.tpm_ownerpassword {
        Some(ref s) => {
            if s.as_str() != "generate" {
                Some(s.to_string())
            } else {
                None
            }
        }
        None => None,
    };

    let ek_handle = match config.agent.ek_handle {
        Some(ref s) => {
            if s.as_str() != "generate" {
                Some(s.to_string())
            } else {
                None
            }
        }
        None => None,
    };

    // Validate the configuration

    // If mTLS is enabled, the trusted client CA certificate is required
    if config.agent.enable_agent_mtls
        && config.agent.trusted_client_ca.is_none()
    {
        error!("The option 'enable_agent_mtls' is set as 'true' but no certificate was set in 'trusted_client_ca' option");
        return Err(Error::Configuration(
                "The option 'enable_agent_mtls' is set as 'true' but no certificate was set in 'trusted_client_ca' option".to_string()));
    }

    // If revocation notifications is enabled, verify all the required options for revocation
    if config.agent.enable_revocation_notifications {
        if config.agent.revocation_notification_ip.is_none() {
            error!("The option 'enable_revocation_notifications' is set as 'true' but no IP was set in 'revocation_notification_ip'");
            return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but no IP was set in 'revocation_notification_ip'".to_string()));
        }
        if config.agent.revocation_notification_port.is_none() {
            error!("The option 'enable_revocation_notifications' is set as 'true' but no port was set in 'revocation_notification_port'");
            return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but no port was set in 'revocation_notification_port'".to_string()));
        }
        if config.agent.revocation_cert.is_none() {
            error!("The option 'enable_revocation_notifications' is set as 'true' but no certificate was set in 'revocation_cert'");
            return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but no certificate was set in 'revocation_notification_cert'".to_string()));
        }
        let actions_dir = match config.agent.revocation_actions_dir {
            Some(ref dir) => dir.to_string(),
            None => {
                error!("The option 'enable_revocation_notifications' is set as 'true' but the revocation actions directory was not set in 'revocation_actions_dir'");
                return Err(Error::Configuration("The option 'enable_revocation_notifications' is set as 'true' but the revocation actions directory was not set in 'revocation_actions_dir'".to_string()));
            }
        };
    }

    Ok(KeylimeConfig {
        agent: AgentConfig {
            keylime_dir,
            uuid,
            server_key,
            server_cert,
            trusted_client_ca,
            tpm_ownerpassword,
            ek_handle,
            agent_data_path,
            revocation_cert,
            ..config.agent.clone()
        },
    })
}

impl KeylimeConfig {
    pub fn new() -> Result<Self, Error> {
        // Get the base configuration file from the environment variable or the default locations
        let setting = config_get_setting()?.build()?;
        let config: KeylimeConfig = setting.try_deserialize()?;

        // Replace keywords with actual values
        config_translate_keywords(&config)
    }
}

/// Expand a file path from the configuration file.
///
/// If the option is None, return None
/// If the option string is empty, return None
/// If the option string is set as "default", return the provided default path relative from the provided work_dir.
/// If the option string is a relative path, return the path relative from the provided work_dir
/// If the option string is an absolute path, return the path without change.
fn config_get_file_path(
    path: &Option<String>,
    work_dir: &str,
    default: &str,
) -> Option<String> {
    if let Some(ref value) = path {
        if value == "default" {
            return Some(
                Path::new(work_dir).join(default).display().to_string(),
            );
        } else if value.is_empty() {
            return None;
        } else {
            let value = Path::new(&value);
            if value.is_relative() {
                return Some(
                    Path::new(work_dir).join(value).display().to_string(),
                );
            } else {
                return Some(value.display().to_string());
            }
        }
    }
    None
}

fn get_uuid(agent_uuid_config: &str) -> String {
    match agent_uuid_config {
        "openstack" => {
            info!("Openstack placeholder...");
            "openstack".into()
        }
        "hash_ek" => {
            info!("Using hashed EK as UUID");
            // DO NOT change this to something else. It is used later to set the correct value.
            "hash_ek".into()
        }
        "generate" => {
            let agent_uuid = Uuid::new_v4();
            info!("Generated a new UUID: {}", &agent_uuid);
            agent_uuid.to_string()
        }
        uuid_config => match Uuid::parse_str(uuid_config) {
            Ok(uuid_config) => uuid_config.to_string(),
            Err(_) => {
                info!("Misformatted UUID: {}", &uuid_config);
                let agent_uuid = Uuid::new_v4();
                agent_uuid.to_string()
            }
        },
    }
}

// Unit Testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let default = KeylimeConfig::default();
    }

    #[test]
    fn get_revocation_cert_path_default() {
        let test_config = KeylimeConfig::default();
        let revocation_cert_path =
            (test_config.agent.revocation_cert.clone()).unwrap(); //#[allow_ci]
        let mut expected = Path::new(&test_config.agent.keylime_dir)
            .join("secure/unzipped")
            .join(DEFAULT_REVOCATION_CERT)
            .display()
            .to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_cert_path_absolute() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: Some(String::from("/test/cert.crt")),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        let revocation_cert_path =
            (test_config.agent.revocation_cert).unwrap(); //#[allow_ci]
        let mut expected = Path::new("/test/cert.crt").display().to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_cert_path_relative() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: Some(String::from("cert.crt")),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        let revocation_cert_path =
            (test_config.agent.revocation_cert.clone()).unwrap(); //#[allow_ci]
        let mut expected = Path::new(&test_config.agent.keylime_dir)
            .join("cert.crt")
            .display()
            .to_string();
        assert_eq!(revocation_cert_path, expected);
    }

    #[test]
    fn get_revocation_cert_path_empty() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: Some(String::from("")),
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        assert_eq!(test_config.agent.revocation_cert, None);
    }

    #[test]
    fn get_revocation_cert_path_none() {
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                revocation_cert: None,
                ..Default::default()
            },
        };
        let result = config_translate_keywords(&test_config);
        // Due to enable_revocation_notifications being set
        assert!(result.is_err());
        let mut test_config = KeylimeConfig {
            agent: AgentConfig {
                enable_revocation_notifications: false,
                revocation_cert: None,
                ..Default::default()
            },
        };

        // Now unset enable_revocation_notifications and check that is allowed
        let result = config_translate_keywords(&test_config);
        assert!(result.is_ok());
        let test_config = result.unwrap(); //#[allow_ci]
        assert_eq!(test_config.agent.revocation_cert, None);
    }

    #[test]
    fn test_get_uuid() {
        assert_eq!(get_uuid("openstack"), "openstack");
        assert_eq!(get_uuid("hash_ek"), "hash_ek");
        let _ = Uuid::parse_str(&get_uuid("generate")).unwrap(); //#[allow_ci]
        assert_eq!(
            get_uuid("D432FBB3-D2F1-4A97-9EF7-75BD81C00000"),
            "d432fbb3-d2f1-4a97-9ef7-75bd81c00000"
        );
        assert_ne!(
            get_uuid("D432FBB3-D2F1-4A97-9EF7-75BD81C0000X"),
            "d432fbb3-d2f1-4a97-9ef7-75bd81c0000X"
        );
        let _ = Uuid::parse_str(&get_uuid(
            "D432FBB3-D2F1-4A97-9EF7-75BD81C0000X",
        ))
        .unwrap(); //#[allow_ci]
    }
}
