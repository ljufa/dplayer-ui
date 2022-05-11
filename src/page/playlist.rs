use api_models::{
    player::*,
    common::*,
    playlist::Playlist,
};
use seed::{prelude::*, *};

#[derive(Debug)]
pub struct Model {
    pub playlists: Vec<Playlist>,
    pub playlist_items: Vec<Song>,
    pub selected_playlist_id: Option<String>,
    pub waiting_response: bool
}
pub enum Msg {
    PlaylistsFetched(fetch::Result<Vec<Playlist>>),
    PlaylistItemsFetched(fetch::Result<Vec<Song>>),

    SendCommand(Command),
    SelectPlaylist(String),

    LoadPlaylistIntoQueue,
}

pub(crate) fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::PlaylistsFetched(get_playlists().await) });
    Model {
        playlists: Vec::new(),
        playlist_items: Vec::new(),
        selected_playlist_id: None,
        waiting_response: false
    }
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, mut model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::PlaylistsFetched(pls) => model.playlists = pls.unwrap_or_default(),
        Msg::SelectPlaylist(pl_id) => {
            model.waiting_response = true;
            model.selected_playlist_id = Some(pl_id.clone());
            orders
                .perform_cmd(async { Msg::PlaylistItemsFetched(get_playlist_items(pl_id).await) });
        }
        Msg::SendCommand(cmd) => log!("Cmd:", cmd),
        Msg::PlaylistItemsFetched(pl_items) => {
            model.waiting_response = false;
            model.playlist_items = pl_items.unwrap_or_default()},
        Msg::LoadPlaylistIntoQueue => {
            model.selected_playlist_id.clone().map(|pl| {
                orders.perform_cmd(async { Msg::SendCommand(Command::LoadPlaylist(pl)) })
            });
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        crate::view_spinner_modal(model.waiting_response),
        view_playlist_selector(model),
        view_playlist_items(model)
    ]
}

fn view_playlist_selector(model: &Model) -> Node<Msg> {
    div![
        C!["transparent", "field"],
        div![
            C!["control"],
            div![
                C!["select"],
                select![
                    option![
                        attrs!(At::Value => "empty"),
                        "--- Select saved playlist ---"
                    ],
                    model
                        .playlists
                        .iter()
                        .map(|pl| option![attrs! {At::Value => &pl.id }, &pl.name]),
                    input_ev(Ev::Change, Msg::SelectPlaylist),
                ]
            ],
            button![
                C!["button is-light"],
                span![C!["icon"], i![C!["fas", "fa-play"]]],
                ev(Ev::Click, |_| Msg::LoadPlaylistIntoQueue)
            ],
        ]
    ]
}
fn view_playlist_items(model: &Model) -> Node<Msg> {
    div![div![
        C![
            "list",
            "has-overflow-ellipsis has-visible-pointer-controls has-hoverable-list-items"
        ],
        model.playlist_items.iter().map(|it| div![
            C!["list-item"],
            div![
                C!["list-item-content"],
                div![C!["list-item-title"], it.info_string()],
            ],
        ])
    ]]
}

pub async fn get_playlists() -> fetch::Result<Vec<Playlist>> {
    Request::new("/api/playlist")
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .json::<Vec<Playlist>>()
        .await
}
pub async fn get_playlist_items(pl_id: String) -> fetch::Result<Vec<Song>> {
    Request::new(format!("/api/playlist/{}", pl_id))
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .json::<Vec<Song>>()
        .await
}
