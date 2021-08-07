use serde::{de, Deserialize, Deserializer, Serialize};
use tera::{Context, Result as TeraResult};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub(crate) load_options: LoadOptions,
    preprocess: Preprocess,
    #[serde(deserialize_with = "de_from_method_str")]
    pub(crate) method: MethodType,
    method_details: MethodDetails,
    #[serde(default = "default_trigger")]
    before_evaluate: Option<StringSequence>,
    evaluate: Option<Evaluate>,
    #[serde(default = "default_trigger")]
    after_evaluate: Option<StringSequence>,
}

impl Config {
    pub fn insert_into_ctx(&self) -> TeraResult<Context> {
        let mut ctx = Context::from_serialize(&self.load_options).unwrap();
        let ctx2 = Context::from_serialize(&self.preprocess).unwrap();
        let ctx3 = Context::from_serialize(&self.method_details).unwrap();
        let ctx4 = Context::from_serialize(&self.evaluate).unwrap();
        ctx.extend(ctx2);
        ctx.extend(ctx3);
        ctx.extend(ctx4);
        ctx.insert("bet", &self.before_evaluate);
        ctx.insert("aet", &self.after_evaluate);
        ctx.insert("methodname", &self.method);
        Ok(ctx)
    }
}

fn de_from_method_str<'de, D>(deserializer: D) -> Result<MethodType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match &*s {
        "wft" => Ok(MethodType::WFTMethod),
        "fft" => Ok(MethodType::FFTMethod),
        "spp" => Ok(MethodType::SPPMethod),
        "cff" => Ok(MethodType::CosFitMethod),
        "mm" => Ok(MethodType::MinMaxMethod),
        method_str => Err(format!("expected valid method name, found {}", method_str)),
    }
    .map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoadOptions {
    pub(crate) extensions: StringSequence,
    #[serde(default = "default_placeholder")]
    pub(crate) exclude_patterns: StringSequence,
    #[serde(default = "default_placeholder")]
    pub(crate) skip_files: StringSequence,
    skiprows: u32,
    meta_len: u32,
    decimal: char,
    delimiter: char,
    #[serde(rename = "mod")]
    #[serde(default)]
    pub(crate) _mod: Option<_Mod>,
    #[serde(default = "no_comment_check_default")]
    no_comment_check: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Preprocess {
    chdomain: Option<bool>,
    #[serde(default = "input_unit_default")]
    input_unit: Option<String>,
    slice_start: Option<f64>,
    slice_stop: Option<f64>,
}

#[allow(clippy::enum_variant_names)]
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub(crate) enum MethodType {
    CosFitMethod,
    FFTMethod,
    WFTMethod,
    MinMaxMethod,
    SPPMethod,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MethodDetails {
    heatmap: Option<bool>,
    windows: Option<i32>,
    fwhm: Option<f64>,
    std: Option<f64>,
    parallel: Option<bool>,
    plot: Option<bool>,
    min: Option<bool>,
    max: Option<bool>,
    both: Option<bool>,
    eager: Option<bool>,
    detach: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Evaluate {
    reference_frequency: Option<f64>,
    order: Option<u32>,
    only_phase: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum StringSequence {
    Vector(Vec<String>),
    String(String),
}

impl StringSequence {
    pub fn into_comparable(self) -> Vec<String> {
        match self {
            StringSequence::String(single) => vec![single],
            StringSequence::Vector(vec) => vec,
        }
    }
}

impl IntoIterator for StringSequence {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> std::vec::IntoIter<Self::Item> {
        let vec = match self {
            StringSequence::String(single) => vec![single],
            StringSequence::Vector(vec) => vec,
        };

        vec.into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct _Mod(pub i32);

impl Default for _Mod {
    fn default() -> Self {
        _Mod(1)
    }
}

impl PartialEq for _Mod {
    fn eq(&self, other: &_Mod) -> bool {
        self.0 == other.0
    }
}

impl Eq for _Mod {}

fn input_unit_default() -> Option<String> {
    Some("nm".to_owned())
}

fn no_comment_check_default() -> Option<bool> {
    Some(false)
}

fn default_trigger() -> Option<StringSequence> {
    Some(StringSequence::Vector(vec![
        String::from(""),
        String::from(""),
    ]))
}

fn default_placeholder() -> StringSequence {
    StringSequence::String(String::from("default_features"))
}
