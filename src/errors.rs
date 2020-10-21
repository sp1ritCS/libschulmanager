use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum SmAuthError {
    #[snafu(display("Unknown authentication issue"))]
    UnknownAuth,
    #[snafu(display("schulmanager-online.de returned statuscode '{}'", statuscode))]
    NonvalidStatusCode { statuscode: u16 }
}

#[derive(Debug, Snafu)]
pub enum SmO365Error {
    #[snafu(display("Unknown Office SSO Error"))]
    UnknownMS,
    #[snafu(display("non-valid client Id"))]
    NonvalidAppId
}
