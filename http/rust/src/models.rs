use serde::{Deserialize, Serialize};

// ============== Request Models ==============

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Matches Node.js deleteDataFun: { flag: number, date: [array] }
#[derive(Debug, Deserialize)]
pub struct DeleteDataRequest {
    pub flag: Option<i64>,
    pub date: Option<Vec<String>>,
}

// ============== Response Models ==============

/// Node.js getRecords response: { keyname, keycount, date, tick }
/// tick: Some(n) → number, None → "" (matches Node.js behavior)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryRecord {
    pub keyname: String,
    pub keycount: i64,
    pub date: String,
    #[serde(default, serialize_with = "ser_tick", deserialize_with = "de_tick")]
    pub tick: Option<i64>,
}

fn ser_tick<S>(tick: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match tick {
        Some(n) => serializer.serialize_i64(*n),
        None => serializer.serialize_str(""),
    }
}

fn de_tick<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct TickVisitor;
    impl<'de> serde::de::Visitor<'de> for TickVisitor {
        type Value = Option<i64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("number, string or null")
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> { Ok(Some(v)) }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> { Ok(Some(v as i64)) }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> { Ok(Some(v as i64)) }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.parse::<i64>().ok())
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E> { Ok(None) }
        fn visit_none<E>(self) -> Result<Self::Value, E> { Ok(None) }
        fn visit_some<D2>(self, d: D2) -> Result<Self::Value, D2::Error>
        where D2: serde::Deserializer<'de> {
            d.deserialize_any(TickVisitor)
        }
    }
    deserializer.deserialize_any(TickVisitor)
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
