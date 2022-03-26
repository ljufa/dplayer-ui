use api_models::{
    player::{Command, FilterType},
    settings::*,
};
use seed::{prelude::*, *};
use std::{str::FromStr};
use strum::IntoEnumIterator;



use crate::Urls;

const API_SETTINGS_PATH: &str = "/api/settings";

// ------ ------
//     Model
#[derive(Debug)]
pub struct Model {
    settings: Settings,
    waiting_response: bool,
}

#[derive(Debug)]
pub enum Msg {
    // ---- on off toggles ----
    ToggleDacEnabled,
    ToggleSpotifyEnabled,
    ToggleLmsEnabled,
    ToggleMpdEnabled,

    // ---- Input capture ----
    InputMpdHostChange(String),
    InputMpdPortChange(u32),
    InputLMSHostChange,
    InputSpotifyDeviceNameChange,
    InputSpotifyUsernameChange,
    InputSpotifyPasswordChange,

    // --- Buttons ----
    SaveSettings,
    SettingsSaved(fetch::Result<Settings>),

    RemoteConfiguration(Settings),
    SendCommand(Command),
}

// ------ ------
//     Init
// ------ ------
pub(crate) fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Model {
    log!("Settings Init called");
    orders.perform_cmd(async {
        let response = fetch(API_SETTINGS_PATH)
            .await
            .expect("Failed to get settings from dplayer backend");

        let sett = response
            .json::<Settings>()
            .await
            .expect("failed to deserialize to Configuration");
        Msg::RemoteConfiguration(sett)
    });
    Model {
        settings: Settings::default(),
        waiting_response: false,
    }
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SaveSettings => {
            let settings = model.settings.clone();
            orders.perform_cmd(async { Msg::SettingsSaved(save_settings(settings).await) });
            model.waiting_response = true;
        }
        Msg::ToggleDacEnabled => {
            model.settings.dac_settings.enabled = !model.settings.dac_settings.enabled;
        }
        Msg::ToggleSpotifyEnabled => {
            model.settings.spotify_settings.enabled = !model.settings.spotify_settings.enabled;
        }
        Msg::ToggleLmsEnabled => {
            model.settings.lms_settings.enabled = !model.settings.lms_settings.enabled;
        }
        Msg::ToggleMpdEnabled => {
            model.settings.mpd_settings.enabled = !model.settings.mpd_settings.enabled;
        }
        Msg::InputMpdHostChange(value) => {
            model.settings.mpd_settings.server_host = value;
        }
        Msg::InputMpdPortChange(value) => {
            model.settings.mpd_settings.server_port = value;
        }
        Msg::InputLMSHostChange => {}
        Msg::InputSpotifyDeviceNameChange => {}
        Msg::InputSpotifyUsernameChange => {}
        Msg::InputSpotifyPasswordChange => {}
        Msg::RemoteConfiguration(sett) => {
            model.settings = sett;
        }
        Msg::SettingsSaved(saved) => {
            log!("Saved settings with result {}", saved);
            model.waiting_response = false;
        }
        _ => {}
    }
}

async fn save_settings(settings: Settings) -> fetch::Result<Settings> {
    Request::new(API_SETTINGS_PATH)
        .method(Method::Post)
        .json(&settings)?
        .fetch()
        .await?
        .check_status()?
        .json::<Settings>()
        .await
}

// ------ ------
//     View
// ------ ------

pub(crate) fn view(model: &Model) -> Node<Msg> {
    div![
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
        view_settings(&model.settings)
    ]
}

// ------ configuration ------

fn view_settings(settings: &Settings) -> Node<Msg> {
    div![
        section![
            C!["section"],
            h1![C!["title"], "Players"],
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleMpdEnabled),
                input![
                    C!["control", "switch"],
                    attrs! {
                        At::Name => "mpd_cb"
                        At::Type => "checkbox"
                        At::Checked => settings.mpd_settings.enabled.as_at_value(),
                    },
                ],
                label![
                    C!("label"),
                    "Enable Music Player Demon integration?",
                    attrs! {
                        At::For => "mpd_cb"
                    }
                ]
            ],
            IF!(settings.mpd_settings.enabled => view_mpd(&settings.mpd_settings)),
            div![
                C!["field", "control"],
                ev(Ev::Click, |_| Msg::ToggleLmsEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "lms_cb"
                        At::Type => "checkbox"
                        At::Checked => settings.lms_settings.enabled.as_at_value(),
                    },
                ],
                label![
                    C!("label"),
                    "Enable Logitech Media Server integration?",
                    attrs! {
                        At::For => "lms_cb"
                    }
                ]
            ],
            IF!(settings.lms_settings.enabled => view_lms(&settings.lms_settings)),
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleSpotifyEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "spotify_cb"
                        At::Type => "checkbox"
                        At::Checked => settings.spotify_settings.enabled.as_at_value(),
                    },
                ],
                label![
                    C!["label"],
                    "Enable spotify integration (for premium Spotify accounts only)?",
                    attrs! {
                        At::For => "spotify_cb"
                    }
                ]
            ],
            IF!(settings.spotify_settings.enabled => view_spotify(&settings.spotify_settings))
        ],
        section![
            C!["section"],
            h1![C!["title"], "Alsa"],
            div![
                C!["field"],
                label!["Alsa audio device:", C!["label"]],
                div![
                    C!["select"],
                    select![settings
                        .alsa_settings
                        .available_alsa_pcm_devices
                        .iter()
                        .map(|d| option![attrs! {At::Value => d.0}, d.1],)],
                ],
            ]
        ],
        section![
            C!["section"],
            h1![C!["title"], "Dac"],
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleDacEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "dac_cb"
                        At::Type => "checkbox"
                        At::Checked => settings.dac_settings.enabled.as_at_value(),
                    },
                ],
                label![
                    "Enable DAC chip control?",
                    attrs! {
                        At::For => "dac_cb"
                    }
                ]
            ],
            IF!(settings.dac_settings.enabled => view_dac(&settings.dac_settings))
        ],
        div![
            C!["field", "is-grouped"],
            div![
                C!("control"),
                button![
                    C!["button", "is-dark"],
                    "Save",
                    ev(Ev::Click, |_| Msg::SaveSettings)
                ]
            ],
            div![
                C!("control"),
                button![
                    C!["button", "is-dark"],
                    "Back",
                    ev(Ev::Click, |_| Urls::player_abs().go_and_load())
                ]
            ]
        ]
    ]
}

fn view_dac(dac_settings: &DacSettings) -> Node<Msg> {
    div![
        div![
            C!["field"],
            label!["DAC Chip:", C!["label"]],
            div![
                C!["select"],
                select![
                    option![attrs! {At::Value => "ak4497"}, "AK4497"],
                    option![attrs! {At::Value => "ak4490"}, "AK4490"],
                    option![attrs! {At::Value => "ak4495"}, "AK4495"],
                ],
            ],
        ],
        div![
            C!["field"],
            label!["DAC I2C address:", C!["label"]],
            div![
                C!["control"],
                input![C!["input"], attrs! {At::Value => dac_settings.i2c_address},],
            ],
        ],
        div![
            C!["field"],
            label!["DAC digital filter:", C!["label"]],
            div![
                C!["control"],
                div![
                    C!["select"],
                    select![
                        FilterType::iter().map(|fs| option![format!("{:?}", fs)]),
                        input_ev(Ev::Change, move |selected| Msg::SendCommand(
                            Command::Filter(FilterType::from_str(selected.as_str()).unwrap())
                        )),
                    ],
                ],
            ],
        ]
    ]
}
fn view_spotify(spot_settings: &SpotifySettings) -> Node<Msg> {
    div![
        style! {
            St::PaddingBottom => "1.2rem"
        },
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Spotify connect device name", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => spot_settings.device_name},],
                ]
            ],
        ],
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Spotify username", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => spot_settings.username},],
                ]
            ],
        ],
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Spotify password", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => spot_settings.password},],
                ]
            ],
        ],
    ]
}
fn view_lms(lms_settings: &LmsSettings) -> Node<Msg> {
    div![
        style! {
            St::PaddingBottom => "1.2rem"
        },
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Logitech media server host", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => lms_settings.server_host},],
                ]
            ],
        ],
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Player port", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => lms_settings.server_port},],
                ]
            ],
        ],
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["CLI port", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![C!["input"], attrs! {At::Value => lms_settings.cli_port},],
                ]
            ],
        ],
    ]
}
fn view_mpd(mpd_settings: &MpdSettings) -> Node<Msg> {
    div![
        style! {
            St::PaddingBottom => "1.2rem"
        },
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Music Player Daemon server host", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![
                        C!["input"],
                        attrs! {At::Value => mpd_settings.server_host},
                        input_ev(Ev::Input, move |value| { Msg::InputMpdHostChange(value) }),
                    ],
                ]
            ],
        ],
        div![
            C!["field", "is-horizontal"],
            div![
                C!["field-label", "is-small"],
                label!["Client port", C!["label"]],
            ],
            div![
                C!["field-body"],
                div![
                    C!["field"],
                    input![
                        C!["input"],
                        attrs! {At::Value => mpd_settings.server_port},
                        input_ev(Ev::Input, move |v| {
                            Msg::InputMpdPortChange(v.parse::<u32>().unwrap_or_default())
                        }),
                    ],
                ]
            ],
        ],
    ]
}
