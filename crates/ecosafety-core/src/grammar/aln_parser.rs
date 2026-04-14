//! ALN (ALN Specification) parser and validator.
//! Reads .aln files and produces structured representations for macros and runtime.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Root of an ALN file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnDocument {
    pub families: Vec<AlnFamily>,
    #[serde(default)]
    pub rows: Vec<AlnRow>,
}

/// A schema family definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnFamily {
    pub name: String,
    pub version: String,
    pub description: String,
    pub columns: Vec<AlnColumn>,
}

/// Column definition within a family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnColumn {
    pub varid: String,
    pub coltype: AlnColType,
    pub mandatory: bool,
    #[serde(default)]
    pub isriskcoord: bool,
    #[serde(default)]
    pub weight: Option<f32>,
    #[serde(default)]
    pub safegoldhard: Option<Vec<f32>>,
    #[serde(default)]
    pub normkind: Option<String>,
    pub description: String,
}

/// Column type enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlnColType {
    RiskCoord01,
    Hex64Evidence,
    Hex64Signature,
    UnixMillis,
    Utf8Id32,
    Utf8Id64,
    Utf8Id128,
    Float01,
    FloatArray6,
    Weight01,
    EnumLane,
    EnumNormKind,
    EnumDeployDecision,
    EnumRouteVariant,
    BoolFlag,
    Uint32,
    Uint64,
    Utf8String,
    Utf8Csv,
    SemverString,
}

/// A data row (for example corridor bands).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnRow {
    #[serde(flatten)]
    pub fields: HashMap<String, AlnValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AlnValue {
    String(String),
    Number(f64),
    Array(Vec<f64>),
    Bool(bool),
}

/// Parse an ALN file from disk.
pub fn parse_aln_file<P: AsRef<Path>>(path: P) -> Result<AlnDocument, AlnParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_aln_str(&content)
}

/// Parse ALN from string.
pub fn parse_aln_str(input: &str) -> Result<AlnDocument, AlnParseError> {
    // Simple line-based parser for ALN format.
    // ALN uses a simple key-value + indentation structure.
    let mut doc = AlnDocument {
        families: Vec::new(),
        rows: Vec::new(),
    };

    let mut lines = input.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("family:") {
            doc.families.push(parse_family(&mut lines)?);
        } else if trimmed.starts_with("rows:") {
            doc.rows = parse_rows(&mut lines)?;
        }
    }

    Ok(doc)
}

fn parse_family<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<AlnFamily, AlnParseError>
where I: Iterator<Item = &'a str>,
{
    let first = lines.next().ok_or(AlnParseError::UnexpectedEof)?;
    let name = first.trim().strip_prefix("family:").unwrap().trim().to_string();

    let mut version = String::new();
    let mut description = String::new();
    let mut columns = Vec::new();

    while let Some(line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.next();
            continue;
        }
        if trimmed.starts_with("family:") || trimmed.starts_with("rows:") {
            break;
        }
        let line = lines.next().unwrap();

        if trimmed.starts_with("version:") {
            version = trimmed.strip_prefix("version:").unwrap().trim().to_string();
        } else if trimmed.starts_with("description:") {
            description = trimmed.strip_prefix("description:").unwrap().trim().to_string();
        } else if trimmed.starts_with("columns:") {
            columns = parse_columns(lines)?;
        }
    }

    Ok(AlnFamily {
        name,
        version,
        description,
        columns,
    })
}

fn parse_columns<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<Vec<AlnColumn>, AlnParseError>
where I: Iterator<Item = &'a str>,
{
    let mut columns = Vec::new();
    while let Some(line) = lines.peek() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.next();
            continue;
        }
        if !trimmed.starts_with('-') && !trimmed.starts_with("varid:") {
            break;
        }
        if trimmed.starts_with('-') {
            // Column entry
            lines.next();
            let mut col = AlnColumn {
                varid: String::new(),
                coltype: AlnColType::Utf8String,
                mandatory: true,
                isriskcoord: false,
                weight: None,
                safegoldhard: None,
                normkind: None,
                description: String::new(),
            };
            // Parse indented fields
            while let Some(inner) = lines.peek() {
                let inner_trimmed = inner.trim();
                if inner_trimmed.is_empty() {
                    lines.next();
                    continue;
                }
                if inner_trimmed.starts_with('-') || inner_trimmed.starts_with("varid:") {
                    break;
                }
                let field_line = lines.next().unwrap().trim_start();
                if field_line.starts_with("varid:") {
                    col.varid = field_line.strip_prefix("varid:").unwrap().trim().to_string();
                } else if field_line.starts_with("coltype:") {
                    let type_str = field_line.strip_prefix("coltype:").unwrap().trim();
                    col.coltype = parse_coltype(type_str)?;
                } else if field_line.starts_with("mandatory:") {
                    col.mandatory = field_line.strip_prefix("mandatory:").unwrap().trim() == "true";
                } else if field_line.starts_with("isriskcoord:") {
                    col.isriskcoord = field_line.strip_prefix("isriskcoord:").unwrap().trim() == "true";
                } else if field_line.starts_with("weight:") {
                    col.weight = Some(field_line.strip_prefix("weight:").unwrap().trim().parse().unwrap());
                } else if field_line.starts_with("safegoldhard:") {
                    let arr_str = field_line.strip_prefix("safegoldhard:").unwrap().trim();
                    col.safegoldhard = Some(parse_float_array(arr_str)?);
                } else if field_line.starts_with("normkind:") {
                    col.normkind = Some(field_line.strip_prefix("normkind:").unwrap().trim().to_string());
                } else if field_line.starts_with("description:") {
                    col.description = field_line.strip_prefix("description:").unwrap().trim().to_string();
                }
            }
            columns.push(col);
        } else {
            break;
        }
    }
    Ok(columns)
}

fn parse_coltype(s: &str) -> Result<AlnColType, AlnParseError> {
    match s {
        "RiskCoord01" => Ok(AlnColType::RiskCoord01),
        "Hex64Evidence" => Ok(AlnColType::Hex64Evidence),
        "Hex64Signature" => Ok(AlnColType::Hex64Signature),
        "UnixMillis" => Ok(AlnColType::UnixMillis),
        "Utf8Id32" => Ok(AlnColType::Utf8Id32),
        "Utf8Id64" => Ok(AlnColType::Utf8Id64),
        "Utf8Id128" => Ok(AlnColType::Utf8Id128),
        "Float01" => Ok(AlnColType::Float01),
        "FloatArray6" => Ok(AlnColType::FloatArray6),
        "Weight01" => Ok(AlnColType::Weight01),
        "EnumLane" => Ok(AlnColType::EnumLane),
        "EnumNormKind" => Ok(AlnColType::EnumNormKind),
        "EnumDeployDecision" => Ok(AlnColType::EnumDeployDecision),
        "EnumRouteVariant" => Ok(AlnColType::EnumRouteVariant),
        "BoolFlag" => Ok(AlnColType::BoolFlag),
        "Uint32" => Ok(AlnColType::Uint32),
        "Uint64" => Ok(AlnColType::Uint64),
        "Utf8String" => Ok(AlnColType::Utf8String),
        "Utf8Csv" => Ok(AlnColType::Utf8Csv),
        "SemverString" => Ok(AlnColType::SemverString),
        _ => Err(AlnParseError::UnknownColType(s.to_string())),
    }
}

fn parse_float_array(s: &str) -> Result<Vec<f32>, AlnParseError> {
    let s = s.trim_matches(|c| c == '[' || c == ']');
    s.split(',')
        .map(|v| v.trim().parse::<f32>().map_err(|_| AlnParseError::InvalidNumber(v.to_string())))
        .collect()
}

fn parse_rows<'a, I>(lines: &mut std::iter::Peekable<I>) -> Result<Vec<AlnRow>, AlnParseError>
where I: Iterator<Item = &'a str>,
{
    // Simplified row parsing
    Ok(Vec::new())
}

#[derive(Debug, thiserror::Error)]
pub enum AlnParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Unknown column type: {0}")]
    UnknownColType(String),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ALN: &str = r#"
family: TestFamily
version: 1.0.0
description: "Test family"
columns:
  - varid: "test_coord"
    coltype: RiskCoord01
    mandatory: true
    isriskcoord: true
    weight: 0.5
    safegoldhard: [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    description: "Test coordinate"
"#;

    #[test]
    fn parse_sample_aln() {
        let doc = parse_aln_str(SAMPLE_ALN).unwrap();
        assert_eq!(doc.families.len(), 1);
        let family = &doc.families[0];
        assert_eq!(family.name, "TestFamily");
        assert_eq!(family.columns[0].varid, "test_coord");
    }
}
