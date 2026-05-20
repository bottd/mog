use crate::error::MogError;
use kdl::{KdlDocument, KdlNode, KdlValue};
use serde::Serialize;
use serde_value::Value;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Serialize, Default, PartialEq)]
#[serde(transparent)]
pub struct Metadata(Vec<(String, Value)>);

impl Deref for Metadata {
    type Target = Vec<(String, Value)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Metadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<KdlDocument> for Metadata {
    type Error = MogError;

    fn try_from(document: KdlDocument) -> Result<Self, Self::Error> {
        let mut meta = Metadata::default();
        for node in document.into_iter() {
            meta.push((node.name().value().to_string(), node_into_value(&node)?));
        }
        Ok(meta)
    }
}

pub fn node_into_value(node: &KdlNode) -> Result<Value, MogError> {
    let mut args = Vec::new();
    let mut props = BTreeMap::new();

    for entry in node.entries() {
        let value = value_into_value(entry.value())?;
        match entry.name() {
            None => args.push(value),
            Some(name) => {
                props.insert(Value::String(name.value().to_string()), value);
            }
        }
    }

    let children: Vec<Value> = node
        .children()
        .into_iter()
        .flat_map(|doc| doc.nodes())
        .map(|child| {
            let mut entry = BTreeMap::new();
            entry.insert(
                Value::String("name".into()),
                Value::String(child.name().value().to_string()),
            );
            entry.insert(Value::String("value".into()), node_into_value(child)?);
            Ok(Value::Map(entry))
        })
        .collect::<Result<_, MogError>>()?;

    let mut map = BTreeMap::new();
    if !args.is_empty() {
        map.insert(Value::String("args".into()), Value::Seq(args));
    }
    if !props.is_empty() {
        map.insert(Value::String("props".into()), Value::Map(props));
    }
    if !children.is_empty() {
        map.insert(Value::String("children".into()), Value::Seq(children));
    }
    Ok(Value::Map(map))
}

fn value_into_value(value: &KdlValue) -> Result<Value, MogError> {
    if let Some(s) = value.as_string() {
        return Ok(Value::String(s.to_string()));
    }
    if let Some(int) = value.as_integer() {
        return i64::try_from(int)
            .map(Value::I64)
            .map_err(|_| MogError::IntegerOverflow(int));
    }
    if let Some(float) = value.as_float() {
        return Ok(Value::F64(float));
    }
    if let Some(bool) = value.as_bool() {
        return Ok(Value::Bool(bool));
    }
    Ok(Value::Unit)
}
