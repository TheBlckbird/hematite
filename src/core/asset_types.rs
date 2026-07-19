use std::fmt::Display;

use crab_nbt::NbtTag;

use crate::core::nbt::{FromNbtTag, IntoNbtTag};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResourceLocation {
    pub namespace: String,
    pub identifier: String,
}

impl ResourceLocation {
    /// Panics if namespace or identifier contain any other characters than a-z and _
    pub fn new(namespace: String, identifier: String) -> Self {
        if !is_correct_format(&namespace) {
            panic!(r#""{namespace}" contains invalid characters"#);
        }

        if !is_correct_format(&identifier) {
            panic!(r#""{namespace}" contains invalid characters"#);
        }

        Self {
            namespace,
            identifier,
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}:{}", self.namespace, self.identifier)
    }
}

impl IntoNbtTag for ResourceLocation {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::String(self.as_string())
    }
}

impl FromNbtTag for ResourceLocation {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::String(resource) => resource.try_into().ok(),
            _ => None,
        }
    }
}

impl From<(String, String)> for ResourceLocation {
    fn from(value: (String, String)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl TryFrom<String> for ResourceLocation {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (namespace, identifier) = value.split_once(':').ok_or(())?;

        if !is_correct_format(namespace) || !is_correct_format(identifier) {
            Err(())
        } else {
            Ok(Self::new(namespace.to_owned(), identifier.to_owned()))
        }
    }
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.identifier)
    }
}

fn is_correct_format(input: &str) -> bool {
    input
        .chars()
        .all(|c: char| c.is_ascii_lowercase() || c == '_')
}
