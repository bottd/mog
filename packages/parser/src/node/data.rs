use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct Data {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    pub value: Value,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
    Node {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        entries: Vec<Data>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        children: Vec<Data>,
    },
}

impl Data {
    pub(crate) fn as_argument(&self) -> Option<&Value> {
        self.name.is_none().then_some(&self.value)
    }
}

pub(crate) fn parse_data(text: &str) -> Option<Vec<Data>> {
    Some(document_into_data(&KdlDocument::parse(text).ok()?))
}

fn document_into_data(document: &KdlDocument) -> Vec<Data> {
    document.nodes().iter().map(node_into_data).collect()
}

fn node_into_data(node: &KdlNode) -> Data {
    Data {
        name: Some(node.name().value().to_string()),
        ty: node.ty().map(|ty| ty.value().to_string()),
        value: Value::Node {
            entries: node.entries().iter().map(entry_into_data).collect(),
            children: node.children().map(document_into_data).unwrap_or_default(),
        },
    }
}

pub(crate) fn entry_into_data(entry: &KdlEntry) -> Data {
    Data {
        name: entry.name().map(|name| name.value().to_string()),
        ty: entry.ty().map(|ty| ty.value().to_string()),
        value: match entry.value() {
            KdlValue::Bool(bool) => Value::Bool(*bool),
            KdlValue::Float(float) => Value::Float(*float),
            KdlValue::Integer(int) => Value::Int(*int),
            KdlValue::Null => Value::Null,
            // TODO: Why clone string and not *string ?
            KdlValue::String(string) => Value::String(string.clone()),
        },
    }
}
