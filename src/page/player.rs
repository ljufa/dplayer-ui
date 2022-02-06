use seed::{prelude::*, *};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use std::{
    fmt::{Display, Write},
    rc::Rc,
    str::FromStr,
    time::Duration,
};

const WS_URL: &str = "ws://192.168.5.59:8000/api/player";

// ------ ------
//     Init
// ------ ------

pub(crate) fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        streamer_status: StreamerStatus {
            source_player: PlayerType::MPD,
            selected_audio_output: AudioOut::SPKR,
            dac_status: DacStatus {
                volume: 0,
                filter: FilterType::SharpRollOff,
                sound_sett: 0,
            },
        },
        player_info: None,
        current_track_info: None,
        input_text: String::new(),
        web_socket: create_websocket(orders),
        web_socket_reconnector: None,
        waiting_response: false,
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
pub struct Model {
    streamer_status: StreamerStatus,
    player_info: Option<PlayerInfo>,
    current_track_info: Option<CurrentTrackInfo>,
    input_text: String,
    web_socket: WebSocket,
    web_socket_reconnector: Option<StreamHandle>,
    waiting_response: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct StreamerStatus {
    pub selected_audio_output: AudioOut,
    pub source_player: PlayerType,
    pub dac_status: DacStatus,
}

#[derive(Debug, serde::Deserialize)]
pub struct DacStatus {
    pub volume: u8,
    pub filter: FilterType,
    pub sound_sett: u8,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CurrentTrackInfo {
    pub filename: Option<String>,
    pub name: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub date: Option<String>,
    pub uri: Option<String>,
}
#[derive(Debug, serde::Deserialize, Clone)]
pub struct PlayerInfo {
    pub state: Option<PlayerState>,
    pub random: bool,
    pub audio_format_rate: Option<u32>,
    pub audio_format_bit: Option<u8>,
    pub audio_format_channels: Option<u8>,
    pub time: Option<(Duration, Duration)>,
}

impl PlayerInfo {
    pub fn format_time(&self) -> String {
        if let Some(time) = self.time {
            return format!("{} / {}", dur_to_string(time.0), dur_to_string(time.1));
        } else {
            return "00:00:00 / 00:00:00".to_string();
        }
    }
}
fn dur_to_string(duration: Duration) -> String {
    let mut result = "00:00:00".to_string();
    let secs = duration.as_secs();
    if secs > 0 {
        let seconds = secs % 60;
        let minutes = (secs / 60) % 60;
        let hours = (secs / 60) / 60;
        result = format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds).to_string();
    }
    return result;
}

#[derive(Debug, PartialEq, serde::Deserialize, IntoStaticStr, Clone)]
pub enum PlayerState {
    PLAYING,
    PAUSED,
    STOPPED,
}

#[derive(Debug, serde::Deserialize, IntoStaticStr)]
pub enum AudioOut {
    SPKR,
    HEAD,
}
#[derive(
    Debug, serde::Deserialize, serde::Serialize, PartialEq, EnumString, IntoStaticStr, EnumIter,
)]
pub enum FilterType {
    SharpRollOff,
    SlowRollOff,
    ShortDelaySharpRollOff,
    ShortDelaySlowRollOff,
    SuperSlow,
}
#[derive(Debug, Copy, PartialEq, Clone, serde::Deserialize, serde::Serialize, IntoStaticStr)]
pub enum PlayerType {
    SPF,
    MPD,
    LMS,
}

#[derive(Debug, serde::Serialize)]
pub enum Command {
    VolUp,
    VolDown,
    Next,
    Prev,
    Pause,
    Play,
    RandomToggle,

    SwitchToPlayer(PlayerType),
    Filter(FilterType),
    SetVol(u8),
    Sound(u8),
    PowerOff,
    ChangeAudioOutput,
    Rewind(i8),
}

pub enum Msg {
    WebSocketOpened,
    CurrentTrackInfoChanged(CurrentTrackInfo),
    StreamerStatusChanged(StreamerStatus),
    PlayerInfoChaged(PlayerInfo),
    CloseWebSocket,
    WebSocketClosed(CloseEvent),
    WebSocketFailed,
    ReconnectWebSocket(usize),
    SendCommand(Command),
    AlbumImageUpdated(Image),
}
#[derive(Debug, serde::Deserialize)]
pub struct AlbumInfo {
    pub album: Album,
}
#[derive(Debug, serde::Deserialize)]
pub struct Album {
    image: Vec<Image>,
}
#[derive(Debug, serde::Deserialize)]
pub struct Image {
    size: String,
    #[serde(rename = "#text")]
    text: String,
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, mut model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::WebSocketOpened => {
            model.web_socket_reconnector = None;
            log!("WebSocket connection is open now");
        }
        Msg::CurrentTrackInfoChanged(message) => {
            model.waiting_response = false;
            let ps = message;
            model.current_track_info = Some(ps.clone());
            if ps.uri.is_none() {
                orders.perform_cmd(async {
                    if ps.album.is_some() && ps.artist.is_some() {
                        let ai =
                            get_album_image_from_lastfm_api(ps.album.unwrap(), ps.artist.unwrap())
                                .await;
                        match ai {
                            Some(ai) => Msg::AlbumImageUpdated(ai),
                            None => Msg::AlbumImageUpdated(Image {
                                size: "mega".to_string(),
                                text: "/no_album.png".to_string(),
                            }),
                        }
                    } else {
                        Msg::AlbumImageUpdated(Image {
                            size: "mega".to_string(),
                            text: "/no_album.png".to_string(),
                        })
                    }
                });
            }
        }
        Msg::AlbumImageUpdated(image) => {
            model.current_track_info.as_mut().unwrap().uri = Some(image.text);
        }
        Msg::StreamerStatusChanged(message) => {
            model.waiting_response = false;
            model.streamer_status = message;
        }
        Msg::PlayerInfoChaged(message) => {
            model.waiting_response = false;
            model.player_info = Some(message);
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

        Msg::SendCommand(cmd) => {
            match cmd {
                Command::SwitchToPlayer(_) => model.waiting_response = true,
                Command::SetVol(vol) => model.streamer_status.dac_status.volume = vol,
                _ => (),
            }

            model.web_socket.send_json(&cmd).unwrap();
        }
        _ => {}
    }
}

async fn get_album_image_from_lastfm_api(album: String, artist: String) -> Option<Image> {
    let response = fetch(format!("http://ws.audioscrobbler.com/2.0/?method=album.getinfo&album={}&artist={}&api_key=3b3df6c5dd3ad07222adc8dd3ccd8cdc&format=json", album, artist)).await;
    if let Ok(response) = response {
        let info = response.json::<AlbumInfo>().await;
        if let Ok(info) = info {
            info.album
                .image
                .into_iter()
                .filter(|i| i.size == "mega" && i.text.len() > 0)
                .next()
        } else {
            log!("Failed to get album info {}", info);
            None
        }
    } else if let Err(e) = response {
        log!("Error getting album info from last.fm {}", e);
        None
    } else {
        None
    }
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
    if let Ok(msg_text) = msg_text {
        if msg_text.contains("title") || msg_text.contains("filename") {
            let msg = message
                .json::<CurrentTrackInfo>()
                .expect("Failed to decode WebSocket text message");
            msg_sender(Some(Msg::CurrentTrackInfoChanged(msg)));
        } else if msg_text.contains("source_player") || msg_text.contains("volume") {
            let msg = message
                .json::<StreamerStatus>()
                .expect("Failed to decode WebSocket text message");
            msg_sender(Some(Msg::StreamerStatusChanged(msg)));
        } else if msg_text.contains("time") {
            let msg = message
                .json::<PlayerInfo>()
                .expect("Failed to decode WebSocket text message");
            msg_sender(Some(Msg::PlayerInfoChaged(msg)));
        }
    }
}

fn get_background_image(model: &Model) -> String {
    if let Some(ps) = model.current_track_info.as_ref() {
        format!("url({})", ps.uri.as_ref().map_or("", |f| f))
    } else {
        String::new()
    }
}

// ------ ------
//     View
// ------ ------
pub(crate) fn view(model: &Model) -> Node<Msg> {
    div![
        style! {
            St::BackgroundImage => get_background_image(model),
            St::BackgroundRepeat => "no-repeat",
            St::BackgroundSize => "cover",
            St::MinHeight => "100vh"
        },
        // spinner
        div![
            C!["modal", IF!(model.waiting_response => "is-active")],
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
        ],
        div![
            style! {
                St::Background => "rgba(86, 92, 86, 0.507)",
                St::MinHeight => "100vh"
            },
            view_track_info(
                model.current_track_info.as_ref(),
                model.player_info.as_ref()
            ),
            view_track_progress_bar(model.player_info.as_ref()),
            view_controls(model.player_info.as_ref()),
            view_volume_slider(&model.streamer_status.dac_status),
            view_controls_down(model.player_info.as_ref(), &model.streamer_status),
        ]
    ]
}

fn view_track_info(
    status: Option<&CurrentTrackInfo>,
    player_info: Option<&PlayerInfo>,
) -> Node<Msg> {
    if let Some(ps) = status {
        div![
            style! {
                St::MinHeight => "300px"
            },
            C!["transparent"],
            nav![
                C!["level", "is-flex-direction-column"],
                IF!(ps.title.is_some() =>
                div![
                    C!["level-item has-text-centered"],
                    div![
                        p![
                            C!["is-size-3 has-text-light has-background-dark-transparent"],
                            ps.title.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                IF!(ps.name.is_some() && ps.name != ps.title =>
                div![
                    C!["level-item"],
                    div![
                        p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.name.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                IF!(ps.album.is_some() =>
                div![
                    C!["level-item"],
                    div![
                        p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.album.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                IF!(ps.artist.is_some() =>
                div![
                    C!["level-item"],
                    div![
                        p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.artist.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                if ps.title.is_none() && ps.filename.is_some() {
                    div![
                        C!["level-item"],
                        div![p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.filename.as_ref().map_or("NA", |f| f)
                        ],],
                    ]
                } else {
                    empty!()
                },
            ],
            nav![
                C!["level", "is-flex-direction-column"],
                IF!(ps.genre.is_some() =>
                div![
                    C!["level-item"],
                    div![
                        p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.genre.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                IF!(ps.date.is_some() =>
                div![
                    C!["level-item"],
                    div![
                        p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.date.as_ref().map_or("NA", |f| f)
                        ],
                    ],
                ]),
                if let Some(pi) = player_info {
                    div![
                        C!["level-item"],
                        IF!(pi.audio_format_rate.is_some() =>
                            div![p![
                            C!["has-text-light has-background-dark-transparent"],
                            format!("Freq: {} | Bit: {} | Ch: {}", pi.audio_format_rate.map_or(0, |f|f),
                            pi.audio_format_bit.map_or(0, |f|f), pi.audio_format_channels.map_or(0,|f|f))
                        ]]),
                    ]
                } else {
                    empty!()
                }
            ],
        ]
    } else {
        empty!()
    }
}
fn view_track_progress_bar(player_info: Option<&PlayerInfo>) -> Node<Msg> {
    if let Some(player_info) = player_info {
        if let Some((current, total)) = player_info.time {
            div![div![
                style! {
                    St::Padding => "1.2rem",
                },
                C!["has-text-centered"],
                span![
                    C![
                        "is-size-6",
                        "has-text-light",
                        "has-background-dark-transparent"
                    ],
                    player_info.format_time()
                ],
                progress![
                    C!["progress", "is-small", "is-info"],
                    attrs! {"value"=> current.as_secs()},
                    attrs! {"max"=> total.as_secs()},
                    current.as_secs()
                ],
            ],]
        } else {
            empty!()
        }
    } else {
        empty!()
    }
}
fn view_controls(player_info: Option<&PlayerInfo>) -> Node<Msg> {
    let playing = player_info.map_or(false, |f| {
        f.state
            .as_ref()
            .map_or(false, |f| *f == PlayerState::PLAYING)
    });
    div![
        C!["transparent"],
        nav![
            C!["level is-mobile"],
            div![
                C!["level-left"],
                div![
                    C!["field", "is-grouped"],
                    div![
                        C!["level-item"],
                        button![
                            IF!(playing => attrs!{"disabled"=>true}),
                            C!["button"],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::Play)),
                            span![C!("icon"), i![C!("fas fa-play")]]
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            IF!(!playing => attrs!{"disabled"=>true}),
                            C!["button"],
                            span![C!("icon"), i![C!("fas fa-stop")]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::Pause))
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!("icon"), i![C!("fas fa-step-backward")]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::Prev))
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!("icon"), i![C!("fas fa-step-forward")]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::Next))
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!("icon"), i![C!("fas fa-volume-down")]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::VolDown))
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!("icon"), i![C!("fas fa-volume-up")]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::VolUp))
                        ]
                    ],
                ],
            ]
        ]
    ]
}
fn view_controls_down(
    player_info: Option<&PlayerInfo>,
    streamer_status: &StreamerStatus,
) -> Node<Msg> {
    let audio_out = match streamer_status.selected_audio_output {
        AudioOut::SPKR => "speaker",
        AudioOut::HEAD => "headphones",
    };
    let shuffle = player_info.map_or(
        "shuffle",
        |r| if r.random { "shuffle_on" } else { "shuffle" },
    );

    div![
        C!["transparent"],
        nav![
            C!["level is-mobile"],
            div![
                C!["level-left"],
                div![
                    C!["field", "is-grouped"],
                    div![
                        C!["level-item"],
                        button![
                            C!["button", IF!(shuffle == "shuffle_on" => "is-active")],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::RandomToggle)),
                            span![C!("icon"), i![C!("material-icons"), shuffle]]
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!("icon"), i![C!("material-icons"), audio_out]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::ChangeAudioOutput))
                        ]
                    ],
                    div![
                        C!["level-item"],
                        div![
                            C!["select"],
                            select![
                                FilterType::iter()
                                    .map(|f| {
                                        let fs: &'static str = f.into();
                                        fs
                                    })
                                    .map(|fs| option![attrs! {At::Value => fs }, fs]),
                                input_ev(Ev::Change, move |selected| Msg::SendCommand(
                                    Command::Filter(
                                        FilterType::from_str(selected.as_str()).unwrap()
                                    )
                                )),
                            ],
                        ],
                    ],
                ],
            ]
        ]
    ]
}
fn view_volume_slider(dac_status: &DacStatus) -> Node<Msg> {
    div![
        C!["transparent"],
        div![div![
            C!["has-text-light has-background-dark-transparent field is-grouped"],
            label!["Volume:"],
            input![
                C!["slider is-fullwidth is-success"],
                attrs! {"value"=> dac_status.volume},
                attrs! {"step"=> 1},
                attrs! {"max"=> 255},
                attrs! {"min"=> 140},
                attrs! {"type"=> "range"},
                input_ev(Ev::Change, move |selected| Msg::SendCommand(
                    Command::SetVol(u8::from_str(selected.as_str()).unwrap())
                )),
            ],
            span![format!("{}/{}", dac_status.volume, 255)]
        ]],
    ]
}

pub fn view_player_switch(model: &Model) -> Node<Msg> {
    let pt = model.streamer_status.source_player;
    nav![
        C!["level is-mobile"],
        div![
            C!["level-left"],
            div![
                C!["level-item"],
                button![
                    IF!(pt == PlayerType::MPD => attrs!{"disabled"=>true}),
                    C!["button", "is-small"],
                    span![C!("icon"), i![C!("fas fa-file-audio")]],
                    span!("MPD"),
                    ev(Ev::Click, |_| Msg::SendCommand(Command::SwitchToPlayer(
                        PlayerType::MPD
                    )))
                ]
            ],
            div![
                C!["level-item"],
                button![
                    IF!(true || pt == PlayerType::SPF=> attrs!{"disabled"=>true}),
                    C!["button", "is-small"],
                    span![C!("icon"), i![C!("fab fa-spotify")]],
                    span!("Spotify"),
                    ev(Ev::Click, |_| Msg::SendCommand(Command::SwitchToPlayer(
                        PlayerType::SPF
                    )))
                ]
            ],
            div![
                C!["level-item"],
                button![
                    IF!(pt == PlayerType::LMS=> attrs!{"disabled"=>true}),
                    C!["button", "is-small"],
                    span![C!("icon"), i![C!("fas fa-compact-disc")]],
                    span!("LMS"),
                    ev(Ev::Click, |_| Msg::SendCommand(Command::SwitchToPlayer(
                        PlayerType::LMS
                    )))
                ]
            ],
        ]
    ]
}
