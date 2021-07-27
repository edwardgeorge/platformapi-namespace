use klap::{AnnotationMap, LabelMap};
use serde::Deserialize;
use serde_yaml::{from_reader, from_str};
use std::fs;

use crate::types::Error;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Metadata {
    #[serde(default)]
    pub labels: LabelMap,
    #[serde(default)]
    pub annotations: AnnotationMap,
}

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    metadata: Metadata,
}

pub fn parse_metadata(input: &str) -> Result<Metadata, Error> {
    let res: Result<Manifest, _> = if let Some(filename) = input.strip_prefix("@") {
        let f = fs::File::open(filename).map_err(|v| Error::UnknownError(v.to_string()))?;
        from_reader(f)
    } else {
        from_str(input)
    };
    res.map(|v| v.metadata)
        .map_err(|e| Error::UnknownError(e.to_string()))
}
