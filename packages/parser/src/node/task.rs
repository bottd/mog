use serde::Serialize;

use crate::{
    Node,
    attribute::{Attribute, parse_attributes},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct Task {
    pub kind: TaskKind,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum TaskKind {
    Undone,
    Done,
    InProgress,
    Uncertain,
    Urgent,
    Cancelled,
    Custom(char),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidTaskKind(pub char);

impl TryFrom<char> for TaskKind {
    type Error = InvalidTaskKind;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            ' ' => Ok(Self::Undone),
            'x' => Ok(Self::Done),
            '~' => Ok(Self::InProgress),
            '?' => Ok(Self::Uncertain),
            '!' => Ok(Self::Urgent),
            '-' => Ok(Self::Cancelled),
            c if matches!(c, '[' | ']' | '{' | '}' | '\n' | '\r') => Err(InvalidTaskKind(c)),
            c => Ok(Self::Custom(c)),
        }
    }
}

pub fn parse_task(line: &str) -> Option<(Node, &str)> {
    let mut chars = line.trim().chars();

    if chars.next() != Some('[') {
        return None;
    }

    let (attributes, rest) = parse_attributes(chars.as_str());
    let mut chars = rest.chars();

    if let Some(kind) = chars.next()
        && let Ok(kind) = TaskKind::try_from(kind)
        && let Some(closing) = chars.next()
        && closing == ']'
    {
        return Some((Node::Task(Task { attributes, kind }), chars.as_str()));
    }
    None
}
