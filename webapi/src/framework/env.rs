#[derive(Clone)]
pub struct Env {
    pub google_client_id: String,
    pub google_redirect_uri: String,
    pub google_client_secret: String,
    pub db_url: String,
}

impl Env {
    pub fn new() -> Env {
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .expect("環境変数にGOOGLE_CLIENT_IDをセットしてください。");
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("環境変数にGOOGLE_CLIENT_SECRETをセットしてください。");
        let google_redirect_uri =
            std::env::var("REDIRECT_URI").expect("環境変数にREDIRECT_URIをセットしてください。");
        let db_url =
            std::env::var("DB_URL").expect("環境変数にDB_URLをセットしてください。");
        Env {
            google_client_id,
            google_redirect_uri,
            google_client_secret,
            db_url
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}