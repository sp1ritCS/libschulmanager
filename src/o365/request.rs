#![allow(non_snake_case)]

pub struct Cookies {
    buid: String,
    esctx: String
}

pub struct AuthBody {
    login: String,
    loginfmt: String,
    type: String,
    LoginOptions: String,
    ps: String,
    passwd: String,
    ctx: String,
    hpgrequestid: String, /*prob. not required*/
    flowToken: String
}
