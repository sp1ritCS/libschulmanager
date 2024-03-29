mod response;
use anyhow::Result;
use html5ever::tendril::stream::TendrilSink;
use kuchiki;
use isahc::{prelude::*, HttpClient};

macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                //eprintln!("Nothing was defined");
                continue;
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct O365Auth {
    pub req_client: HttpClient,
    o365_app_id: String,
    data: response::InitO365
}
impl O365Auth {
    pub async fn new(req_url: String, app_id: String) -> Result<Self> {
        let client = HttpClient::builder()
            .cookies()
            .redirect_policy(isahc::config::RedirectPolicy::Follow)
            .build()?;
        let mut initial = client.get_async(&req_url).await?;
        Ok(O365Auth {
            req_client: client,
            o365_app_id: app_id.to_string(),
            data: O365Auth::parse_page(initial.text().await?).await?
        })
    }
    async fn parse_page(data: String) -> Result<response::InitO365> {
        let dom = kuchiki::parse_html()
            .from_utf8()
            .read_from(&mut data.as_bytes())?;
        let mut config_serialized: String = String::from("");
        for script in dom.select("script").unwrap() {
            let as_node = script.as_node();
            let text_node = skip_none!(as_node.first_child());
            let text = text_node.as_text().unwrap().borrow();
            if text[..20] == String::from("//<![CDATA[\n$Config=") {
                config_serialized = String::from(&text[20..text.len()-7]);
            }
        }
        let parsed: response::InitO365 = serde_json::from_str(&config_serialized)?;
        Ok(parsed)
    }
    pub async fn login(&self, email: String, password: String) -> Result<()> {
        let login_params = [("login", &email), ("passwd", &password), ("canary", &self.data.canary), ("ctx", &self.data.sCtx), ("hpgrequestid", &self.data.sessionId), ("flowToken", &self.data.sFT)];
        let post_url: String = format!("https://login.microsoftonline.com/{}/login", self.o365_app_id);

        let body = serde_urlencoded::to_string(&login_params)?;

        let login_req = self.req_client.post_async(&post_url, body).await?;

        if login_req.headers().get("x-ms-request-id").is_some() {
        	// still on M$ server, likely incorrect auth data
        	Err(crate::errors::SmError::InvalidMSCredentials.into())
        } else {
        	Ok(())
        }
    }
}
