use indexmap::IndexMap;
use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------
pub(crate) fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    log!("Settings Init called");
    orders.perform_cmd(async {
        let response = fetch("/api/settings")
            .await
            .expect("Failed to get settings from dplayer backend");

        let sett = response
            .json::<Settings>()
            .await
            .expect("failed to deserialize to Configuration");
        log!("Remote settings {}", sett);
        Msg::RemoteConfiguration(sett)
    });
    Model {
        settings: Settings::default(),
        dac_enabled: false,
        spotify_enabled: false,
        lms_enabled: false,
        mpd_enabled: false,
    }
}

// ------ ------
//     Model
#[derive(Debug)]
pub struct Model {
    settings: Settings,
    dac_enabled: bool,
    spotify_enabled: bool,
    lms_enabled: bool,
    mpd_enabled: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub spotify_settings: Option<SpotifySettings>,
    pub lms_settings: Option<LmsSettings>,
    pub mpd_settings: Option<MpdSettings>,
    pub dac_settings: Option<DacSettings>,
    pub alsa_settings: AlsaSettings,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SpotifySettings {
    pub enabled: bool,
    pub device_name: String,
    pub username: String,
    pub password: String,
    pub bitrate: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LmsSettings {
    pub enabled: bool,
    pub cli_port: u32,
    pub server_host: String,
    pub server_port: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MpdSettings {
    pub enabled: bool,
    pub server_host: String,
    pub server_port: u32,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AlsaSettings {
    pub device_name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DacSettings {
    pub chip_id: String,
    pub i2c_address: u16,
    pub volume_step: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            alsa_settings: AlsaSettings {
                device_name: String::from(""),
            },
            dac_settings: None,
            lms_settings: None,
            mpd_settings: None,
            spotify_settings: None,
        }
    }
}

pub(crate) enum Msg {
    // ---- on off toggles ----
    ToggleDacEnabled,
    ToggleSpotifyEnabled,
    ToggleLmsEnabled,
    ToggleMpdEnabled,

    // ---- Input capture ----
    InputMpdHostChange(String),
    InputLMSHostChange,
    InputSpotifyDeviceNameChange,
    InputSpotifyUsernameChange,
    InputSpotifyPasswordChange,

    // --- Buttons ----
    SaveConfiguration,

    RemoteConfiguration(Settings),
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SaveConfiguration => {
            log!("New data: {} {}", model.settings);
            //model.config.mpd_host = name.clone();
        }
        Msg::ToggleDacEnabled => {
            model.dac_enabled = !model.dac_enabled;
        }
        Msg::ToggleSpotifyEnabled => {
            model.spotify_enabled = !model.spotify_enabled;
        }
        Msg::ToggleLmsEnabled => {
            model.lms_enabled = !model.lms_enabled;
        }
        Msg::InputMpdHostChange(value) => {}
        Msg::InputLMSHostChange => {}
        Msg::InputSpotifyDeviceNameChange => {}
        Msg::InputSpotifyUsernameChange => {}
        Msg::InputSpotifyPasswordChange => {}
        Msg::ToggleMpdEnabled => {
            model.mpd_enabled = !model.mpd_enabled;
        }
        Msg::RemoteConfiguration(sett) => {
            model.spotify_enabled = sett.spotify_settings.is_some();
            model.mpd_enabled = sett.mpd_settings.is_some();
            model.lms_enabled = sett.lms_settings.is_some();
            model.dac_enabled = sett.dac_settings.is_some();
            model.settings = sett;
        }
    }
}

// ------ ------
//     View
// ------ ------

pub(crate) fn view(model: &Model) -> Node<Msg> {
    view_settings(model)
}

// ------ configuration ------

fn view_settings(model: &Model) -> Node<Msg> {
    div![
        section![
            C!["section"],
            h1![C!["title"], "Players"],
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleMpdEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "mpd_cb"
                        At::Type => "checkbox"
                        At::Checked => model.mpd_enabled.as_at_value(),
                    },
                ],
                label![
                    "Enable Music Player Demon integration?",
                    attrs! {
                        At::For => "mpd_cb"
                    }
                ]
            ],
            IF!(model.mpd_enabled => view_mpd(model.settings.mpd_settings.as_ref())),
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleLmsEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "lms_cb"
                        At::Type => "checkbox"
                        At::Checked => model.lms_enabled.as_at_value(),
                    },
                ],
                label![
                    "Enable Logitech Media Server integration?",
                    attrs! {
                        At::For => "lms_cb"
                    }
                ]
            ],
            IF!(model.lms_enabled => view_lms(model.settings.lms_settings.as_ref())),
            div![
                C!["field"],
                ev(Ev::Click, |_| Msg::ToggleSpotifyEnabled),
                input![
                    C!["switch"],
                    attrs! {
                        At::Name => "spotify_cb"
                        At::Type => "checkbox"
                        At::Checked => model.spotify_enabled.as_at_value(),
                    },
                ],
                label![
                    "Enable spotify integration?",
                    attrs! {
                        At::For => "spotify_cb"
                    }
                ]
            ],
            IF!(model.spotify_enabled => view_spotify(model.settings.spotify_settings.as_ref()))
        ],
        section![
            C!["section"],
            h1![C!["title"], "Alsa"],
            div![
                C!["field"],
                label!["Alsa audio device:", C!["label"]],
                div![
                    C!["select"],
                    select![
                        option![attrs! {At::Value => "waveIO"}, "WaveIO USB"],
                        option![attrs! {At::Value => "amanero"}, "Amanero USB"],
                    ],
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
                        At::Checked => model.dac_enabled.as_at_value(),
                    },
                ],
                label![
                    "Enable DAC chip control?",
                    attrs! {
                        At::For => "dac_cb"
                    }
                ]
            ],
            IF!(model.dac_enabled => view_dac(model.settings.dac_settings.as_ref()))
        ],
        div![
            C!["field", "is-grouped"],
            div![
                C!("control"),
                button![
                    C!["button", "is-dark"],
                    "Save",
                    ev(Ev::Click, |_| Msg::SaveConfiguration)
                ]
            ]
        ]
    ]
}

fn view_dac(dac_settings: Option<&DacSettings>) -> Node<Msg> {
    if let Some(dac_settings) = dac_settings {
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
            ]
        ]
    } else {
        div![]
    }
}
fn view_spotify(spot_settings: Option<&SpotifySettings>) -> Node<Msg> {
    if let Some(spot_settings) = spot_settings {
        div![
            div![
                C!["field"],
                label!["Spotify connect device name:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => spot_settings.device_name},],
                ],
            ],
            div![
                C!["field"],
                label!["Spotify username:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => spot_settings.username},],
                ],
            ],
            div![
                C!["field"],
                label!["Spotify password:", C!["label"]],
                div![
                    C!["control"],
                    input![
                        C!["input"],
                        attrs! {
                            At::Value => spot_settings.password,
                            At::Type => "password",
                        }
                    ],
                ],
            ]
        ]
    } else {
        div![]
    }
}
fn view_lms(lms_settings: Option<&LmsSettings>) -> Node<Msg> {
    if let Some(lms_settings) = lms_settings {
        div![
            div![
                C!["field"],
                label!["Logitech media server host:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => lms_settings.server_host},],
                ],
            ],
            div![
                C!["field"],
                label!["Player port:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => lms_settings.server_port},],
                ],
            ],
            div![
                C!["field"],
                label!["Cli port:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => lms_settings.cli_port},],
                ],
            ]
        ]
    } else {
        div![]
    }
}
fn view_mpd(maybe_settings: Option<&MpdSettings>) -> Node<Msg> {
    if let Some(mpd_settings) = maybe_settings {
        div![
            div![
                C!["field"],
                label!["Music Player Daemon server host:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => mpd_settings.server_host},],
                ],
            ],
            div![
                C!["field"],
                label!["Client port:", C!["label"]],
                div![
                    C!["control"],
                    input![C!["input"], attrs! {At::Value => mpd_settings.server_port},],
                ],
            ],
        ]
    } else {
        div![]
    }
}
