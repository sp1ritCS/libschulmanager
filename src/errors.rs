use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum SmError {
    #[snafu(display("Unknown authentication issue"))]
    UnknownAuth,
    #[snafu(display("Unauthenticated"))]
    Unauthenticated,
    #[snafu(display("schulmanager-online.de returned statuscode '{}'", statuscode))]
    NonvalidStatusCode { statuscode: u16 },
    #[snafu(display("schuldmanager-online.de did not return a json webtoken"))]
    NoJwt,
    #[snafu(display("Unknown Office SSO Error"))]
    UnknownMS,
    #[snafu(display("non-valid client Id"))]
    NonvalidAppId
}
