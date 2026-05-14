pub mod friend;
pub mod friend_request;
pub mod game;
pub mod group;
pub mod health;
pub mod invitation;
pub mod invite;
pub mod library;
pub mod me;
pub mod push;
pub mod session;
pub mod user;

use axum::Router;

use crate::config::app_state::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(push::public_routes())
        .merge(library::public_thumbnail_route())
        .merge(invite::public_routes())
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .merge(me::routes())
        .merge(group::routes())
        .merge(game::routes())
        .merge(user::routes())
        .merge(friend::routes())
        .merge(friend_request::routes())
        .merge(session::routes())
        .merge(push::routes())
        .merge(invitation::routes())
        .merge(invite::protected_routes())
        .merge(library::routes())
}

pub fn sse_routes() -> Router<AppState> {
    group::sse_routes()
}
