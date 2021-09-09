use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

type JsonMap = HashMap<String, Value>;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq)]
pub enum CoeffitientType {
    GD,
    GDD,
    TOD,
    FOD,
    QOD,
    SOD,
}

impl fmt::Display for CoeffitientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct DispersionCoeffitient {
    coeff_vec: Vec<f64>,
    coeff_type: CoeffitientType,
}

impl fmt::Display for DispersionCoeffitient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.coeff_type)?;
        if self.is_omitted() {
            write!(f, ": omitted..")?;
            Ok(())
        } else if self.coeff_type == CoeffitientType::GD {
            write!(
                f,
                " ranging from {:.5} to {:.5} {} (might be inaccurate due to sign conversions)",
                self.min(),
                self.max(),
                self.unit()
            )?;
            Ok(())
        } else {
            write!(
                f,
                ": mean = {:>12.5} | std = {:>12.5} | min = {:>12.5} | max = {:>12.5}  {}",
                self.mean().unwrap_or(0.0),
                self.std_deviation().unwrap_or(0.0),
                self.min(),
                self.max(),
                self.unit()
            )?;
            Ok(())
        }
    }
}

pub trait Evaluated {
    fn unit(&self) -> String;
    fn mean(&self) -> Option<f64>;
    fn std_deviation(&self) -> Option<f64>;
}

impl Evaluated for DispersionCoeffitient {
    fn unit(&self) -> String {
        match self.coeff_type {
            CoeffitientType::GD => "fs",
            CoeffitientType::GDD => "fs^2",
            CoeffitientType::TOD => "fs^3",
            CoeffitientType::FOD => "fs^4",
            CoeffitientType::QOD => "fs^5",
            CoeffitientType::SOD => "fs^6",
        }
        .to_owned()
    }

    fn mean(&self) -> Option<f64> {
        if self.coeff_type == CoeffitientType::GD {
            return None;
        }
        let sum = self.coeff_vec.iter().sum::<f64>();
        let count = self.coeff_vec.len();

        match count {
            positive if positive > 0 => Some(sum / count as f64),
            _ => None,
        }
    }

    fn std_deviation(&self) -> Option<f64> {
        if self.coeff_type == CoeffitientType::GD {
            return None;
        }
        match (self.mean(), self.coeff_vec.len()) {
            (Some(data_mean), count) if count > 0 => {
                let variance = self
                    .coeff_vec
                    .iter()
                    .map(|value| {
                        let diff = data_mean - (*value as f64);

                        diff * diff
                    })
                    .sum::<f64>()
                    / count as f64;

                Some(variance.sqrt())
            }
            _ => None,
        }
    }
}

impl DispersionCoeffitient {
    pub const fn empty_with_type(_type: CoeffitientType) -> Self {
        Self {
            coeff_vec: Vec::<f64>::new(),
            coeff_type: _type,
        }
    }

    pub fn push(&mut self, item: f64) {
        if self.coeff_vec.is_empty() {
            self.coeff_vec.push(item);
        } else {
            // To keep consistent sign between runs, we simply take the first
            // element's sign and apply it to the rest.
            // SAFETY: `coeff_vec` is not empty, so there's definitely a first element in it
            self.coeff_vec
                .push(item.copysign(unsafe { *self.coeff_vec.get_unchecked(0) }));
        }
    }

    pub fn extend_from_slice(&mut self, items: &[f64]) {
        self.coeff_vec.extend_from_slice(items);
    }

    pub fn with_values_and_type(values: &[f64], _type: CoeffitientType) -> Self {
        Self {
            coeff_vec: values.to_vec(),
            coeff_type: _type,
        }
    }

    pub fn len(&self) -> usize {
        self.coeff_vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_omitted(&self) -> bool {
        !self.coeff_vec.iter().any(|x| *x != 0.0)
    }

    pub fn min(&self) -> f64 {
        self.coeff_vec.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }

    pub fn max(&self) -> f64 {
        self.coeff_vec
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }
}

fn read_results_from_file<P: AsRef<Path>>(path: P) -> Result<JsonMap, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let map: JsonMap = serde_json::from_reader(reader)?;

    Ok(map)
}

/// Extract the results from the given deserialized map.
macro_rules! extract_coeff {
    ($name:tt, $v:ident) => {{
        let coeff: f64 = match &$v[$name] {
            Value::String(val) => val.parse::<f64>().unwrap(),
            Value::Number(val) => val.as_f64().unwrap(),
            _ => 0.0,
        };
        coeff
    }};
}

pub fn summarize<P: AsRef<Path>>(path: P) {
    let map = read_results_from_file(path).unwrap();

    let mut gds = DispersionCoeffitient::empty_with_type(CoeffitientType::GD);
    let mut gdds = DispersionCoeffitient::empty_with_type(CoeffitientType::GDD);
    let mut tods = DispersionCoeffitient::empty_with_type(CoeffitientType::TOD);
    let mut fods = DispersionCoeffitient::empty_with_type(CoeffitientType::FOD);
    let mut qods = DispersionCoeffitient::empty_with_type(CoeffitientType::QOD);
    let mut sods = DispersionCoeffitient::empty_with_type(CoeffitientType::SOD);
    let mut method: &str = "";

    for v in map.values() {
        let curr_gd = extract_coeff!("GD", v);
        gds.push(curr_gd);
        let curr_gdd = extract_coeff!("GDD", v);
        gdds.push(curr_gdd);
        let curr_tod = extract_coeff!("TOD", v);
        tods.push(curr_tod);
        let curr_fod = extract_coeff!("FOD", v);
        fods.push(curr_fod);
        let curr_qod = extract_coeff!("QOD", v);
        qods.push(curr_qod);
        let curr_sod = extract_coeff!("SOD", v);
        sods.push(curr_sod);
        method = v["method"].as_str().unwrap_or("unknown");
    }
    println!("{} entries found.", gds.len());
    println!("method: {}", method);
    println!("{}", gds);
    println!("{}", gdds);
    println!("{}", tods);
    println!("{}", fods);
    println!("{}", qods);
    println!("{}", sods);
}
