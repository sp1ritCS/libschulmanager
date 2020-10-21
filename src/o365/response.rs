#![allow(non_snake_case)]
use serde::Deserialize;

/* Initial Response */

#[derive(Deserialize, Debug, Clone)]
pub struct InitO365 {
    pub sCtx: String,
    pub sFT: String,
    pub canary: String,
    pub sessionId: String,
}

/* Credential Response */
