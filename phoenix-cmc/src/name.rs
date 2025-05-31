use crate::types::ArcStr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Name {
    pub name: ArcStr,
    pub blurb: ArcStr,
}

impl Name {
    pub fn new(name: impl Into<ArcStr>, blurb: impl Into<ArcStr>) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            blurb: blurb.into(),
        })
    }

    pub fn with_name_only(name: impl Into<ArcStr>) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            blurb: "".into(),
        })
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
    }
}

impl Eq for Name {}

impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_lowercase().hash(state);
    }
}
