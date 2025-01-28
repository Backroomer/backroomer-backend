#![allow(dead_code)]
use std::{fmt::Display, num::{ParseFloatError, ParseIntError}};
use reqwest::header::{InvalidHeaderValue, ToStrError};

// 定义错误实现宏
macro_rules! impl_error {
    ($type:ty) => {
        impl Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[{}] {}", self.kind, self.message)
            }
        }
        impl std::error::Error for $type {}
    };
}

// 定义错误创建宏
macro_rules! define_error {
    ($type:ident, $($variant:ident => ($kind:expr, $message:expr)),* $(,)?) => {
        #[derive(Debug)]
        pub struct $type {
            kind: String,
            message: String,
        }

        impl $type {
            pub fn new(kind: &str, message: &str) -> Self {
                Self { kind: kind.to_string(), message: message.to_string() }
            }

            $(
                pub fn $variant() -> Self {
                    Self::new($kind, $message)
                }
            )*
        }

        impl_error!($type);
    };
}

// 使用宏定义所有基础错误类型
define_error!(ParseElementError,
    revision_id => ("Revision", "Cannot get revision id from the element"),
    revision_ele => ("Revision", "Element out of bound"),
    page_num => ("Page", "Cannot get page number"),
    page_ele => ("Mongodb", "Element out of bound"),
    site_id => ("Site", "Cannot get site id from the element"),
    site_title => ("Site", "Cannot get site title from the element"),
    site_ele => ("Site", "Element out of bound"),
    parser_id => ("Parser", "Cannot get id from the element"),
    parser_unix_name => ("Parser", "Cannot get unix name from the element"),
    user_date => ("User", "Cannot get joined date from the element"),
    user_ele => ("User", "Element out of bound"),
    user_avatar => ("User", "Cannot get avatar url from the element"),
    mongo_ele => ("Mongodb", "Element out of bound"),
);

define_error!(IdNotFound,
    site => ("Site", "Failed to parse site id"),
    page => ("Page", "Failed to parse page id"),
);

define_error!(TargetNotExist,
    site => ("Site", "Site not found"),
    page => ("Page", "Page not found"),
);

define_error!(WikidotRespondError,
    try_again => ("body", "Body status is 'try_again'"),
    empty => ("body", "Body is empty"),
);

impl WikidotRespondError {
    pub fn status(status: reqwest::StatusCode) -> Self {
        Self::new("status", status.as_str())
    }
}

#[derive(Debug)]
pub enum AjaxClientError {
    ReqwestError(reqwest::Error),
    HeaderParseError(InvalidHeaderValue),
    WikidotRespondError(WikidotRespondError),
    ToStrError(ToStrError),
}

impl Display for AjaxClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestError(e) => write!(f, "HTTP request error: {}", e),
            Self::HeaderParseError(e) => write!(f, "Header parse error: {}", e),
            Self::WikidotRespondError(e) => write!(f, "Wikidot response error: {}", e),
            Self::ToStrError(e) => write!(f, "String conversion error: {}", e),
        }
    }
}

impl std::error::Error for AjaxClientError {}

// From implementations for AjaxClientError
impl From<reqwest::Error> for AjaxClientError {
    fn from(value: reqwest::Error) -> Self { Self::ReqwestError(value) }
}

impl From<InvalidHeaderValue> for AjaxClientError {
    fn from(value: InvalidHeaderValue) -> Self { Self::HeaderParseError(value) }
}

impl From<WikidotRespondError> for AjaxClientError {
    fn from(value: WikidotRespondError) -> Self { Self::WikidotRespondError(value) }
}

impl From<ToStrError> for AjaxClientError {
    fn from(value: ToStrError) -> Self { Self::ToStrError(value) }
}

#[derive(Debug)]
pub enum WikidotError {
    ParseRegexError(regex::Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ParseElementError(ParseElementError),
    ClientError(AjaxClientError),
    IdNotFound(IdNotFound),
    TargetNotExist(TargetNotExist),
    SerdeJsonError(serde_json::Error),
    MongodbError(mongodb::error::Error),
}

impl Display for WikidotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseRegexError(e) => write!(f, "Regex parse error: {}", e),
            Self::ParseIntError(e) => write!(f, "Integer parse error: {}", e),
            Self::ParseFloatError(e) => write!(f, "Float parse error: {}", e),
            Self::ParseElementError(e) => write!(f, "Element parse error: {}", e),
            Self::ClientError(e) => write!(f, "Client error: {}", e),
            Self::IdNotFound(e) => write!(f, "ID not found: {}", e),
            Self::TargetNotExist(e) => write!(f, "Target not exist: {}", e),
            Self::SerdeJsonError(e) => write!(f, "JSON parse error: {}", e),
            Self::MongodbError(e) => write!(f, "MongoDB error: {}", e),
        }
    }
}

impl std::error::Error for WikidotError {}

// From implementations for WikidotError
impl From<reqwest::Error> for WikidotError {
    fn from(value: reqwest::Error) -> Self { Self::ClientError(AjaxClientError::from(value)) }
}

impl From<ToStrError> for WikidotError {
    fn from(value: ToStrError) -> Self { Self::ClientError(AjaxClientError::from(value)) }
}

impl From<AjaxClientError> for WikidotError {
    fn from(value: AjaxClientError) -> Self { Self::ClientError(value) }
}

impl From<regex::Error> for WikidotError {
    fn from(value: regex::Error) -> Self { Self::ParseRegexError(value) }
}

impl From<ParseElementError> for WikidotError {
    fn from(value: ParseElementError) -> Self { Self::ParseElementError(value) }
}

impl From<ParseIntError> for WikidotError {
    fn from(value: ParseIntError) -> Self { Self::ParseIntError(value) }
}

impl From<ParseFloatError> for WikidotError {
    fn from(value: ParseFloatError) -> Self { Self::ParseFloatError(value) }
}

impl From<IdNotFound> for WikidotError {
    fn from(value: IdNotFound) -> Self { Self::IdNotFound(value) }
}

impl From<TargetNotExist> for WikidotError {
    fn from(value: TargetNotExist) -> Self { Self::TargetNotExist(value) }
}

impl From<serde_json::Error> for WikidotError {
    fn from(value: serde_json::Error) -> Self { Self::SerdeJsonError(value) }
}

impl From<mongodb::error::Error> for WikidotError {
    fn from(value: mongodb::error::Error) -> Self { Self::MongodbError(value) }
}