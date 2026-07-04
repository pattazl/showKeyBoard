use serde::{Deserialize, Serialize};

// ============== Request Models ==============

#[derive(Debug, Deserialize)]
pub struct GetParaRequest {}

#[derive(Debug, Deserialize)]
pub struct SetParaRequest {
    pub config: Option<serde_json::Value>,
    #[serde(rename = "keyList")]
    pub key_list: Option<serde_json::Value>,
    #[serde(rename = "dataSetting")]
    pub data_setting: Option<std::collections::HashMap<String, String>>,
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
pub struct MinuteDataRequest {
    #[serde(rename = "beginDate")]
    pub begin_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "freqType")]
    pub freq_type: Option<serde_json::Value>,
    #[serde(rename = "isApp")]
    pub is_app: Option<bool>,
    pub db: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KeymapRequest {
    pub flag: Option<i32>,
    #[serde(rename = "mapName")]
    pub map_name: Option<String>,
    #[serde(rename = "mapDetail")]
    pub map_detail: Option<String>,
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

/// Node.js getRecords response: { keyname, keycount, date, tick }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryRecord {
    pub keyname: String,
    pub keycount: i64,
    pub date: String,
    #[serde(default)]
    pub tick: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppMinuteRecord {
    pub app: Option<String>,
    pub date: Option<String>,
    pub minutes: Option<i64>,
}

/// Node.js /minuteData response when isApp != true (statFreq table)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinuteStatRecord {
    #[serde(rename = "Distance")]
    pub distance: Option<i32>,
    #[serde(rename = "KeyCount")]
    pub key_count: Option<i32>,
    #[serde(rename = "Minute")]
    pub minute: Option<String>,
    #[serde(rename = "MouseCount")]
    pub mouse_count: Option<i32>,
    #[serde(rename = "Date")]
    pub date: Option<String>,
}

/// Node.js /minuteData response when isApp == true (appFreq table)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinuteAppRecord {
    #[serde(rename = "Apps")]
    pub apps: Option<String>,
    #[serde(rename = "KeyCount")]
    pub key_count: Option<i32>,
    #[serde(rename = "Minute")]
    pub minute: Option<String>,
    #[serde(rename = "MouseCount")]
    pub mouse_count: Option<i32>,
    #[serde(rename = "Date")]
    pub date: Option<String>,
    #[serde(rename = "Duration")]
    pub duration: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserKeymap {
    #[serde(rename = "mapName")]
    pub map_name: String,
    #[serde(rename = "mapDetail")]
    pub map_detail: String,
}

/// Keymap in {mapName, mapDetail} format for getPara compatibility
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeymapBrief {
    #[serde(rename = "mapName")]
    pub map_name: String,
    #[serde(rename = "mapDetail")]
    pub map_detail: String,
}

// ============== getPara Response ==============

#[derive(Debug, Serialize)]
pub struct ParaResponse {
    pub config: ConfigSection,
    #[serde(rename = "keyList")]
    pub key_list: std::collections::HashMap<String, String>,
    pub fonts: Vec<String>,
    #[serde(rename = "infoPC")]
    pub info_pc: serde_json::Value,
    #[serde(rename = "dataSetting")]
    pub data_setting: std::collections::HashMap<String, String>,
    pub keymaps: Vec<KeymapBrief>,
    #[serde(rename = "networkIP")]
    pub network_ip: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigSection {
    pub common: std::collections::HashMap<String, String>,
    pub dialog: std::collections::HashMap<String, String>,
}

// ============== WebSocket Messages ==============

#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: Option<serde_json::Value>,
}
