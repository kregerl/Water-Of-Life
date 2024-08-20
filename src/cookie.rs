use tower_cookies::{cookie::SameSite, Cookie};

pub fn create_token_cookie<'a>(key: &'a str, token: String) -> Cookie<'a> {
    let mut cookie = Cookie::new(key, token);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie
}