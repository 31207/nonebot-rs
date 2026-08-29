use crate::log::{colored::*, event, Level};
use config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// 全局 Debug 开关（由 NbConfig::load 设置）
static DEBUG: AtomicBool = AtomicBool::new(false);

/// 获取当前 Debug 开关状态
pub fn debug_enabled() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

/// nbrs 配置文件名
pub static CONFIG_PATH: &str = "Nonebotrs.toml";

/// nbrs 配置项结构体
#[derive(Serialize, Deserialize, Clone)]
pub struct NbConfig {
    /// 全局配置
    pub global: GlobalConfig,
    /// bot 配置
    pub bots: Option<HashMap<String, BotConfig>>,
    /// 反向 WS 服务器设置
    pub ws_server: Option<WebSocketServerConfig>,
    #[serde(skip)]
    config: Config, // save the full config
}

impl std::fmt::Debug for NbConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NbConfig")
            .field("Global", &self.global)
            .field("Bots", &self.bots)
            .finish()
    }
}

/// 反向 WS 服务器设置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketServerConfig {
    /// Host
    pub host: std::net::Ipv4Addr,
    /// Port
    pub port: u16,
    /// Onebot authorization
    #[serde(alias = "access-token")]
    #[serde(default)]
    access_token: String,
}

/// nbrs 全局配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    /// Debug 模式
    pub debug: bool,
    /// Trace 模式
    pub trace: Option<bool>,
    /// 事件通道容量，默认 1024
    #[serde(default)]
    pub event_channel_capacity: Option<usize>,
    /// 全局管理员账号设置
    pub superusers: Vec<String>,
    /// 全局昵称设置
    pub nicknames: Vec<String>,
    /// 全局命令起始符设置
    pub command_starts: Vec<String>,
}

/// nbrs bot 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotConfig {
    /// bot id
    #[serde(skip)]
    pub bot_id: String,
    /// 管理员账号设置
    #[serde(default)]
    pub superusers: Vec<String>,
    /// 昵称设置
    #[serde(default)]
    pub nicknames: Vec<String>,
    /// 命令起始符设置
    #[serde(default)]
    pub command_starts: Vec<String>,
    #[serde(alias = "access-token")]
    #[serde(default)]
    access_token: String, // Onebot authorization
    /// 正向 WS 地址
    #[serde(default)]
    pub ws_server: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            bot_id: String::new(),
            superusers: vec![],
            nicknames: vec![],
            command_starts: vec![],
            access_token: String::default(),
            ws_server: String::default(),
        }
    }
}

impl Default for NbConfig {
    fn default() -> Self {
        NbConfig {
            global: GlobalConfig {
                debug: true,
                trace: None,
                event_channel_capacity: None,
                superusers: vec![],
                nicknames: vec![],
                command_starts: vec!["/".to_string()],
            },
            bots: None,
            config: Config::default(),
            ws_server: Some(WebSocketServerConfig {
                host: std::net::Ipv4Addr::new(127, 0, 0, 1),
                port: 8088,
                access_token: String::default(),
            }),
        }
    }
}

impl NbConfig {
    /// 从配置文件读取配置
    pub fn load() -> Self {
        use colored::*;
        let mut config: NbConfig;
        let config_pathbuf = std::path::PathBuf::from(&CONFIG_PATH);
        if !config_pathbuf.exists() {
            config = NbConfig::default();
            let config_string = toml::to_string(&config).expect("Failed to serialize default config");
            std::fs::write(&config_pathbuf, &config_string).expect("Failed to write config file");
            println!("{}", "未发现配置文件，已新建配置文件。".green())
        } else {
            let mut _config = Config::default();
            _config.merge(config::File::with_name(CONFIG_PATH)).expect("Failed to merge config");
            config = _config.clone().try_into().expect("Failed to parse config");
            config.config = _config;
        }
        DEBUG.store(config.global.debug, Ordering::Relaxed);
        config
    }

    /// 根据 key_word 获取 config
    pub fn get_config<'de, T>(&self, key_word: &str) -> Option<T>
    where
        T: serde::Deserialize<'de>,
    {
        let _config = self.config.clone();
        let get_config: Result<T, config::ConfigError> = _config.get(key_word);
        match get_config {
            Ok(t) => {
                event!(Level::DEBUG, "Found config for {}", key_word);
                Some(t)
            }
            Err(_) => {
                event!(Level::DEBUG, "Not found config for {}", key_word);
                None
            }
        }
    }

    /// 获取 full config
    pub fn get_full_config(&self) -> Config {
        self.config.clone()
    }

    /// 生成 BotConfig
    pub fn gen_bot_config(&self, bot_id: &str) -> BotConfig {
        let mut rbotconfig = BotConfig {
            bot_id: bot_id.to_string(),
            superusers: self.global.superusers.clone(),
            nicknames: self.global.nicknames.clone(),
            command_starts: self.global.command_starts.clone(),
            access_token: String::default(),
            ws_server: String::default(),
        };

        if let Some(server_config) = &self.ws_server {
            rbotconfig.access_token = server_config.access_token.clone();
        }

        if let Some(bots_config) = &self.bots {
            if let Some(bot_config) = bots_config.get(bot_id) {
                if !bot_config.superusers.is_empty() {
                    rbotconfig.superusers = bot_config.superusers.clone();
                }
                if !bot_config.nicknames.is_empty() {
                    rbotconfig.nicknames = bot_config.nicknames.clone();
                }
                if !bot_config.command_starts.is_empty() {
                    rbotconfig.command_starts = bot_config.command_starts.clone();
                }
                if !bot_config.access_token.is_empty() {
                    rbotconfig.access_token = bot_config.access_token.clone();
                }
            }
        }
        rbotconfig
    }

    pub fn gen_access_token(&self) -> AccessToken {
        let mut at = AccessToken {
            global: if let Some(ws_server_config) = &self.ws_server {
                ws_server_config.access_token.clone()
            } else {
                String::default()
            },
            bots: HashMap::default(),
        };
        if let Some(bots) = &self.bots {
            for (bot_id, bot) in bots {
                if !bot.access_token.is_empty() {
                    at.bots
                        .insert(bot_id.to_string(), bot.access_token.to_string());
                }
            }
        }
        at
    }
}

#[derive(Clone)]
pub struct AccessToken {
    pub global: String,
    pub bots: HashMap<String, String>,
}

impl AccessToken {
    pub fn get(&self, bot_id: &str) -> &str {
        if let Some(a) = self.bots.get(bot_id) {
            a
        } else {
            &self.global
        }
    }

    pub fn check_auth(&self, bot_id: &str, token: Option<String>) -> bool {
        let access_token = if let Some(a) = self.bots.get(bot_id) {
            &a
        } else {
            &self.global
        };

        if access_token.is_empty() {
            return true;
        }

        fn check(head: &str, token: &str, access_token: &str) -> bool {
            if token.starts_with(head) {
                let token = crate::utils::remove_space(&token.replace(head, ""));
                if token == access_token {
                    return true;
                }
            }
            false
        }

        let mut result = false;
        if let Some(token) = &token {
            result = check("Token", token, access_token) || check("Bearer", &token, access_token)
        }

        if !result {
            event!(
                Level::WARN,
                "Access Token match fail Bot:[{}] Token:{:?}",
                bot_id.red(),
                token
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_nb_config() {
        let config = NbConfig::default();
        assert_eq!(config.global.debug, true);
        assert_eq!(config.global.trace, None);
        assert!(config.global.superusers.is_empty());
        assert_eq!(config.global.command_starts, vec!["/".to_string()]);
        assert!(config.ws_server.is_some());
    }

    #[test]
    fn test_gen_bot_config_merges_global() {
        let mut nb = NbConfig::default();
        nb.global.superusers = vec!["10001".to_string()];
        nb.global.nicknames = vec!["bot".to_string()];
        nb.global.command_starts = vec!["/".to_string(), "#".to_string()];

        let bot_config = nb.gen_bot_config("20001");
        assert_eq!(bot_config.bot_id, "20001");
        assert_eq!(bot_config.superusers, vec!["10001".to_string()]);
        assert_eq!(bot_config.nicknames, vec!["bot".to_string()]);
        assert_eq!(bot_config.command_starts, vec!["/".to_string(), "#".to_string()]);
    }

    #[test]
    fn test_access_token_check_empty() {
        let token = AccessToken {
            global: String::new(),
            bots: HashMap::new(),
        };
        assert!(token.check_auth("any_bot", None));
        assert!(token.check_auth("any_bot", Some("anything".to_string())));
    }

    #[test]
    fn test_access_token_check_bearer() {
        let token = AccessToken {
            global: "my_token".to_string(),
            bots: HashMap::new(),
        };
        assert!(token.check_auth("any_bot", Some("Bearer my_token".to_string())));
        assert!(token.check_auth("any_bot", Some("Token my_token".to_string())));
        assert!(!token.check_auth("any_bot", Some("wrong".to_string())));
    }
}
