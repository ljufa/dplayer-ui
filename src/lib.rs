use indexmap::IndexMap;
use seed::{prelude::*, *};
mod page;

const SETTINGS: &str = "settings";
const FIRST_SETUP: &str = "setup";

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
        let slice = url.remaining_hash_path_parts();
        match slice.as_slice() {
            [FIRST_SETUP] => Self::Home,
            [SETTINGS] => Self::Settings(page::settings::init(
                url,
                &mut orders.proxy(Msg::SettingsMsg),
            )),
            [] => Self::Player(page::player::init(url, &mut orders.proxy(Msg::PlayerMsg))),
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
    fn player(self) -> Url {
        self.base_url()
    }

    fn player_abs() -> Url {
        Url::new()
    }
}

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
    nodes![view_content(&model.page, &model.base_url)]
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

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
