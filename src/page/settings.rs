use std::collections::HashMap;

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
    }
}

// ------ ------
//     Model
#[derive(Debug)]
pub struct Model {
    settings: Settings,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub spotify_settings: SpotifySettings,
    pub lms_settings: LmsSettings,
    pub mpd_settings: MpdSettings,
    pub dac_settings: DacSettings,
    pub alsa_settings: AlsaSettings,
    pub ir_control_settings: IRInputControlerSettings,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct SpotifySettings {
    pub enabled: bool,
    pub device_name: String,
    pub username: String,
    pub password: String,
    pub bitrate: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct LmsSettings {
    pub enabled: bool,
    pub cli_port: u32,
    pub server_host: String,
    pub server_port: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct MpdSettings {
    pub enabled: bool,
    pub server_host: String,
    pub server_port: u32,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct AlsaSettings {
    pub device_name: String,
    #[serde(skip_deserializing)]
    pub available_alsa_pcm_devices: HashMap<String, String>,
    #[serde(skip_deserializing)]
    pub available_alsa_control_devices: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct DacSettings {
    pub enabled: bool,
    pub chip_id: String,
    pub i2c_address: u16,
    pub volume_step: u8,
    #[serde(skip_deserializing)]
    pub available_dac_chips: HashMap<String, String>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct IRInputControlerSettings {
    pub enabled: bool,
    pub input_socket_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            alsa_settings: AlsaSettings::default(),
            dac_settings: DacSettings::default(),
            lms_settings: LmsSettings::default(),
            mpd_settings: MpdSettings::default(),
            spotify_settings: SpotifySettings::default(),
            ir_control_settings: IRInputControlerSettings::default(),
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
        Msg::InputMpdHostChange(value) => {}
        Msg::InputLMSHostChange => {}
        Msg::InputSpotifyDeviceNameChange => {}
        Msg::InputSpotifyUsernameChange => {}
        Msg::InputSpotifyPasswordChange => {}
        Msg::RemoteConfiguration(sett) => {
            model.settings = sett;
        }
    }
}

// ------ ------
//     View
// ------ ------

pub(crate) fn view(model: &Model) -> Node<Msg> {
    view_settings(&model.settings)
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
                    ev(Ev::Click, |_| Msg::SaveConfiguration)
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
                    input![C!["input"], attrs! {At::Value => mpd_settings.server_host},],
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
                    input![C!["input"], attrs! {At::Value => mpd_settings.server_port},],
                ]
            ],
        ],
    ]
}
