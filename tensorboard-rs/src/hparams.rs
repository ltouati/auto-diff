use crate::status::JobStatus;
use protobuf::well_known_types::{ListValue, Value};
use protobuf::{Message, RepeatedField, SingularPtrField};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tensorboard_proto::api::{DataType, Experiment, HParamInfo, MetricInfo, MetricName};
use tensorboard_proto::plugin_hparams::{HParamsPluginData, SessionEndInfo, SessionStartInfo};
use tensorboard_proto::summary::{
    Summary, SummaryMetadata, SummaryMetadata_PluginData, Summary_Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericValue {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<GenericValue>),
    None,
}

impl From<GenericValue> for Value {
    fn from(value: GenericValue) -> Self {
        let mut ret = Value::new();
        match value {
            GenericValue::Number(n) => {
                ret.set_number_value(n);
            }
            GenericValue::String(s) => {
                ret.set_string_value(s);
            }
            GenericValue::Bool(b) => {
                ret.set_bool_value(b);
            }
            GenericValue::None => {}
            GenericValue::List(v) => {
                let v = v.iter().map(|v| v.clone().into()).collect::<Vec<Value>>();
                let mut list_value = ListValue::new();
                list_value.set_values(RepeatedField::from_vec(v));
                ret.set_list_value(list_value);
            }
        }
        ret
    }
}
impl From<&GenericValue> for Value {
    fn from(value: &GenericValue) -> Self {
        value.clone().into()
    }
}
#[derive(Debug, Clone)]
pub struct HyperParameter {
    name: String,
    value: GenericValue,
}

impl HyperParameter {
    
    pub fn as_kv(&self) -> (String,GenericValue) {
        (self.name.clone(),self.value.clone())
    } 
    
    pub fn with_values(name: &str, values: Vec<GenericValue>) -> Self {
        HyperParameter {
            name: name.to_string(),
            value: GenericValue::List(values),
        }
    }
    pub fn new(name: &str) -> Self {
        HyperParameter {
            name: name.to_string(),
            value: GenericValue::None,
        }
    }
    pub fn with_bool(name: &str, value: bool) -> Self {
        HyperParameter {
            name: name.to_string(),
            value: GenericValue::Bool(value),
        }
    }
    pub fn with_bools(name: &str) -> Self {
        let value = vec![GenericValue::Bool(true), GenericValue::Bool(false)];
        Self::with_values(name, value)
    }
    pub fn with_string(name: &str, value: &str) -> Self {
        HyperParameter {
            name: name.to_string(),
            value: GenericValue::String(value.to_string()),
        }
    }
    pub fn with_strings(name: &str, values: &[&str]) -> Self {
        let value = values
            .iter()
            .map(|v| GenericValue::String(v.to_string()))
            .collect::<Vec<_>>();
        Self::with_values(name, value)
    }
    pub fn with_f64(name: &str, value: f64) -> Self {
        HyperParameter {
            name: name.to_string(),
            value: GenericValue::Number(value),
        }
    }
    pub fn with_f64s(name: &str, values: &[f64]) -> Self {
        let value = values
            .iter()
            .map(|v| GenericValue::Number(*v))
            .collect::<Vec<_>>();
        Self::with_values(name, value)
    }
}
fn get_type_and_value(value: &GenericValue) -> (DataType, RepeatedField<Value>) {
    let (type_, value) = match value {
        GenericValue::Number(n) => {
            let mut value = Value::new();
            value.set_number_value(*n);
            (
                DataType::DATA_TYPE_FLOAT64,
                RepeatedField::from_vec(vec![value]),
            )
        }
        GenericValue::String(s) => {
            let mut value = Value::new();
            value.set_string_value(s.clone());
            (
                DataType::DATA_TYPE_STRING,
                RepeatedField::from_vec(vec![value]),
            )
        }
        GenericValue::Bool(b) => {
            let mut value = Value::new();
            value.set_bool_value(*b);
            (
                DataType::DATA_TYPE_BOOL,
                RepeatedField::from_vec(vec![value]),
            )
        }
        GenericValue::None => (DataType::DATA_TYPE_UNSET, RepeatedField::new()),
        GenericValue::List(l) => {
            if l.is_empty() {
                (DataType::DATA_TYPE_UNSET, RepeatedField::new())
            } else {
                let v = l.first().unwrap();
                let type_ = match v {
                    GenericValue::Number(_) => DataType::DATA_TYPE_FLOAT64,
                    GenericValue::String(_) => DataType::DATA_TYPE_STRING,
                    GenericValue::Bool(_) => DataType::DATA_TYPE_BOOL,
                    _ => {
                        panic!("Not supported")
                    }
                };
                let values = l.iter().flat_map(|v| get_type_and_value(v).1).collect();
                (type_, RepeatedField::from_vec(values))
            }
        }
    };
    (type_, value)
}
impl From<&HyperParameter> for HParamInfo {
    fn from(value: &HyperParameter) -> Self {
        let value = value.clone();
        let mut ret = HParamInfo::new();
        ret.set_name(value.name);

        let (type_, value) = get_type_and_value(&value.value);

        ret.set_field_type(type_);
        let mut list_value = ListValue::new();
        list_value.set_values(value);
        ret.set_domain_discrete(list_value);
        ret
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    name: String,
}

impl Metric {
    pub fn new(name: &str) -> Self {
        Metric {
            name: name.to_string(),
        }
    }
}

impl From<&Metric> for MetricInfo {
    fn from(value: &Metric) -> Self {
        let value = value.clone();
        let mut metric_name = MetricName::new();
        metric_name.set_tag(value.name);
        let mut ret = MetricInfo::new();
        ret.set_name(metric_name);
        ret
    }
}

const PLUGIN_NAME: &str = "hparams";
const PLUGIN_DATA_VERSION: i32 = 0;
fn create_summary_metadata(hparams_plugin_data_pb: &HParamsPluginData) -> SummaryMetadata {
    let mut content = HParamsPluginData::new();
    content.clone_from(hparams_plugin_data_pb);
    content.version = PLUGIN_DATA_VERSION;

    let mut summary_plugin_data = SummaryMetadata_PluginData::new();
    summary_plugin_data.set_content(content.write_to_bytes().unwrap());
    summary_plugin_data.set_plugin_name(PLUGIN_NAME.to_string());

    let mut ret = SummaryMetadata::new();
    ret.plugin_data = SingularPtrField::from_option(Some(summary_plugin_data));
    ret
}

fn sumary_pb(tag: &str, hparams_plugin_data: &HParamsPluginData) -> Summary {
    let mut summary = Summary::new();
    let summary_metadata = create_summary_metadata(hparams_plugin_data);
    let mut summary_value = Summary_Value::new();
    summary_value.set_tag(tag.to_string());
    summary_value.set_metadata(summary_metadata);
    summary.set_value(RepeatedField::from_vec(vec![summary_value]));
    summary
}

pub const EXPERIMENT_TAG: &str = "_hparams_/experiment";
pub const SESSION_START_INFO_TAG: &str = "_hparams_/session_start_info";
pub const SESSION_END_INFO_TAG: &str = "_hparams_/session_end_info";
pub fn hparams_config_pb(hparams: Vec<HParamInfo>, metrics: Vec<MetricInfo>) -> Summary {
    let mut experiment = Experiment::new();
    experiment.set_hparam_infos(RepeatedField::from(hparams));
    experiment.set_metric_infos(RepeatedField::from(metrics));
    let mut hparam_plugin_data = HParamsPluginData::new();
    hparam_plugin_data.set_experiment(experiment);
    sumary_pb(EXPERIMENT_TAG, &hparam_plugin_data)
}

pub fn hparams_config(hyper_parameters: &[HyperParameter], metrics: &[Metric]) -> Summary {
    let hyper_parameters = hyper_parameters
        .iter()
        .map(|h| h.into())
        .collect::<Vec<HParamInfo>>();
    let metrics = metrics
        .iter()
        .map(|h| h.into())
        .collect::<Vec<MetricInfo>>();
    hparams_config_pb(hyper_parameters, metrics)
}

fn derive_session_group_name(
    trial_id: Option<String>,
    hparams: &HashMap<String, GenericValue>,
) -> String {
    if let Some(trial_id) = trial_id {
        trial_id
    } else {
        let json_str = serde_json::to_string(&hparams).expect("Failed to serialize to JSON");
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

pub fn hparams(
    hparams: HashMap<String, GenericValue>,
    trial_id: Option<String>,
    start_time_secs: Option<i64>,
) -> Summary {
    let group_name = derive_session_group_name(trial_id, &hparams);
    let mut session_start_info = SessionStartInfo::new();
    session_start_info.set_group_name(group_name);
    if let Some(start_time_secs) = start_time_secs {
        session_start_info.set_start_time_secs(start_time_secs as f64);
    }
    let hparams = hparams
        .iter()
        .map(|(k, v)| {
            let value: Value = v.into();
            (k.clone(), value)
        })
        .collect::<HashMap<_, _>>();
    session_start_info.set_hparams(hparams);
    let mut hparams_plugin_data = HParamsPluginData::new();
    hparams_plugin_data.set_session_start_info(session_start_info);
    sumary_pb(SESSION_START_INFO_TAG, &hparams_plugin_data)
}

pub fn status_config(job_status: &JobStatus, end_time_secs: Option<i64>) -> Summary {
    let mut session_end_info = SessionEndInfo::new();
    if let Some(end_time_secs) = end_time_secs {
        session_end_info.set_end_time_secs(end_time_secs as f64);
    }
    session_end_info.set_status(job_status.clone().into());
    let mut hparams_plugin_data = HParamsPluginData::new();
    hparams_plugin_data.set_session_end_info(session_end_info);
    sumary_pb(SESSION_END_INFO_TAG, &hparams_plugin_data)
}
