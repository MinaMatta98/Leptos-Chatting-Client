use std::collections::HashMap;
use std::sync::Arc;

use base64::engine::general_purpose;
use base64::Engine;
use lazy_static::lazy_static;
use leptos::*;
use leptos_icons::*;

use crate::app::pages::loading_fallback;
use crate::app::{IconVec, SideBarContext};
use crate::server_function::get_icon;

lazy_static! {
    #[derive(Debug)]
    pub static ref ICONVEC: Arc<parking_lot::RwLock<Vec<UserIcon>>> = Arc::new(parking_lot::RwLock::new(Vec::new()));
}

lazy_static! {
    #[derive(Debug)]
    pub static ref IMAGEVEC: Arc<parking_lot::RwLock<Vec<UserImage>>> = Arc::new(parking_lot::RwLock::new(Vec::new()));
}

pub fn base_64_encode_uri(base64_encoded_image: String) -> String {
    format!("data:image/png;base64, {}", base64_encoded_image)
}

// impl IconVec {
//     pub fn fetch_image(id: i32, cx: Scope) -> ImageFetcher {
//         let icons = use_context::<IconVec>(cx).unwrap();
//         if let Some(image) = icons.icons.get().iter().find(|&item| *item.0 == id) {
//             ImageFetcher::Cached(image.1.clone())
//         } else {
//             ImageFetcher::Fetched(create_local_resource(
//                 cx,
//                 move || use_context::<SideBarContext>(cx).unwrap().status.get(),
//                 move |_| async move { get_icon(cx, id).await },
//             ))
//         }
//     }

//     pub fn init_and_return_signal(id: i32, cx: Scope) -> RwSignal<IconType> {
//         let fetch_icon = move |icons: RwSignal<HashMap<i32, UserIcon>>| {
//             icons
//                 .get()
//                 .iter()
//                 .find(|icons| *icons.0 == id)
//                 .unwrap()
//                 .1
//                 .image
//         };
//         let icons = move || use_context::<IconVec>(cx).unwrap().icons;
//         if !icons()
//             .get()
//             .iter()
//             .any(|user_icons| *user_icons.0 == id && user_icons.1.image.try_get().is_some())
//         {
//             log!("couldn't find icon{id}");
//             icons().update(|item| {
//                 item.insert(
//                     id,
//                     UserIcon {
//                         user_id: id,
//                         image: create_rw_signal(cx, IconType::String("".to_string())),
//                     },
//                 );
//             });
//             fetch_icon(icons())
//         } else {
//             log!("found icon {id}");
//             fetch_icon(icons())
//         }
//     }

//     pub fn render_image(
//         resolved_image: ImageFetcher,
//         cx: Scope,
//         id: i32,
//         is_group: bool,
//         sidebar: bool,
//     ) -> impl IntoView {
//         let icon_class = match is_group {
//             false => {
//                 format!(
//                     "h-10 w-10 md:w-12 md:h-12 text-gray-400 {}",
//                     if sidebar { "" } else { "mx-auto" }
//                 )
//             }
//             true => String::from("text-gray-400 h-[21px] w-[21px]"),
//         };

//         let image_view = move |image_signal: RwSignal<IconType>, cx: Scope| -> Fragment {
//             let image = move || std::convert::Into::<String>::into(image_signal.get());

//             view! { cx, <> <img src=image alt="Image" class=move || if sidebar {"w-12 h-12 rounded-full"} else {""} /> </> }
//         };

//         let icon_view = move |image_signal: RwSignal<IconType>, cx: Scope| -> Fragment {
//             let image = move || std::convert::Into::<Icon>::into(image_signal.get());

//             view! {cx, <> <Icon icon=image() class=icon_class.clone()/> </> }
//         };

//         match resolved_image {
//             ImageFetcher::Fetched(image) => image
//                 .read(cx)
//                 .map(|image| {
//                     log!("triggered fetch");
//                     let image = image.unwrap_or_default();
//                     let view: Fragment;
//                     {
//                         IconVec::init_and_return_signal(id, cx);
//                     }
//                     let icons = use_context::<IconVec>(cx).unwrap().icons;
//                     log!("{:?}", icons);
//                     let image_signal = icons
//                         .get()
//                         .iter()
//                         .find(|icons| *icons.0 == id)
//                         .unwrap()
//                         .1
//                         .image;

//                     if let Some(image) = image {
//                         let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
//                         image_signal
//                             .set(IconType::String(base_64_encode_uri(base64_encoded_image)));
//                         view = image_view(image_signal, cx)
//                     } else {
//                         image_signal.set(IconType::Icon(Icon::Bi(BiIcon::BiUserCircleSolid)));
//                         view = icon_view(image_signal, cx)
//                     };

//                     view
//                 })
//                 .into_view(cx),
//             ImageFetcher::Cached(image) => {
//                 log!("triggered cached");
//                 view! {cx,
//                      <>
//                         {move ||
//                             match image.image.get() {
//                                 IconType::String(_) => image_view(image.image, cx).into_view(cx),
//                                 IconType::Icon(_) => icon_view(image.image, cx).into_view(cx),
//                             }
//                         }
//                     </>
//                 }
//                 .into_view(cx)
//             }
//         }
//     }
// }

impl ICONVEC {
    pub fn fetch_image(id: i32, cx: Scope) -> ImageFetcher {
        if let Some(image) = ICONVEC.read().iter().find(|&item| item.user_id == id) {
            ImageFetcher::Cached(image.clone())
        } else {
            ImageFetcher::Fetched(create_local_resource(
                cx,
                move || use_context::<SideBarContext>(cx).unwrap().status.get(),
                move |_| async move { get_icon(cx, id).await },
            ))
        }
    }

    // pub fn _insert_or_update(id: i32, image: Vec<u8>) -> LockStatus {
    //     if let Some(mut guard) = ICONVEC.try_write() {
    //         let index = guard.iter().position(|item| item.user_id == id);

    //         if let Some(index) = index {
    //             let rw_signal = guard.get_mut(index).unwrap().image;
    //             let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
    //             rw_signal.set(IconType::String(base_64_encode_uri(base64_encoded_image)))
    //         }
    //         LockStatus::Aquired
    //     } else {
    //         LockStatus::UnAquired
    //     }
    // }

    pub fn init_and_return_signal(id: i32, cx: Scope) -> RwSignal<IconType> {
        let mut lock = ICONVEC.write();
        if !lock.iter().any(|user_icons| user_icons.user_id == id && user_icons.image.try_get().is_some()) {
            lock.push(UserIcon {
                user_id: id,
                image: create_rw_signal(cx, IconType::String("".to_string())),
            });
            lock.iter().last().unwrap().image
        } else {
            lock.iter().find(|&icon| icon.user_id == id).unwrap().image
        }
    }

    pub fn render_image(
        resolved_image: ImageFetcher,
        cx: Scope,
        id: i32,
        is_group: bool,
        sidebar: bool,
    ) -> impl IntoView {
        let icon_class = match is_group {
            false => {
                format!(
                    "h-10 w-10 md:w-12 md:h-12 text-gray-400 {}",
                    if sidebar { "" } else { "mx-auto" }
                )
            }
            true => String::from("text-gray-400 h-[21px] w-[21px]"),
        };

        let image_view = move |image_signal: RwSignal<IconType>, cx: Scope| -> Fragment {
            let image = move || std::convert::Into::<String>::into(image_signal.get());

            view! { cx, <> <img src=image alt="Image" class=move || if sidebar {"w-12 h-12 rounded-full"} else {""} /> </> }
        };

        let icon_view = move |image_signal: RwSignal<IconType>, cx: Scope| -> Fragment {
            let image = move || std::convert::Into::<Icon>::into(image_signal.get());

            view! {cx, <> <Icon icon=image() class=icon_class.clone()/> </> }
        };

        match resolved_image {
            ImageFetcher::Fetched(image) => image
                .read(cx)
                .map(|image| {
                    let image = image.unwrap_or_default();
                    let view: Fragment;
                    let image_signal = ICONVEC::init_and_return_signal(id, cx);

                    if let Some(image) = image {
                        let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
                        image_signal
                            .set(IconType::String(base_64_encode_uri(base64_encoded_image)));
                        view = image_view(image_signal, cx)
                    } else {
                        image_signal.set(IconType::Icon(Icon::Bi(BiIcon::BiUserCircleSolid)));
                        view = icon_view(image_signal, cx)
                    };

                    view
                })
                .into_view(cx),
            ImageFetcher::Cached(image) => view! {cx,
                 <>
                    {move ||
                        match image.image.get() {
                            IconType::String(_) => image_view(image.image, cx).into_view(cx),
                            IconType::Icon(_) => icon_view(image.image, cx).into_view(cx),
                        }
                    }
                </>
            }
            .into_view(cx),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserIcon {
    pub user_id: i32,
    pub image: RwSignal<IconType>,
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

// #[derive(Clone, Debug, PartialEq)]
// pub enum LockStatus {
//     Aquired,
//     UnAquired,
// }

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

#[derive(Clone)]
pub enum ImageFetcher {
    Fetched(leptos::Resource<bool, std::result::Result<Option<Vec<u8>>, ServerFnError>>),
    Cached(UserIcon),
}

#[component]
pub fn Avatar(cx: Scope, id: i32) -> impl IntoView {
    let fetched_image = ICONVEC::fetch_image(id, cx);
    log!("triggered avatar");
    view! {cx,
        <div class="relative inline-block
            rounded-full
            h-9 w-9 md:h-11 md:w-11 text-slate-800">
                <Suspense fallback=loading_fallback(cx)>
                    {
                        let fetched_image = fetched_image.clone();
                            move ||
                            ICONVEC::render_image(fetched_image.clone(), cx, id, false, false)
                    }
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
                        let image = ICONVEC::fetch_image(user.1, cx);
                          view! {
                            cx,
                              <div class=move || format!("absolute flex rounded-full overflow-y-auto h-[21px] w-[21px] justify-center {}",
                                  position_map.get(&(user.0 as i8)).unwrap())>
                                     <Suspense fallback=loading_fallback(cx)>
                                        {
                                            let image = image.clone();
                                            move || ICONVEC::render_image(image.clone(), cx, user.1, true, false)
                                        }
                                     </Suspense>
                            </div>
             }}/>
        </div>
    }
}
