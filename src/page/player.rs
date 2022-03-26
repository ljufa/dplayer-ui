use api_models::player::*;
use seed::{prelude::*, *};

use std::str::FromStr;

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
pub struct Model {
    streamer_status: StreamerStatus,
    player_info: Option<PlayerInfo>,
    current_track_info: Option<Song>,
    waiting_response: bool,
    remote_error: Option<String>,
}
#[derive(Debug)]

pub enum Msg {
    StatusChangeEventReceived(StatusChangeEvent),
    AlbumImageUpdated(Image),
    SendCommand(Command),
    CurrentStatusFetched(fetch::Result<LastStatus>),
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
//     Init
// ------ ------

pub(crate) fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::CurrentStatusFetched(get_current_status().await) });
    Model {
        streamer_status: StreamerStatus {
            source_player: PlayerType::MPD,
            selected_audio_output: AudioOut::SPKR,
            dac_status: DacStatus::default(),
        },
        player_info: None,
        current_track_info: None,
        waiting_response: false,
        remote_error: None,
    }
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, mut model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::AlbumImageUpdated(image) => {
            model.current_track_info.as_mut().unwrap().uri = Some(image.text);
        }
        Msg::CurrentStatusFetched(Ok(st)) => {
            let track = st.current_track_info.clone();
            model.current_track_info = st.current_track_info;
            model.player_info = st.player_info;
            if let Some(status) = st.streamer_status {
                model.streamer_status = status;
            }
            if let Some(track) = track {
                orders.perform_cmd(async { update_album_cover(track).await });
            }
        }
        Msg::StatusChangeEventReceived(StatusChangeEvent::CurrentTrackInfoChanged(track_info)) => {
            model.waiting_response = false;
            let ps = track_info.clone();
            model.current_track_info = Some(track_info);
            orders.perform_cmd(async { update_album_cover(ps).await });
        }

        Msg::StatusChangeEventReceived(StatusChangeEvent::PlayerInfoChanged(player_info)) => {
            model.waiting_response = false;
            model.player_info = Some(player_info);
        }

        Msg::StatusChangeEventReceived(StatusChangeEvent::StreamerStatusChanged(
            streamer_status,
        )) => {
            model.waiting_response = false;
            model.streamer_status = streamer_status;
        }

        Msg::StatusChangeEventReceived(StatusChangeEvent::Error(error)) => {
            model.remote_error = Some(error)
        }
        Msg::StatusChangeEventReceived(_) => {}
        Msg::SendCommand(cmd) => {
            log!("Player {}", cmd);
            match cmd {
                Command::SwitchToPlayer(_) => model.waiting_response = true,
                Command::SetVol(vol) => model.streamer_status.dac_status.volume = vol,
                _ => (),
            }
        }
        _ => {
            log!("Unknown variant");
        }
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
            St::MinHeight => "95vh"
        },
        div![
            style! {
                St::Background => "rgba(86, 92, 86, 0.507)",
                St::MinHeight => "95vh"
            },
            view_track_info(
                model.current_track_info.as_ref(),
                model.player_info.as_ref()
            ),
            view_track_progress_bar(model.player_info.as_ref()),
            view_volume_slider(&model.streamer_status.dac_status),
            view_controls(model.player_info.as_ref()),
            view_controls_down(model.player_info.as_ref(), &model.streamer_status),
        ]
    ]
}

fn view_track_info(status: Option<&Song>, player_info: Option<&PlayerInfo>) -> Node<Msg> {
    if let Some(ps) = status {
        div![
            style! {
                St::MinHeight => "300px",
                St::PaddingTop => "2rem"
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
                // IF!(ps.name.is_some() && ps.name != ps.title =>
                // div![
                //     C!["level-item"],
                //     div![
                //         p![
                //             C!["has-text-light has-background-dark-transparent"],
                //             ps.name.as_ref().map_or("NA", |f| f)
                //         ],
                //     ],
                // ]),
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
                if ps.title.is_none(){
                    div![
                        C!["level-item"],
                        div![p![
                            C!["has-text-light has-background-dark-transparent"],
                            ps.file.clone()
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
        div![
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
                C!["progress", "is-small", "is-success"],
                attrs! {"value"=> player_info.time.0.as_secs()},
                attrs! {"max"=> player_info.time.1.as_secs()},
                player_info.time.0.as_secs()
            ],
        ]
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
        AudioOut::HEAD => "headset",
    };
    let shuffle = player_info.map_or("shuffle", |r| {
        if r.random.unwrap_or(false) {
            "shuffle"
        } else {
            "format_list_numbered"
        }
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
                            C!["button"],
                            span![C!["icon"], i![C!("material-icons"), shuffle]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::RandomToggle)),
                        ]
                    ],
                    div![
                        C!["level-item"],
                        button![
                            C!["button"],
                            span![C!["icon"], i![C!("material-icons"), audio_out]],
                            ev(Ev::Click, |_| Msg::SendCommand(Command::ChangeAudioOutput))
                        ]
                    ],
                ]
            ]
        ]
    ]
}

fn view_volume_slider(dac_status: &DacStatus) -> Node<Msg> {
    div![
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
            format!("Volume: {}/{}", dac_status.volume, 255)
        ],
        input![
            C!["slider", "is-fullwidth", "is-success"],
            style! {
                St::PaddingRight => "1.2rem"
            },
            attrs! {"value"=> dac_status.volume},
            attrs! {"step"=> 1},
            attrs! {"max"=> 255},
            attrs! {"min"=> 140},
            attrs! {"type"=> "range"},
            attrs! {"disabled"=> "disabled"},
            input_ev(Ev::Change, move |selected| Msg::SendCommand(
                Command::SetVol(u8::from_str(selected.as_str()).unwrap())
            )),
        ],
    ]
}

#[allow(clippy::logic_bug)]
#[allow(dead_code)]
fn view_player_switch(model: &Model) -> Node<Msg> {
    let pt = model.streamer_status.source_player;

    div![
        C!["transparent"],
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
    ]
}

fn get_background_image(model: &Model) -> String {
    if let Some(ps) = model.current_track_info.as_ref() {
        format!("url({})", ps.uri.as_ref().map_or("", |f| f))
    } else {
        String::new()
    }
}

pub async fn get_current_status() -> fetch::Result<LastStatus> {
    Request::new("/api/status")
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .json::<LastStatus>()
        .await
}

pub async fn update_album_cover(track: Song) -> Msg {
    if track.album.is_some() && track.artist.is_some() {
        let ai = get_album_image_from_lastfm_api(track.album.unwrap(), track.artist.unwrap()).await;
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
}
async fn get_album_image_from_lastfm_api(album: String, artist: String) -> Option<Image> {
    let response = fetch(format!("http://ws.audioscrobbler.com/2.0/?method=album.getinfo&album={}&artist={}&api_key=3b3df6c5dd3ad07222adc8dd3ccd8cdc&format=json", album, artist)).await;
    if let Ok(response) = response {
        let info = response.json::<AlbumInfo>().await;
        if let Ok(info) = info {
            info.album
                .image
                .into_iter()
                .find(|i| i.size == "mega" && !i.text.is_empty())
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
