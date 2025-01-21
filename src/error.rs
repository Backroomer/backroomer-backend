#![allow(dead_code)]
use std::{fmt::Display, num::{ParseFloatError, ParseIntError}};
use reqwest::header::{InvalidHeaderValue, ToStrError};

#[derive(Debug)]
pub struct ParseElementError {
    kind: String,
    message: String,
}

impl ParseElementError{
    pub fn new(kind: &str, message: &str) -> Self {
        ParseElementError{kind: kind.to_string(), message: message.to_string()}
    }

    pub fn revision_id() -> Self {
        Self::new("Revision", "Cannot get revision id from the element.")
    }

    pub fn revision_ele() -> Self {
        Self::new("Revision", "Element out of bound.")
    }

    pub fn page_num() -> Self {
        Self::new("Page", "Cannot get page number.")
    }

    pub fn site_id() -> Self {
        Self::new("Site", "Cannot get site id from the element.")
    }

    pub fn site_title() -> Self {
        Self::new("Site", "Cannot get site title from the element.")
    }

    pub fn site_ele() -> Self {
        Self::new("Site", "Element out of bound.")
    }

    pub fn parser_id() -> Self {
        Self::new("Parser", "Cannot get id from the element.")
    }

    pub fn parser_unix_name() -> Self {
        Self::new("Parser", "Cannot get unix name from the element.")
    }

    pub fn user_date() -> Self {
        Self::new("User", "Cannot get joined date from the element.")
    }

    pub fn user_ele() -> Self {
        Self::new("User", "Element out of bound.")
    }

    pub fn user_avatar() -> Self {
        Self::new("User", "Cannot get avatar url from the element.")
    }
}

impl Display for ParseElementError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseElementError{
}

#[derive(Debug)]
pub struct IdNotFound{
    kind: String,
    message: String,
}

impl Display for IdNotFound{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for IdNotFound{
}

impl IdNotFound{
    pub fn new(kind: &str, message: &str) -> Self {
        IdNotFound{kind: kind.to_string(), message: message.to_string()}
    }

    pub fn site() -> Self{
        Self::new("Site", "Failed to parse site id.")
    }

    pub fn page() -> Self{
        Self::new("Page", "Failed to parse page id.")
    }
}

#[derive(Debug)]
pub struct TargetNotExist{
    kind: String,
    message: String,
}

impl Display for TargetNotExist{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TargetNotExist{
}

impl TargetNotExist{
    pub fn new(kind: &str, message: &str) -> Self {
        TargetNotExist{kind: kind.to_string(), message: message.to_string()}
    }

    pub fn site() -> Self{
        Self::new("Site", "Site not found.")
    }

    pub fn page() -> Self{
        Self::new("Page", "Page not found.")
    }
}

#[derive(Debug)]
pub struct WikidotRespondError{
    kind: String,
    message: String,
}

impl Display for WikidotRespondError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for WikidotRespondError{
}

#[derive(Debug)]
pub enum AjaxClientError{
    ReqwestError(reqwest::Error),
    HeaderParseError(InvalidHeaderValue),
    WikidotRespondError(WikidotRespondError),
    ToStrError(ToStrError),
}

impl Display for AjaxClientError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AjaxClientError{
}

impl WikidotRespondError{
    pub fn new(kind: &str, message: &str) -> Self {
        WikidotRespondError{kind: kind.to_string(), message: message.to_string()}
    }

    pub fn status(status: reqwest::StatusCode) -> Self{
        Self::new("status", status.as_str())
    }

    pub fn try_again() -> Self {
        Self::new("body", "Body status is 'try_again'.")
    }

    pub fn empty() -> Self {
        Self::new("body", "Body is empty.")
    }
}

impl From<reqwest::Error> for AjaxClientError{
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<InvalidHeaderValue> for AjaxClientError{
    fn from(value: InvalidHeaderValue) -> Self {
        Self::HeaderParseError(value)
    }
}

impl From<WikidotRespondError> for AjaxClientError{
    fn from(value: WikidotRespondError) -> Self {
        Self::WikidotRespondError(value)
    }
}


impl From<ToStrError> for AjaxClientError{
    fn from(value: ToStrError) -> Self {
        Self::ToStrError(value)
    }
}

#[derive(Debug)]
pub enum WikidotError{
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

impl Display for WikidotError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for WikidotError{
}

impl From<reqwest::Error> for WikidotError{
    fn from(value: reqwest::Error) -> Self {
        Self::ClientError(AjaxClientError::from(value))
    }
}

impl From<ToStrError> for WikidotError{
    fn from(value: ToStrError) -> Self {
        Self::ClientError(AjaxClientError::from(value))
    }
}

impl From<AjaxClientError> for WikidotError{
    fn from(value: AjaxClientError) -> Self {
        Self::ClientError(value)
    }
}

impl From<regex::Error> for WikidotError{
    fn from(value: regex::Error) -> Self {
        Self::ParseRegexError(value)
    }
}

impl From<ParseElementError> for WikidotError{
    fn from(value: ParseElementError) -> Self {
        Self::ParseElementError(value)
    }
}

impl From<ParseIntError> for WikidotError{
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<ParseFloatError> for WikidotError{
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

impl From<IdNotFound> for WikidotError{
    fn from(value: IdNotFound) -> Self {
        Self::IdNotFound(value)
    }
}

impl From<TargetNotExist> for WikidotError{
    fn from(value: TargetNotExist) -> Self {
        Self::TargetNotExist(value)
    }
}

impl From<serde_json::Error> for WikidotError{
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJsonError(value)
    }
}

impl From<mongodb::error::Error> for WikidotError{
    fn from(value: mongodb::error::Error) -> Self {
        Self::MongodbError(value)
    }
}