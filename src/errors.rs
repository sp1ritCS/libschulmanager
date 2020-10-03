use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum SmAuthError {
    #[snafu(display("Unknown authentication issue"))]
    Unknown,
    #[snafu(display("schulmanager-online.de returned statuscode '{}'", statuscode))]
    NonvalidStatusCode { statuscode: u16 }
}
