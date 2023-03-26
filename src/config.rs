use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct AgencyService {
    pub transport_type: TransportType,
    pub port: usize
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
    pub local_addr: String,
    pub name: String,
    pub service: AgencyService,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    pub service_name: String,
    pub remote_addr: String,
    pub services: HashMap<String, ClientServiceConfig>,
    pub heartbeat_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub bind_addr: String,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorAckType {
    CommandError
}

impl From<ErrorAckType> for u32 {
    fn from(value: ErrorAckType) -> Self {
        match value {
            ErrorAckType::CommandError => 0 as u32
        }
    }
}