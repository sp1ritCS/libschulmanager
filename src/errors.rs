use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmError {
    #[error("Unknown authentication issue")]
    UnknownAuth,
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("schulmanager-online.de returned statuscode '{}'", statuscode)]
    NonvalidStatusCode { statuscode: u16 },
    #[error("schuldmanager-online.de did not return a json webtoken")]
    NoJwt,
    #[error("schuldmanager-online.de did not return any data")]
    NoData,
    #[error("Unknown Office SSO Error")]
    UnknownMS,
    #[error("Incorrect Microsoft username or password")]
    InvalidMSCredentials,
    #[error("non-valid client Id")]
    NonvalidAppId
}
