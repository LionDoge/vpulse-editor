#![allow(nonstandard_style)]
use std::fmt;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
pub enum PulseValueType {
    PVAL_INT,
    PVAL_FLOAT,
    PVAL_STRING,
    PVAL_INVALID,
    PVAL_EHANDLE(String),
    PVAL_VEC3,
    PVAL_COLOR_RGB,
    DOMAIN_ENTITY_NAME,
}
impl fmt::Display for PulseValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PulseValueType::PVAL_INT => write!(f, "PVAL_INT"),
            PulseValueType::PVAL_FLOAT => write!(f, "PVAL_FLOAT"),
            PulseValueType::PVAL_STRING => write!(f, "PVAL_STRING"),
            PulseValueType::PVAL_INVALID => write!(f, "PVAL_INVALID"),
            PulseValueType::DOMAIN_ENTITY_NAME => write!(f, "PVAL_ENTITY_NAME"),
            PulseValueType::PVAL_EHANDLE(ent_type) => write!(f, "PVAL_EHANDLE:{}", *ent_type),
            PulseValueType::PVAL_VEC3 => write!(f, "PVAL_VEC3"),
            PulseValueType::PVAL_COLOR_RGB => write!(f, "PVAL_COLOR_RGB"),
        }
    }
}