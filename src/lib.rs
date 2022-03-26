use std::rc::Rc;

use api_models::player::StatusChangeEvent;
use page::settings;
use seed::{prelude::*, *};
use strum_macros::IntoStaticStr;
mod page;

#[cfg(feature = "remote")]
const WS_URL: &str = "ws://192.168.5.59:8000/api/ws";

#[cfg(feature = "local")]
const WS_URL: &str = "ws://localhost:8000/api/ws";
const SETTINGS: &str = "settings";
const PLAYLIST: &str = "playlist";
const QUEUE: &str = "queue";
const FIRST_SETUP: &str = "setup";

const PLAYER: &str = "player";
// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
struct Model {
    base_url: Url,
    page: Page,
    web_socket: WebSocket,
    web_socket_reconnector: Option<StreamHandle>,
}

pub enum Msg {
    WebSocketOpened,
    CloseWebSocket,
    WebSocketClosed(CloseEvent),
    WebSocketFailed,
    ReconnectWebSocket(usize),
    UrlChanged(subs::UrlChanged),
    StatusChangeEventReceived(StatusChangeEvent),
    Settings(page::settings::Msg),
    Player(page::player::Msg),
    Playlist(page::playlist::Msg),
    Queue(page::queue::Msg),
}

// ------ Page ------
#[derive(Debug, IntoStaticStr)]
enum Page {
    Home,
    Settings(page::settings::Model),
    Player(page::player::Model),
    Playlist(page::playlist::Model),
    Queue(page::queue::Model),
    NotFound,
}
impl Page {
    fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Self {
        let slice = url.remaining_hash_path_parts();
        log!("Init", slice);
        match slice.as_slice() {
            [FIRST_SETUP] => Self::Home,
            [SETTINGS] => {
                Self::Settings(page::settings::init(url, &mut orders.proxy(Msg::Settings)))
            }
            [PLAYLIST] => {
                Self::Playlist(page::playlist::init(url, &mut orders.proxy(Msg::Playlist)))
            }
            [QUEUE] => Self::Queue(page::queue::init(url, &mut orders.proxy(Msg::Queue))),
            [PLAYER] | [] => Self::Player(page::player::init(url, &mut orders.proxy(Msg::Player))),
            _ => Self::NotFound,
        }
    }
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    fn settings(self) -> Url {
        self.base_url().add_hash_path_part(SETTINGS)
    }
    fn settings_abs() -> Url {
        Url::new().add_hash_path_part(SETTINGS)
    }
    fn queue_abs() -> Url {
        Url::new().add_hash_path_part(QUEUE)
    }
    fn playlist_abs() -> Url {
        Url::new().add_hash_path_part(PLAYLIST)
    }

    fn player_abs() -> Url {
        Url::new().add_hash_path_part(PLAYER)
    }
}

// ------ ------
//     Init
// ------ ------

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url.clone()));
    Model {
        base_url: url.to_base_url(),
        page: Page::init(url, orders),
        web_socket: create_websocket(orders),
        web_socket_reconnector: None,
    }
}

// ------ ------
//    Update
// ------ ------

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::WebSocketOpened => {
            model.web_socket_reconnector = None;
            log!("WebSocket connection is open now");
        }

        Msg::CloseWebSocket => {
            model.web_socket_reconnector = None;
            model
                .web_socket
                .close(None, Some("user clicked Close button"))
                .unwrap();
        }

        Msg::WebSocketClosed(close_event) => {
            log!("==================");
            log!("WebSocket connection was closed:");
            log!("Clean:", close_event.was_clean());
            log!("Code:", close_event.code());
            log!("Reason:", close_event.reason());
            log!("==================");

            // Chrome doesn't invoke `on_error` when the connection is lost.
            if !close_event.was_clean() && model.web_socket_reconnector.is_none() {
                model.web_socket_reconnector = Some(
                    orders.stream_with_handle(streams::backoff(None, Msg::ReconnectWebSocket)),
                );
            }
        }

        Msg::WebSocketFailed => {
            log!("WebSocket failed");
            if model.web_socket_reconnector.is_none() {
                model.web_socket_reconnector = Some(
                    orders.stream_with_handle(streams::backoff(None, Msg::ReconnectWebSocket)),
                );
            }
        }

        Msg::ReconnectWebSocket(retries) => {
            log!("Reconnect attempt:", retries);
            model.web_socket = create_websocket(orders);
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => model.page = Page::init(url, orders),

        Msg::StatusChangeEventReceived(chg_ev) => {
            if let Page::Player(model) = &mut model.page {
                page::player::update(
                    page::player::Msg::StatusChangeEventReceived(chg_ev),
                    model,
                    &mut orders.proxy(Msg::Player),
                );
            }
        }

        Msg::Settings(msg) => {
            if let Page::Settings(sett_model) = &mut model.page {
                if let settings::Msg::SendCommand(cmd) = &msg {
                    model.web_socket.send_json(cmd).unwrap();
                }
                page::settings::update(msg, sett_model, &mut orders.proxy(Msg::Settings));
            }
        }

        Msg::Player(msg) => {
            log!("Lib::Player {}", msg);
            if let Page::Player(player_model) = &mut model.page {
                if let page::player::Msg::SendCommand(cmd) = &msg {
                    model.web_socket.send_json(cmd).unwrap();
                }
                page::player::update(msg, player_model, &mut orders.proxy(Msg::Player));
            }
        }

        Msg::Playlist(msg) => {
            if let Page::Playlist(player_model) = &mut model.page {
                if let page::playlist::Msg::SendCommand(cmd) = &msg {
                    model.web_socket.send_json(cmd).unwrap();
                }

                page::playlist::update(msg, player_model, &mut orders.proxy(Msg::Playlist));
            }
        }
        Msg::Queue(msg) => {
            if let Page::Queue(player_model) = &mut model.page {
                if let page::queue::Msg::SendCommand(cmd) = &msg {
                    model.web_socket.send_json(cmd).unwrap();
                }
                page::queue::update(msg, player_model, &mut orders.proxy(Msg::Queue));
            }
        }
    }
}

// ------ ------
//     View
// ------ ------
fn view(model: &Model) -> impl IntoNodes<Msg> {
    div![
        C!["container"],
        view_navigation_tabs(&model.page),
        view_content(&model.page, &model.base_url),
    ]
}

// ----- view_content ------

fn view_content(page: &Page, base_url: &Url) -> Node<Msg> {
    match page {
        Page::Home => page::home::view(base_url),
        Page::NotFound => page::not_found::view(),
        Page::Settings(model) => page::settings::view(model).map_msg(Msg::Settings),
        Page::Player(model) => page::player::view(model).map_msg(Msg::Player),
        Page::Playlist(model) => page::playlist::view(model).map_msg(Msg::Playlist),
        Page::Queue(model) => page::queue::view(model).map_msg(Msg::Queue),
    }
}
fn view_navigation_tabs(page: &Page) -> Node<Msg> {
    let page_name: &str = page.into();
    div![
        C!["tabs", "is-toggle", "is-centered", "is-fullwidth"],
        ul![
            li![
                IF!(page_name == "Player" => C!["is-active"]),
                a![span![
                    C!["icon", "is-small"],
                    i![C!["material-icons"], attrs!("aria-hidden" => "true"), "music_note"],
                ]],
                ev(Ev::Click, |_| { Urls::player_abs().go_and_load() }),
            ],
            li![
                IF!(page_name == "Queue" => C!["is-active"]),
                a![span![
                    C!["icon", "is-small"],
                    i![C!["material-icons"], attrs!("aria-hidden" => "true"), "queue_music"],
                ]],
                ev(Ev::Click, |_| { Urls::queue_abs().go_and_load() }),
            ],
            li![
                IF!(page_name == "Playlist" => C!["is-active"]),
                a![span![
                    C!["icon", "is-small"],
                    i![C!["material-icons"], attrs!("aria-hidden" => "true"), "library_music"],
                ],],
                ev(Ev::Click, |_| { Urls::playlist_abs().go_and_load() }),
            ],
            li![
                IF!(page_name == "Settings" => C!["is-active"]),
                a![span![
                    C!["icon", "is-small"],
                    i![C!["material-icons"], attrs!("aria-hidden" => "true"), "tune"],
                ]],
                ev(Ev::Click, |_| { Urls::settings_abs().go_and_load() }),
            ],
        ]
    ]
}
pub fn view_spinner_modal<Ms>(active: bool) -> Node<Ms> {
    // spinner
    div![
        C!["modal", IF!(active => "is-active")],
        div![C!["modal-background"]],
        div![
            C!["modal-content"],
            div![
                C!("sk-fading-circle"),
                div![C!["sk-circle1 sk-circle"]],
                div![C!["sk-circle2 sk-circle"]],
                div![C!["sk-circle3 sk-circle"]],
                div![C!["sk-circle4 sk-circle"]],
                div![C!["sk-circle5 sk-circle"]],
                div![C!["sk-circle6 sk-circle"]],
                div![C!["sk-circle7 sk-circle"]],
                div![C!["sk-circle8 sk-circle"]],
                div![C!["sk-circle9 sk-circle"]],
                div![C!["sk-circle10 sk-circle"]],
                div![C!["sk-circle11 sk-circle"]],
                div![C!["sk-circle12 sk-circle"]],
            ]
        ]
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

fn create_websocket(orders: &impl Orders<Msg>) -> WebSocket {
    let msg_sender = orders.msg_sender();

    WebSocket::builder(WS_URL, orders)
        .on_open(|| Msg::WebSocketOpened)
        .on_message(move |msg| decode_message(msg, msg_sender))
        .on_close(Msg::WebSocketClosed)
        .on_error(|| Msg::WebSocketFailed)
        .build_and_open()
        .unwrap()
}

fn decode_message(message: WebSocketMessage, msg_sender: Rc<dyn Fn(Option<Msg>)>) {
    let msg_text = message.text();
    if msg_text.is_ok() {
        let msg = message
            .json::<StatusChangeEvent>()
            .unwrap_or_else(|_| panic!("Failed to decode WebSocket text message: {:?}", msg_text));
        msg_sender(Some(Msg::StatusChangeEventReceived(msg)));
    }
}
