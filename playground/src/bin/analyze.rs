use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Eq, PartialEq, Hash, Debug)]
enum Kind {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl From<&Value> for Kind {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Kind::Null,
            Value::Bool(_) => Kind::Bool,
            Value::Number(_) => Kind::Number,
            Value::String(_) => Kind::String,
            Value::Array(_) => Kind::Array,
            Value::Object(_) => Kind::Object,
        }
    }
}

struct FieldInfo {
    count: usize,
    kinds: HashSet<Kind>,
    min: Option<f64>,
    max: Option<f64>,
}

impl Display for FieldInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FieldInfo {{count: `{}`, kinds: `{:?}`, min: `{:?}`, max: `{:?}`}}",
            self.count, self.kinds, self.min, self.max
        )
    }
}

fn num_from_value(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(n.as_f64().expect("number can not be turned into f64")),
        _ => None,
    }
}

struct CountMap(HashMap<String, FieldInfo>);

impl CountMap {
    fn from_file(file: &str) -> Result<CountMap, Box<dyn Error>> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let mut fields = HashMap::<String, FieldInfo>::new();

        for line in reader.lines() {
            let node: Value = serde_json::from_str(line?.as_str())?;
            let lots = &node["data"]["results"]["content"]
                .as_array()
                .ok_or("can not convert to array")?;

            for lot in *lots {
                let lot = lot.as_object().ok_or("can not convert to object")?;
                lot.iter().for_each(|(k, v)| {
                    if let Some(info) = fields.get_mut(k) {
                        info.count += 1;
                        info.kinds.insert(Kind::from(v));
                        if let Some(num) = num_from_value(v) {
                            match (info.min, info.max) {
                                (Some(min), _) if num < min => info.min = Some(num),
                                (_, Some(max)) if num > max => info.max = Some(num),
                                (None, None) => {
                                    info.min = Some(num);
                                    info.max = Some(num);
                                }
                                (None, Some(_)) | (Some(_), None) => {}
                                (Some(_), Some(_)) => {}
                            }
                        }
                    } else {
                        fields.insert(
                            k.to_owned(),
                            FieldInfo {
                                count: 1,
                                kinds: HashSet::from([Kind::from(v)]),
                                min: num_from_value(v),
                                max: num_from_value(v),
                            },
                        );
                    }
                });
            }
        }

        Ok(Self(fields))
    }

    fn join(&mut self, other: CountMap) {
        other.0.into_iter().for_each(|(k, v)| {
            if let Some(info) = self.0.get_mut(&k) {
                info.count += v.count;
                info.kinds.extend(v.kinds);
            } else {
                self.0.insert(k, v);
            }
        })
    }
}

impl Display for CountMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fields = self.0.iter().collect::<Vec<(&String, &FieldInfo)>>();
        fields.sort_by_key(|(_, v)| -(v.count as isize));
        let msg = fields
            .iter()
            .fold(String::new(), |acc, (k, v)| format!("{acc}{k}: {v}\n"));
        write!(f, "{msg}")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut cm1 = CountMap::from_file("samples/lot_search_1.jsonl")?;
    let cm2 = CountMap::from_file("samples/lot_search_2.jsonl")?;
    cm1.join(cm2);

    println!("{cm1}");
    Ok(())
}
