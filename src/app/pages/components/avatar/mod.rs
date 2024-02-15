use crate::app::pages::{
    components::anciliary::loading_fallback, HandleWebSocket, StreamData, SyncChannel, WsData,
};
use crate::server_function::routes::get_icon;
use base64::{engine::general_purpose, Engine};
use lazy_static::lazy_static;
use leptos::*;
use leptos_icons::*;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::{collections::HashMap, sync::Arc};

lazy_static! {
    #[derive(Debug)]
    pub static ref ICONVEC: Arc<parking_lot::RwLock<Vec<UserIcon>>> = Arc::new(parking_lot::RwLock::new(Vec::new()));
}

lazy_static! {
    #[derive(Debug)]
    pub static ref IMAGEVEC: Arc<parking_lot::RwLock<Vec<UserImage>>> = Arc::new(parking_lot::RwLock::new(Vec::new()));
}

pub type WsVecType = Arc<parking_lot::RwLock<HashMap<(WsData, i32), SyncChannel>>>;

lazy_static! {
    #[derive(Debug)]
    pub static ref STREAMVEC: WsVecType = Arc::new(parking_lot::RwLock::new(HashMap::new()));
}

lazy_static! {
    #[derive(Debug)]
    pub static ref SINKVEC: WsVecType = Arc::new(parking_lot::RwLock::new(HashMap::new()));
}
pub trait ToStreamData {
    fn from_inner(inner: &str) -> Result<StreamData, std::io::Error>;
}

#[derive(Debug)]
pub enum WebSocketState {
    NewConnection,
    PassThrough,
}

fn push_to_map<'a, F>(accessor: F, id: i32, data: WsData, channel: SyncChannel)
where
    F: FnOnce() -> RwLockWriteGuard<'a, HashMap<(WsData, i32), SyncChannel>>,
{
    let mut glob_vec = accessor();
    glob_vec.insert((data, id), channel);
}

fn retrieve_from_map<'a, F>(accessor: F, data: WsData, id: i32) -> Option<SyncChannel>
where
    F: FnOnce() -> RwLockReadGuard<'a, HashMap<(WsData, i32), SyncChannel>>,
{
    let glob_vec = accessor();
    glob_vec.get(&(data, id)).cloned()
}

impl SINKVEC {
    pub fn sync_stream(id: i32, data: WsData) -> SyncChannel {
        match retrieve_from_map(|| SINKVEC.read(), data, id) {
            Some(dual_channel) => match dual_channel {
                SyncChannel::BroadCast(tx, rx) => SyncChannel::BroadCast(tx, rx.new_receiver()),
                SyncChannel::Mpsc(tx, rx) => SyncChannel::Mpsc(tx, rx),
            },
            None => {
                let channel = match data {
                    WsData::IconData => {
                        let (tx, rx) = async_broadcast::broadcast::<StreamData>(100000);
                        SyncChannel::BroadCast(tx, rx)
                    }
                    WsData::MessageData => {
                        let (tx, rx) = async_broadcast::broadcast::<StreamData>(100000);
                        SyncChannel::Mpsc(tx, rx)
                    }
                };
                push_to_map(|| SINKVEC.write(), id, data, channel.clone());
                channel
            }
        }
    }

    pub fn send_clear() {
        spawn_local(async move {
            let mut lock = SINKVEC.write();
            for (_, sync_channel) in lock.iter_mut() {
                sync_channel.send(StreamData::Close).await
            }
            lock.clear();
        })
    }
}

impl STREAMVEC {
    pub fn sync_stream(id: i32, data: WsData) -> (SyncChannel, WebSocketState) {
        match retrieve_from_map(|| STREAMVEC.read(), data, id) {
            Some(dual_channel) => {
                let channel = match dual_channel {
                    SyncChannel::BroadCast(tx, rx) => SyncChannel::BroadCast(tx, rx.new_receiver()),
                    SyncChannel::Mpsc(..) => dual_channel,
                };
                (channel, WebSocketState::PassThrough)
            }
            None => {
                let channel = match data {
                    WsData::IconData => {
                        let (tx, rx) = async_broadcast::broadcast::<StreamData>(100000);
                        SyncChannel::BroadCast(tx, rx)
                    }
                    WsData::MessageData => {
                        let (tx, rx) = async_broadcast::broadcast::<StreamData>(100000);
                        SyncChannel::Mpsc(tx, rx)
                    }
                };
                push_to_map(|| STREAMVEC.write(), id, data, channel.clone());
                (channel, WebSocketState::NewConnection)
            }
        }
    }
}

pub fn base_64_encode_uri(base64_encoded_image: String) -> String {
    format!("data:image/png;base64, {}", base64_encoded_image)
}

impl ICONVEC {
    fn icon_class(_cx: Scope, is_group: bool, sidebar: bool) -> String {
        match is_group {
            false => {
                format!(
                    "h-10 w-10 md:w-12 md:h-12 text-gray-400 {}",
                    if sidebar { "" } else { "mx-auto" }
                )
            }
            true => String::from("text-gray-400 h-[21px] w-[21px]"),
        }
    }

    fn image_view(cx: Scope, image: &str, sidebar: bool, image_signal: RwSignal<Fragment>) {
        image_signal.set(view! {cx,
            <>
                <img src=image.to_string() alt="Image"
                    class=move || if sidebar {"w-12 h-12 rounded-full"} else {""}
                />
            </>
        })
    }

    fn icon_view(
        cx: Scope,
        icon: Icon,
        is_group: bool,
        sidebar: bool,
        image_signal: RwSignal<Fragment>,
    ) {
        image_signal.set(view! {cx,
            <>
                <Icon icon=icon
                    class=Self::icon_class(cx, is_group, sidebar)
                />
            </>
        })
    }

    pub fn fetch_image(
        cx: Scope,
        id: i32,
        is_group: bool,
        sidebar: bool,
        message_string: Option<String>,
        image_signal: RwSignal<Fragment>,
    ) {
        if let Some(message_string) = message_string {
            Self::image_view(cx, &message_string, sidebar, image_signal);
            let mut lock = ICONVEC.write();

            if let Some(position) = lock.iter().position(|icon| icon.user_id == id) {
                lock.remove(position);
            };

            lock.push(UserIcon {
                user_id: id,
                image: message_string,
            });
            
        } else if let Some(image) = ICONVEC.read().iter().find(|item| item.user_id == id) {
            Self::image_view(cx, image.image.as_str(), sidebar, image_signal);
        } else {
            let image =
                create_local_resource(cx, || (), move |_| async move { get_icon(cx, id).await });

            view! {
                     cx,
                    <>
                     <Suspense fallback=loading_fallback(cx)>
                         {move || {
                            image.read(cx).map(|image| {
                                 if let Some(image) = image.unwrap() {
                                     let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
                                     ICONVEC.write().push(UserIcon { user_id: id, image: base_64_encode_uri(base64_encoded_image.clone()) });
                                     Self::image_view(cx, base_64_encode_uri(base64_encoded_image).as_str(), sidebar, image_signal);
                                 } else {
                                     Self::icon_view(cx, Icon::Bi(BiIcon::BiUserCircleSolid), is_group, sidebar, image_signal);
                                 }
                            })
                        }}
                    </Suspense>
                    </>
            };
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIcon {
    pub user_id: i32,
    pub image: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IconData {
    pub user_id: i32,
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImage {
    pub path: String,
    pub image: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IconType {
    String(String),
    Icon(Icon),
}

impl From<IconType> for String {
    fn from(val: IconType) -> Self {
        match val {
            IconType::String(e) => e,
            IconType::Icon(_) => "".to_string(),
        }
    }
}

impl From<IconType> for Icon {
    fn from(val: IconType) -> Self {
        match val {
            IconType::String(_) => Icon::Bi(BiIcon::BiUserCircleSolid),
            IconType::Icon(i) => i,
        }
    }
}

#[component]
pub fn Avatar(cx: Scope, id: i32) -> impl IntoView {
    let image_signal = create_rw_signal(cx, view! {cx, <><div/></>});

    spawn_local(async move {
        HandleWebSocket::handle_split_stream::<String, IconData>(
            cx,
            id,
            None,
            "ws://localhost:8000/ws/icons/",
            move |_signal, value: crate::app::pages::avatar::IconData| {
                match value.user_id == id {
                    true => {
                        ICONVEC::fetch_image(cx, id, false, false, Some(value.data), image_signal);
                    }
                    false => (),
                };
            },
        )
        .await;
    });

    let image = create_local_resource(
        cx,
        move || (),
        move |_| async move { ICONVEC::fetch_image(cx, id, false, false, None, image_signal) },
    );

    view! {cx,
        <div class="relative inline-block
            rounded-full
            h-9 w-9 md:h-11 md:w-11 text-slate-800">
                <Suspense fallback=loading_fallback(cx)>
                    {move || image.read(cx)}
                    {move || image_signal}
                    <span class="absolute block rounded-full
                    bg-green-500 ring-2 ring-white top-0
                    right-0 h-2 w-2 md:h-3 md:w-3"/>
                </Suspense>
        </div>
    }
}

#[component]
pub fn AvatarGroup(cx: Scope, user_ids: Vec<i32>) -> impl IntoView {
    let mut user_ids = user_ids;
    user_ids.truncate(3);
    let user_ids = user_ids.into_iter().enumerate().collect::<Vec<_>>();
    view! {cx,
        <div class="relative h-11 w-11">
               <For
                 each=move || user_ids.clone()
                 key=|(position,_)| *position
                 view=move |cx, user: (usize, i32)| {
                    let position_map: HashMap<i8, &str> = HashMap::from([
                        (0, "top-0 left-[12px]"),
                        (1,  "bottom-0"),
                        (2, "bottom-0 right-0")
                    ]);
                    let message_signal: RwSignal<Option<String>> = create_rw_signal(cx, None);
                    let image_signal = create_rw_signal(cx, view!{cx, <><div/></>});
                    spawn_local(async move {
                    HandleWebSocket::handle_split_stream(
                         cx,
                         user.1,
                         Some(message_signal),
                         "ws://localhost:8000/ws/icons/",
                         move |_signal, value: crate::app::pages::avatar::IconData| {
                                match value.user_id == user.1 {
                                    true => {ICONVEC::fetch_image(cx, user.1, true, false, Some(value.data), image_signal);},
                                    false => ()
                            };
                        }
                     )
                     .await
                    });

                    let image = create_local_resource(cx, move || (),
                        move |_|
                        async move {
                        ICONVEC::fetch_image(cx, user.1, true, false, None, image_signal)
                    });
                          view! {
                            cx,
                              <div class=move || format!("absolute flex rounded-full overflow-y-auto h-[21px] w-[21px] justify-center {}",
                                  position_map.get(&(user.0 as i8)).unwrap())>
                                     <Suspense fallback=loading_fallback(cx)>
                                        {move || image.read(cx)}
                                        {move || image_signal}
                                     </Suspense>
                            </div>
             }}/>
        </div>
    }
}
