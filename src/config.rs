use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::Digest;

// 服务类型
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Default)]
pub enum ServiceType {
    #[serde(rename = "Client")]
    Client,
    #[default]
    #[serde(rename = "Service")]
    Service,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Default)]
pub enum TransportType {
    #[default]
    #[serde(rename = "Tcp")]
    Tcp,

    #[serde(rename = "Udp")]
    Udp,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    #[serde(skip)]
    pub name: String,
    pub service_config: Option<ServiceConfig>,
    pub client_config: Option<ClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ClientServiceConfig {
    pub name: String,
    pub transport_type: TransportType,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    pub remote_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ClientServiceConfig>,
    pub heartbeat_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ServerServiceConfig {
    pub name: String,
    pub transport_type: TransportType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub bind_addr: String,
    pub default_token: Option<String>,
    pub services: HashMap<String, ServerServiceConfig>,
}
