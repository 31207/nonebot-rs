use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemeInfo {
    pub key: String,
    pub params: MemeParams,
    pub keywords: Vec<String>,
    pub shortcuts: Vec<MemeShortcut>,
    pub tags: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemeParams {
    pub min_images: usize,
    pub max_images: usize,
    pub min_texts: usize,
    pub max_texts: usize,
    pub default_texts: Vec<String>,
    pub options: Vec<MemeOption>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemeShortcut {
    pub pattern: String,
    pub humanized: Option<String>,
    pub names: Vec<String>,
    pub texts: Vec<String>,
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MemeOption {
    #[serde(rename = "boolean")]
    Boolean {
        name: String,
        default: Option<bool>,
        description: Option<String>,
        parser_flags: ParserFlags,
    },

    #[serde(rename = "integer")]
    Integer {
        name: String,
        default: Option<i64>,
        description: Option<String>,
        parser_flags: ParserFlags,
        minimum: Option<i64>,
        maximum: Option<i64>,
    },

    #[serde(rename = "float")]
    Float {
        name: String,
        default: Option<f64>,
        description: Option<String>,
        parser_flags: ParserFlags,
        minimum: Option<f64>,
        maximum: Option<f64>,
    },

    #[serde(rename = "string")]
    String {
        name: String,
        default: Option<String>,
        description: Option<String>,
        parser_flags: ParserFlags,
        choices: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParserFlags {
    pub short: bool,
    pub long: bool,
    pub short_aliases: Vec<String>,
    pub long_aliases: Vec<String>,
}
