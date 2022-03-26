use api_models::player::{Command, Song };
use seed::{prelude::*, *};

#[derive(Debug)]
pub struct Model {
    pub queue_items: Vec<Song>,
    waiting_response: bool,
}

pub enum Msg {
    PlaylistItemsFetched(fetch::Result<Vec<Song>>),
    SendCommand(Command),
}

pub(crate) fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::PlaylistItemsFetched(get_queue_items().await) });
    Model {
        queue_items: Vec::new(),
        waiting_response: true,
    }
}

// ------ ------
//    Update
// ------ ------

pub(crate) fn update(msg: Msg, mut model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::PlaylistItemsFetched(pl_items) => {
            model.waiting_response = false;
            model.queue_items = pl_items.unwrap_or_default();
        }
        _ => {}
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        crate::view_spinner_modal(model.waiting_response),
        view_queue_items(model)
    ]
}

fn view_queue_items(model: &Model) -> Node<Msg> {
    div![
        div![
            C![
                "list",
                "has-overflow-ellipsis has-visible-pointer-controls has-hoverable-list-items"
            ],
            model.queue_items.iter().map(|it| {
                let cp = it.position.unwrap_or(0);
                div![
                    C!["list-item"],
                    div![
                        C!["list-item-content"],
                        div![C!["list-item-title"], &it.get_title()],
                        div![C!["description"], &it.album],
                    ],
                    div![
                        C!["list-item-controls"],
                        div![
                            C!["buttons"],
                            button![
                                C!["button is-light is-small"],
                                span![C!["icon"], i![C!["fas", "fa-play"]]],
                                ev(Ev::Click, move |_| Msg::SendCommand(Command::PlayAt(cp))),
                            ],
                        ]
                    ]
                ]
            })
        ]
    ]
}

pub async fn get_queue_items() -> fetch::Result<Vec<Song>> {
    Request::new("/api/queue".to_string())
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .json::<Vec<Song>>()
        .await
}
