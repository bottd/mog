use serde::Serialize;

use self::attribute::Attribute;
use self::delimiter::Delimiter;
use self::marker::Marker;
use crate::metadata::Metadata;

pub mod attribute;
pub mod delimiter;
pub mod marker;

#[derive(Debug, Serialize, PartialEq)]
pub enum NodeKind {
    Marker(Marker),
    Delimiter(Delimiter),
    Text,
    Raw(String),
    Table,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub attributes: Option<Vec<Attribute>>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct Document {
    pub meta: Option<Metadata>,
    pub body: Vec<Node>,
}

impl Node {
    pub(crate) fn leaf(kind: NodeKind) -> Node {
        Node {
            kind,
            attributes: None,
            children: None,
        }
    }
}
