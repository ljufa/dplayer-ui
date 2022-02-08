use indexmap::IndexMap;
use seed::{prelude::*, *};
mod page;

const SETTINGS: &str = "settings";
const PLAYER: &str = "player";
const FIRST_SETUP: &str = "setup";

// ------ ------
//     Init
// ------ ------

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
        menu_visible: false,
        base_url: url.to_hash_base_url(),
        page: Page::init(url, orders),
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
struct Model {
    menu_visible: bool,
    base_url: Url,
    page: Page,
}

enum Msg {
    UrlChanged(subs::UrlChanged),
    ToggleMenu,
    SettingsMsg(page::settings::Msg),
    PlayerMsg(page::player::Msg),
}

// ------ Page ------
#[derive(Debug)]
enum Page {
    Home,
    Settings(page::settings::Model),
    Player(page::player::Model),
    NotFound,
}
impl Page {
    fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Self {
        match url.remaining_hash_path_parts().as_slice() {
            [FIRST_SETUP] => Self::Home,
            [SETTINGS] => Self::Settings(page::settings::init(
                url,
                &mut orders.proxy(Msg::SettingsMsg),
            )),
            [PLAYER] => Self::Player(page::player::init(url, &mut orders.proxy(Msg::PlayerMsg))),

            _ => Self::NotFound,
        }
    }
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    fn home(self) -> Url {
        self.base_url()
    }
    fn settings(self) -> Url {
        self.base_url().add_hash_path_part(SETTINGS)
    }
    fn player(self) -> Url {
        self.base_url().add_hash_path_part(PLAYER)
    }
}

// ------ ------
//    Update
// ------ ------

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => model.page = Page::init(url, orders),
        Msg::ToggleMenu => model.menu_visible = not(model.menu_visible),
        Msg::SettingsMsg(msg) => {
            if let Page::Settings(model) = &mut model.page {
                page::settings::update(msg, model, &mut orders.proxy(Msg::SettingsMsg))
            }
        }
        Msg::PlayerMsg(msg) => {
            if let Page::Player(model) = &mut model.page {
                page::player::update(msg, model, &mut orders.proxy(Msg::PlayerMsg))
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> impl IntoNodes<Msg> {
    nodes![
        view_navbar(model.menu_visible, &model.base_url, &model.page),
        view_content(&model.page, &model.base_url)
    ]
}

// ----- view_content ------

fn view_content(page: &Page, base_url: &Url) -> Node<Msg> {
    div![
        C!["container"],
        match page {
            Page::Home => page::home::view(base_url),
            Page::NotFound => page::not_found::view(),
            Page::Settings(model) => page::settings::view(model).map_msg(Msg::SettingsMsg),
            Page::Player(model) => page::player::view(model).map_msg(Msg::PlayerMsg),
        }
    ]
}

// ----- view_navbar ------

fn view_navbar(menu_visible: bool, base_url: &Url, page: &Page) -> Node<Msg> {
    nav![
        C!["navbar", "is-dark"],
        attrs! {
            At::from("role") => "navigation",
            At::AriaLabel => "main navigation",
        },
        view_brand_and_hamburger(menu_visible, base_url, page),
        view_navbar_menu(menu_visible, base_url, page),
    ]
}

fn view_brand_and_hamburger(menu_visible: bool, base_url: &Url, page: &Page) -> Node<Msg> {
    div![
        style! {
            St::Padding => "1.2rem",
        },
        C!["navbar-brand"],
        // ------ Logo ------
        a![
            C!("navbar-item"),
            match page {
                Page::Player(model) =>
                    page::player::view_player_switch(model).map_msg(Msg::PlayerMsg),
                _ => empty!(),
            }
        ],
        // ------ Hamburger ------
        a![
            C!["navbar-burger", "burger", IF!(menu_visible => "is-active")],
            style! {
                St::MarginTop => "auto",
                St::MarginBottom => "auto",
            },
            attrs! {
                At::from("role") => "button",
                At::AriaLabel => "menu",
                At::AriaExpanded => menu_visible,
            },
            ev(Ev::Click, |event| {
                event.stop_propagation();
                Msg::ToggleMenu
            }),
            span![attrs! {At::AriaHidden => "true"}],
            span![attrs! {At::AriaHidden => "true"}],
            span![attrs! {At::AriaHidden => "true"}],
        ]
    ]
}

fn view_navbar_menu(menu_visible: bool, base_url: &Url, page: &Page) -> Node<Msg> {
    div![
        C!["navbar-menu", IF!(menu_visible => "is-active")],
        div![
            C!["navbar-start"],
            a![
                C![
                    "navbar-item",
                    IF!(matches!(page, Page::Settings(_)) => "is-active"),
                ],
                attrs! {At::Href => Urls::new(base_url).settings()},
                "Settings",
            ],
            a![
                C![
                    "navbar-item",
                    IF!(matches!(page, Page::Player(_)) => "is-active"),
                ],
                attrs! {At::Href => Urls::new(base_url).player()},
                "Player",
            ],
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
