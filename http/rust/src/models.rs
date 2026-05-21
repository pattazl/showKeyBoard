use serde::{Deserialize, Serialize};

// ============== Request Models ==============

#[derive(Debug, Deserialize)]
pub struct GetParaRequest {}

#[derive(Debug, Deserialize)]
pub struct SetParaRequest {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DataRequest {
    #[serde(rename = "appName")]
    pub app_name: Option<String>,
    #[serde(rename = "dataType")]
    pub data_type: Option<String>,
    pub data: Option<serde_json::Value>,
    pub tick: Option<i64>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PcInfoRequest {
    pub display: Option<String>,
    pub resolution: Option<String>,
    #[serde(rename = "cpuInfo")]
    pub cpu_info: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryDataRequest {
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub db: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MinuteDataRequest {
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "freqType")]
    pub freq_type: Option<String>,
    #[serde(rename = "isApp")]
    pub is_app: Option<bool>,
    pub db: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KeymapRequest {
    #[serde(rename = "optType")]
    pub opt_type: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDataRequest {
    #[serde(rename = "optType")]
    pub opt_type: Option<String>,
    pub id: Option<i64>,
    pub date: Option<String>,
    pub db: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppMinuteRequest {
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "isTotal")]
    pub is_total: Option<bool>,
    pub db: Option<String>,
}

// ============== Response Models ==============

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
    
    pub fn error(message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub msg: String,
    pub ver: String,
    #[serde(rename = "majorVersion")]
    pub major_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyRecord {
    pub id: Option<i64>,
    pub key: Option<String>,
    pub app: Option<String>,
    pub tick: Option<i64>,
    pub date: Option<String>,
    pub count: Option<i64>,
    #[serde(rename = "totalCount")]
    pub total_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppMinuteRecord {
    pub app: Option<String>,
    pub date: Option<String>,
    pub minutes: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryDate {
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatRecord {
    pub date: Option<String>,
    pub app: Option<String>,
    pub count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserKeymap {
    pub id: i64,
    pub name: String,
    pub data: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbInfo {
    pub name: String,
    pub path: String,
}

// ============== WebSocket Messages ==============

#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: Option<serde_json::Value>,
}
