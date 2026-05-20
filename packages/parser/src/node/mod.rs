use serde::Serialize;

use crate::attribute::Attribute;
use crate::marker::Marker;
use crate::metadata::Metadata;
use crate::node::task::Task;

pub mod task;

#[derive(Debug, Serialize, PartialEq)]
pub enum SemanticDelimiter {}

#[derive(Debug, Serialize, PartialEq)]
pub enum Node {
    Marker(Marker),
    Task(Task),
    Delimiter {
        attributes: Vec<Attribute>,
        kind: SemanticDelimiter,
    },
    Escape(char),
    EmptyLine,
    Text {
        value: String,
    },
}

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct Document {
    pub meta: Option<Metadata>,
    pub body: Vec<Node>,
}
