use clap::ArgMatches;
use klap::{annotation_from_str, labels_from_str_either, AnnotationMap, Label, LabelMap};
use serde::Deserialize;
use serde_yaml::{from_reader, from_str};
use std::fs;
use std::io::BufReader;

use crate::types::Error;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Metadata {
    pub name: Option<String>,
    #[serde(default)]
    pub labels: LabelMap,
    #[serde(default)]
    pub annotations: AnnotationMap,
}

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    metadata: Metadata,
}

fn parse_metadata(input: &str) -> Result<Metadata, Error> {
    let res: Result<Manifest, _> = if let Some(filename) = input.strip_prefix('@') {
        let f = fs::File::open(filename).map_err(|v| {
            Error::Option(
                "metadata-from-manifest".to_string(),
                input.to_string(),
                v.to_string(),
            )
        })?;
        from_reader(BufReader::new(f))
    } else {
        from_str(input)
    };
    res.map(|v| v.metadata)
        .map_err(|e| Error::Unknown(e.to_string()))
}

fn match_labels(matches: &ArgMatches<'_>, labels: &mut LabelMap) -> Result<(), Error> {
    if let Some(label_opts) = matches.values_of("labels") {
        for labelstr in label_opts {
            match labels_from_str_either(labelstr) {
                Err(e) => {
                    return Err(Error::Option(
                        "labels".to_string(),
                        labelstr.to_string(),
                        format!("\n{}", e),
                    ));
                }
                Ok(l) => {
                    labels.extend(l.into_iter().map(Label::into_tuple));
                }
            }
        }
    }
    Ok(())
}

fn match_annotations(
    matches: &ArgMatches<'_>,
    annotations: &mut AnnotationMap,
) -> Result<(), Error> {
    if let Some(anno_opts) = matches.values_of("annotations") {
        for anno_str in anno_opts {
            match annotation_from_str(anno_str) {
                Err(e) => {
                    return Err(Error::Option(
                        "annotation".to_string(),
                        anno_str.to_string(),
                        format!("\n{}", e),
                    ));
                }
                Ok(an) => {
                    annotations.insert(an.key, an.value);
                }
            }
        }
    }
    Ok(())
}

pub fn metadata_from_matches(matches: &ArgMatches<'_>) -> Result<Metadata, Error> {
    let mut metadata: Metadata;
    if let Some(manifest) = matches.value_of("manifest") {
        metadata = parse_metadata(manifest)?;
    } else {
        metadata = Metadata::default();
    }
    match_labels(matches, &mut metadata.labels)?;
    match_annotations(matches, &mut metadata.annotations)?;
    Ok(metadata)
}
