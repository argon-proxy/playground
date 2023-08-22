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
    #[serde(rename = "type")]
    slot_type: String,
    workers: usize,
}
