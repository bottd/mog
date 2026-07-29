use serde::Serialize;

use self::data::{Data, Value, parse_data};
use self::delimiter::Delimiter;
use self::marker::Marker;

pub mod attribute;
pub mod data;
pub mod delimiter;
pub mod marker;

#[derive(Debug, Serialize, PartialEq)]
pub enum NodeKind {
    Marker(Marker),
    Delimiter(Delimiter),
    Data(Vec<Data>),
    Raw(String),
    Table,
    Text(String),
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<Data>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Node>>,
}

pub(crate) enum DataArgument {
    Meta,
    Data,
}

impl DataArgument {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Meta => "meta",
            Self::Data => "data",
        }
    }
}

impl Node {
    pub(crate) fn push_child(&mut self, child: Node) {
        self.children.get_or_insert_default().push(child);
    }

    pub(crate) fn raw(text: impl Into<String>) -> Node {
        Node::empty(NodeKind::Raw(text.into()))
    }

    pub(crate) fn empty(kind: NodeKind) -> Node {
        Node {
            attributes: None,
            children: None,
            kind,
        }
    }

    pub(crate) fn get_verbatim_data(&self, argument: DataArgument) -> Option<Vec<Data>> {
        let argument = argument.as_str();
        let has_data_attribute = self.attributes.iter().flatten().any(|attribute| {
            matches!(attribute.as_argument(), Some(Value::String(string)) if string == argument)
        });

        (self.kind == NodeKind::Delimiter(Delimiter::Verbatim) && has_data_attribute)
            .then(|| parse_data(&self.get_raw_text()))
            .flatten()
    }

    pub(crate) fn get_raw_text(&self) -> String {
        self.children
            .iter()
            .flatten()
            .filter_map(|child| match &child.kind {
                NodeKind::Raw(text) => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
