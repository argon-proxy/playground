use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ArgonConfig {
    pub tun: ArgonTunConfig,
    pub rack: ArgonRackConfig,
    pub slots: HashMap<String, ArgonSlotConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgonTunConfig {
    pub mtu: u16,
}

impl Default for ArgonTunConfig {
    fn default() -> Self {
        Self { mtu: 1420 }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgonRackConfig {
    pub channel_size: usize,
    pub layout: Vec<ArgonSlotLayoutConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgonSlotLayoutConfig {
    pub slot: String,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sink: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subslots: Vec<Self>,
}

impl Default for ArgonRackConfig {
    fn default() -> Self {
        Self {
            channel_size: 4096,
            layout: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgonSlotConfig {
    plugin: String,

    #[serde(default, skip_serializing_if = "ArgonRuntimeType::is_sync")]
    runtime: ArgonRuntimeType,

    workers: usize,

    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    config: serde_json::Value,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgonRuntimeType {
    #[default]
    Sync,

    Async,
}

impl ArgonRuntimeType {
    fn is_sync(&self) -> bool {
        match self {
            Self::Sync => true,
            Self::Async => false,
        }
    }
}
