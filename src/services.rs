mod api;
mod oidc;

pub use api::{
    add_spirit, edit_spirit, get_spirit_image, search_spirit, upload_spirit_image, user_info,
    WebError, WebResult,
};
pub use oidc::{get_jwks, get_well_known_configuration, login, logout, token, OpenidConfiguration};
