use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use base64::{engine::general_purpose, Engine as _};
use lazy_static::lazy_static;
use leptos::{
    html::{Div, Input},
    *,
};
use leptos_icons::*;
use leptos_router::*;
use web_sys::{FormData, HtmlFormElement, HtmlLiElement, MouseEvent, SubmitEvent};

use crate::{
    app::IsOpen,
    server_function::{
        self, associated_conversation, conversation_action, delete_conversations, find_image,
        get_conversations, get_image, get_user, get_users, handle_message_input, handle_seen,
        upload_user_info, validate_conversation, view_messages, ConversationAction,
        ConversationMeta, CreateGroupConversation, ImageAvailability, MergedConversation,
        MergedMessages, UserModel,
    },
};

use super::DrawerContext;

lazy_static! {
    #[derive(Debug)]
    static ref IMAGEVEC: Arc<RwLock<Vec<UserImage>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImage {
    user_id: i32,
    image: String,
}

#[derive(Params, PartialEq, Clone, Debug, Eq)]
struct ConversationIdParams {
    id: i32,
}

#[derive(Clone)]
pub enum ImageEnum {
    Some(std::result::Result<server_function::ImageAvailability, leptos::ServerFnError>),
    None,
}

#[derive(Clone)]
pub enum ImageFetcher {
    Fetched(leptos::Resource<(), std::result::Result<Option<Vec<u8>>, ServerFnError>>),
    Cached(UserImage),
}

#[derive(Clone)]
enum ButtonVal {
    Bool(bool),
    RwSignal(RwSignal<bool>),
}

pub struct SidebarIcon<'a> {
    _label: &'a str,
    href: &'a str,
    icon: HiIcon,
    active: Box<dyn Fn(Scope) -> &'a str>,
    on_click: Option<Box<dyn std::ops::Fn(Scope)>>,
}

impl<'a> SidebarIcon<'a> {
    pub fn init(cx: Scope) -> Vec<SidebarIcon<'a>> {
        let chat = SidebarIcon {
            _label: "Chat",
            href: "/conversations",
            icon: HiIcon::HiChatBubbleOvalLeftEllipsisSolidMd,
            active: Box::new(move |_| {
                let path = use_location(cx).pathname.get();
                path.contains("conversations")
                    .then_some("bg-gray-100 text-black")
                    .map_or_else(|| "", |v| v)
            }),
            on_click: None,
        };

        let users = SidebarIcon {
            _label: "Users",
            href: "/user",
            icon: HiIcon::HiUserCircleSolidMd,
            active: Box::new(move |_| {
                (use_location(cx).pathname.get().as_str() == "/user")
                    .then_some("bg-gray-100 text-black")
                    .map_or_else(|| "", |v| v)
            }),
            on_click: None,
        };

        let logout = SidebarIcon {
            _label: "Logout",
            href: "/login",
            icon: HiIcon::HiArrowLeftCircleOutlineLg,
            active: Box::new(move |_| ""),
            on_click: Some(Box::new(|cx| {
                create_resource(
                    cx,
                    || (),
                    async move |_| server_function::logout(cx).await.unwrap(),
                );
                // queue_microtask(move || use_navigate(cx)("/login", Default::default()).unwrap());
            })),
        };

        vec![chat, users, logout]
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ConversationParams {
    pub id: usize,
    pub none: String,
}

#[derive(PartialEq, Clone, Debug)]
pub struct UserContext {
    pub id: RwSignal<i32>,
    pub email: RwSignal<String>,
}

fn format_created_at(created_at: String) -> String {
    let created_at = created_at.trim_end_matches(" UTC").trim();
    let time = chrono::NaiveTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S").unwrap();
    time.format("%-I:%M %p").to_string()
}

fn get_current_id(cx: Scope) -> impl Fn() -> i32 + 'static + Copy {
    move || {
        use_params::<ConversationIdParams>(cx)
            .get()
            .map(|params| params.id)
            .unwrap_or_default()
    }
}

fn loading_fallback(cx: Scope) -> Box<dyn Fn() -> View> {
    Box::new(move || {
        view! {cx,
                <div class="relative flex items-center justify-center rounded-full max-w-[48px] min-w-[21px] min-h-[21px] font-semibold text-sm shadow text-white bg-indigo-400 hover:bg-indigo-500 transition ease-in-out duration-150 cursor-not-allowed">
                    <svg class="animate-spin max-h-[48px] max-w-[48px] min-w-[22px] min-h-[22px] text-white z-50" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="white" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                </div>
        }.into_view(cx)
    })
}

#[component]
pub fn Users(cx: Scope) -> impl IntoView {
    view! {cx,
        <Sidebar>
            <UserList/>
            <div class="lg:block lg:pl-80 h-screen">
                <EmptyState />
            </div>
        </Sidebar>
    }
}

#[component]
pub fn EmptyState(cx: Scope) -> impl IntoView {
    view! {cx,
        <div class="px-4 py-10 sm:px-6
        lg:px-8 h-full flex justify-center items-center bg-gray-100">
            <div class="text-center items-center flex flex-col">
                <h3 class="mt-2 text-2xl font-semibold text-gray-900">
                    "Select a chat or start a new conversation"
                </h3>
            </div>
        </div>
    }
}

#[component]
fn Sidebar(cx: Scope, children: Children) -> impl IntoView {
    view! {cx,
        <div class="h-screen">
          <DesktopSidebar />
          <MobileFooter />
            <main class="lg:pl-20 h-screen">
                {children(cx)}
            </main>
        </div>
    }
}

#[component]
fn DesktopSidebar(cx: Scope) -> impl IntoView {
    create_effect(cx, move |_| {
        spawn_local(async move {
            if server_function::redirect(cx).await.unwrap() {
                queue_microtask(move || {
                    leptos_router::use_navigate(cx)("/login", Default::default()).unwrap()
                });
            }
        })
    });

    let status = create_local_resource(
        cx,
        || (),
        move |_| async move { server_function::login_status(cx).await.unwrap() },
    );

    let settings_modal_setter = create_rw_signal(cx, false);
    let id = move || use_context::<UserContext>(cx).unwrap().id.get();

    view! {cx,
        <SettingsModal settings_modal_setter/>
        <div class="hidden lg:fixed lg:inset-y-0 lg:left-0
            lg:z-40 lg:w-20 xl:px-6 lg:overflow-y-auto lg:bg-white
            lg:border-r-[1px] lg:pb-4 lg:flex lg:flex-col justify-between">
            <nav class="mt-4 flex flex-col justify-between">
                <ul role="list" class="flex flex-col items-center space-y-1">
                    <For each=move || SidebarIcon::init(cx)
                      key=|vec| vec.href.to_string()
                      view=move |cx, item: SidebarIcon| {
                        view! {cx, <DesktopItem item />}
                    }/>
                 </ul>
            </nav>
            <Suspense fallback=||()>
            {move || status.read(cx).map(|user| {
                let user_context = use_context::<UserContext>(cx).unwrap();
                user_context.email.set(user.clone().email);
                user_context.id.set(user.id);
                view!{cx,
                 <nav class="mt-4 flex flex-col
                     justify-center items-center">
                         <div class="cursor-pointer
                             hover:opacity-75 transition" on:click=move |_| settings_modal_setter.set(true)>
                                 <Avatar id=id()/>
                         </div>
                 </nav>
                }})}
            </Suspense>
        </div>
    }
}

#[component]
fn MobileFooter(cx: Scope) -> impl IntoView {
    let params = use_params::<ConversationParams>(cx);

    view! {cx,
    {   move ||
            if params().map(|params| params.id).is_err() {
                view! {cx,
                    <div class="fixed justify-between
                    w-screen bottom-0 z-40 flex items-center
                    bg-white border-t-[1px] lg:hidden">
                         <For each=move || SidebarIcon::init(cx)
                           key=|vec| vec.href.to_string()
                           view=move |cx, item: SidebarIcon| {
                             view! {cx, <MobileItem item />}
                         }/>
                             </div>
                         }
            }
            else {
                view! {cx, <div></div>}
            }
    }
    }
}

#[component]
fn MobileItem(cx: Scope, item: SidebarIcon<'static>) -> impl IntoView {
    // let current_user = create_resource(cx, || (), fetcher)
    view! {cx,
        <A href=item.href class=move ||
            format!("group flex gap-x-3 text-sm
            leading-6 font-semibold w-full justify-center
            p-4 hover:bg-gray-100 {}", (item.active)(cx))
            on:click=move |_| if let Some(function) =
            &item.on_click {function(cx)}>
            <Icon style="gray" icon=item.icon
                class="h-6 w-6"/>
        </A>
    }
}

#[component]
fn DesktopItem(cx: Scope, item: SidebarIcon<'static>) -> impl IntoView {
    view! { cx,
    <A on:click=move |_| if let Some(function)
             = &item.on_click {function(cx)} href=item.href
             class=move || format!("group flex gap-x-3
                rounded-md p-4 text-sm leading-6 font-semibold
                text-gray-500 hover:text-black hover:bg-gray-100
                 {}", (item.active)(cx))>
    <Icon icon=item.icon class="h-6 w-6 shrink-0"
             style="color: red"/>
    </A>
    }
}

#[component]
fn Avatar(cx: Scope, id: i32) -> impl IntoView {
    let fetched_image = if let Some(image) = IMAGEVEC
        .read()
        .unwrap()
        .iter()
        .find(|&item| item.user_id == id)
    {
        ImageFetcher::Cached(image.clone())
    } else {
        ImageFetcher::Fetched(create_local_resource(
            cx,
            || (),
            move |_| async move { get_image(cx, id).await },
        ))
    };

    view! {cx,
        <div class="relative inline-block
            rounded-full
            h-9 w-9 md:h-11 md:w-11 text-slate-800">
            <Suspense fallback=loading_fallback(cx)>
            {let fetched_image = fetched_image.clone();
                    move ||
                    match fetched_image.clone() {
                        ImageFetcher::Fetched (image) => {
                             image.read(cx).map(|image| {
                                let image = image.unwrap_or_default();
                                if let Some(image) = image {
                                 let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
                                 let data_uri = format!("data:image/png;base64, {}", base64_encoded_image);
                                 IMAGEVEC.write().unwrap().push(UserImage {
                                         user_id: id,
                                         image: base64_encoded_image
                                 });
                                         view!{cx,
                                           <>
                                               <img src=data_uri alt="Image" />
                                           </>
                                         }
                                     } else {
                                         view!{cx,
                                             <>
                                             <Icon icon=Icon::Bi(leptos_icons::BiIcon::BiUserCircleSolid)
                                             class="h-10 w-10 md:w-12 md:h-12 mx-auto text-gray-400"/>
                                             </>
                                         }
                                }
                            }).into_view(cx)
                        },
                            ImageFetcher::Cached (image) => {
                                 let data_uri = format!("data:image/png;base64, {}", image.image);
                                         view!{cx,
                                               <img src=data_uri alt="Image" />
                                        }.into_view(cx)
                        }
                    }
            }
             <span class="absolute block rounded-full
             bg-green-500 ring-2 ring-white top-0
             right-0 h-2 w-2 md:h-3 md:w-3"/>
            </Suspense>
        </div>
    }
}

#[component]
fn UserList(cx: Scope) -> impl IntoView {
    view! {cx,
        <aside class="fixed inset-y-0 pb-20 lg:pb-0 lg:left-20 lg:w-80 lg:block overflow-y-auto border-r border-gray-200 block w-full left-0">
            <div class="px-5">
                <div class="flex-col">
                    <div class="text-2xl font-bold text-neutral-800 py-4">
                        "Users"
                    </div>
                </div>
                <UserBox/>
            </div>
        </aside>
    }
}

#[component]
fn UserBox(cx: Scope) -> impl IntoView {
    fn callback(cx: Scope, id: i32) -> impl Fn(MouseEvent) {
        move |_event: MouseEvent| {
            let _ = create_local_resource(
                cx,
                || (),
                move |_| async move { conversation_action(cx, vec![id], false, None).await },
            );
        }
    }
    let users_arr = create_resource(cx, || (), move |_| async move { get_users(cx).await });
    view! {cx,
        <Suspense fallback=loading_fallback(cx)>
            {move || users_arr.read(cx).map(|items| {
                let items = items.unwrap();
                view!{cx,
                        <For
                          each=move || items.clone()
                          key=|items| items.id
                          view=move |cx, item: UserModel| {
                            let id = create_local_resource(cx, ||(), move |_| async move {associated_conversation(cx, item.id).await});
                            let name = create_rw_signal(cx, format!("{} {}", item.first_name, item.last_name));
                            view! {
                              cx,
                                <Suspense fallback=||()>
                                    {move || id.read(cx).map(|conversation_id|
                                        view!{cx,
                                         <A href=format!("/conversations/{}", conversation_id.unwrap())>
                                             <div class="w-full relative flex
                                                 items-center space-x-3 bg-white
                                                 p-3 hover:bg-neutral-100 rounded-lg
                                                 transition cursor-pointer"
                                                 on:click=callback(cx, item.id)
                                                 >
                                                     <Avatar id=item.id/>
                                                     <div class="min-w-0 flex-1">
                                                         <div class="focus:outline-none">
                                                             <div class="flex justify-between items-center mb-1">
                                                                 <p class="text-sm font-medium text-gray-900">
                                                                     {move || name.get()}
                                                                 </p>
                                                             </div>
                                                         </div>
                                                     </div>
                                             </div>
                                         </A>
                                        })}
                                </Suspense>
                                  }
                          }
                        />
                    }
            })}
        </Suspense>
    }
}

#[component]
pub fn Conversations(cx: Scope) -> impl IntoView {
    use_context::<IsOpen>(cx).unwrap().status.set(false);
    view! {cx,
        <ConversationsLayout>
            <Outlet/>
        </ConversationsLayout>
    }
}

#[component]
fn ConversationsLayout(cx: Scope, children: Children) -> impl IntoView {
    let conversations = create_local_resource(
        cx,
        || (),
        move |_| async move { get_conversations(cx).await.unwrap() },
    );

    let group_chat_context = create_rw_signal(cx, false);

    view! {cx,
        <Sidebar>
            <div class="h-screen">
                <GroupChatModal context=group_chat_context/>
                <Suspense fallback=||()>
                {move || conversations.read(cx).map(|val: Vec<MergedConversation>|{
                view!{cx,
                                <aside class=move || format!("fixed inset-y-0 pb-20 lg:pb-0
                                    lg:left-20 lg:w-80 lg:block overflow-y-auto border-r
                                    border-gray-200 {}", if use_context::<IsOpen>(cx).unwrap().status.get()
                                    {"hidden"} else {"block w-full left-0"})>
                                    <div class="px-5">
                                        <div class="flex justify-between mb-4 pt-4">
                                            <div class="text-2xl font-bold text-neutral-800">
                                                "Messages"
                                            </div>
                                            <div class="rounded-full p-2 bg-gray-100 text-gray-600
                                            hover:opacity-75 transition cursor-pointer" on:click=move |_| group_chat_context.set(true)>
                                                <Icon icon=AiIcon::AiUserAddOutlined class="text-20 text-gray-500"/>
                                            </div>
                                        </div>
                                     <For
                                       each=move || val.clone()
                                       key=|val| val.conversation_id
                                       view=move |cx, item: MergedConversation| {
                                         view! {
                                           cx,
                                            <ConversationBox item/>
                                  }}/>
                                    </div>
                                </aside>
                          }
                })
                 }
                </Suspense>
                {children(cx)}
            </div>
        </Sidebar>
    }
}

#[component]
fn ConversationBox(cx: Scope, item: MergedConversation) -> impl IntoView {
    let seen_status = create_rw_signal(cx, false);
    let cloned_item = item.clone();

    let message = create_resource(
        cx,
        move || item.clone(),
        move |item| async move {
            if let Some(message) = item.conversation.messages.last() {
                if let Some(message_body) = &message.message_body {
                    message_body.to_owned()
                } else if message.message_image.is_some() {
                    String::from("Sent an image")
                } else {
                    String::from("Started a conversation")
                }
            } else {
                String::from("Started a conversation")
            }
        },
    );

    if let Some(message) = cloned_item.conversation.messages.last() {
        seen_status.set(message.seen_status.iter().any(|messages| {
            messages.seen_id.unwrap() == use_context::<UserContext>(cx).unwrap().id.get()
        }))
    }
    let query = move || {
        use_location(cx)
            .pathname
            .get()
            .contains(&cloned_item.conversation_id.to_string())
    };

    view! {cx,
        <A href=format!("/conversations/{}", &cloned_item.conversation_id.to_string())
                class=move || format!("w-full relative flex items-center space-x-3 hover:bg-neutral-100 rounded-lg transition cursor-pointer p-3 {}",
                if query() {"bg-neutral-100"} else {"bg-white"})>
            {

                match cloned_item.conversation.is_group {
                true => view!{cx, <><AvatarGroup user_ids=cloned_item.conversation.user_ids/></> },
                false => view!{cx, <><Avatar id=*cloned_item.conversation.user_ids.first().unwrap()/></> }
                }
            }
            <div class="min-w-0 flex-1">
                <div class="focus:outline-none">
                    <div class="flex justify-between items-center mb-1">
                        <p class="text-md font-medium font-bold text-gray-900">
                            {
                                match cloned_item.conversation.is_group {
                                    true => cloned_item.conversation.name.unwrap(),
                                    false => cloned_item.conversation.first_name + " " + &cloned_item.conversation.last_name
                                }

                            }
                        </p>
                        <p>

                        </p>
                    </div>
                <p class=move || format!("text-sm {}", if seen_status.get()
                        {"text-gray-500"} else {"text-black font-medium"})>
                    <Suspense fallback=loading_fallback(cx)>
                        {move || message.read(cx).map(|message|
                            view!{cx,
                                <>
                                {message}
                                </>
                        }
                )}
                    </Suspense>
                </p>
                </div>
            </div>
        </A>
    }
}

#[component]
pub fn ConversationId(cx: Scope) -> impl IntoView {
    let current_id = get_current_id(cx);
    use_context::<IsOpen>(cx).unwrap().status.set(true);
    let conversation = create_local_resource(cx, current_id, move |current_id| async move {
        validate_conversation(cx, current_id).await
    });

    let messages = create_local_resource(cx, current_id, move |current_id| async move {
        view_messages(cx, current_id).await
    });

    create_effect(cx, move |_| {
        spawn_local(async move {
            create_local_resource(cx, current_id, move |current_id| async move {
                handle_seen(cx, current_id).await.unwrap();
            });
        })
    });

    view! {cx,
        <ConfirmModal/>
        <div class="lg:pl-80 h-screen">
            <div class="h-screen flex flex-col">
                <Suspense fallback=||()>
                 {move ||
                    conversation.read(cx).map(|conversations| {
                        if ! conversations.iter().len().gt(&0) {
                            view!{cx,
                                    <>
                                        <EmptyState/>
                                    </>
                            }
                        } else {
                            view!{cx,
                                    <>
                                        <Header conversation=conversations.unwrap()/>
                                            {move || messages.read(cx).map(|messages|
                                                view!{cx, <Body messages=messages.unwrap()/>
                                            })}
                                        <MessageForm current_id/>
                                    </>
                            }
                    }
                })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn Header(cx: Scope, conversation: Vec<ConversationMeta>) -> impl IntoView {
    let status_text = create_rw_signal(cx, String::from(""));

    let drawer_status = create_rw_signal(cx, false);

    if let Some(banner_conversation) = conversation.first() {
        if banner_conversation.is_group != 0 {
            status_text.set(banner_conversation.count.to_string() + " members")
        } else {
            status_text.set(String::from("Active"))
        }
    } else {
        queue_microtask(move || use_navigate(cx)("/user", Default::default()).unwrap());
    };

    view! {cx,
        <>
        <ProfileDrawer data=conversation.first().unwrap() is_open=move || drawer_status.get() on_close=move |_| drawer_status.set(false)/>
        <div class="bg-white w-full flex border-b-[1px] sm:px-4
            py-3 px-4 lg:px-6 justify-between items-center shadow-sm">
            <div class="flex gap-3 items-center">
                <A href="/conversations"
                    class="lg:hidden block text-sky-500
                    hover:text-sky-600 transition cursor-pointer">
                         <Icon icon=HiIcon::HiChevronLeftSolidLg style="font-size: 16px; stroke: currentColor" on:click=move |_| {
                                    use_context::<IsOpen>(cx).unwrap().status.set(false);
                                }
                        />
                </A>
                {
                    conversation.first().is_some().then(|| {
                        let conversation = conversation.first().unwrap();
                        match conversation.is_group.eq(&1) {
                            true => view!{cx, <><AvatarGroup user_ids=conversation.other_users.iter().map(|(_,_,id)| *id).collect()/></> },
                            false => view!{cx, <><Avatar id=conversation.other_users.first().unwrap().2/></> }
                    }})
                }
                <div class="flex flex-col">
                    <div>
                        {move ||
                            conversation
                               .first()
                               .map(|conversation_getter| {
                                   if conversation_getter.name.is_some() && conversation_getter.is_group.ne(&0) {
                                       conversation_getter.name.clone().unwrap()
                                   } else {
                                       conversation_getter.other_users.first().unwrap().0.clone()
                                   }
                               })
                               .unwrap_or_else(String::new)
                        }
                    </div>
                    <div class="text-sm font-light text-neutral-500">
                        {status_text}
                    </div>
                </div>
            </div>
             <Icon icon=Icon::Hi(leptos_icons::HiIcon::HiEllipsisHorizontalSolidMd) class="text-sky-500 cursor-pointer" style="font-size: 16px; stroke: currentColor" on:click=move |_| drawer_status.set(true)/>
        </div>
        </>
    }
}

#[component]
fn Body(cx: Scope, messages: Vec<MergedMessages>) -> impl IntoView {
    let last: i32;

    if let Some(message) = messages.last() {
        last = message.message_id;
    } else {
        last = 0;
    }

    view! {cx,
        <div class="flex-1 overflow-y-auto ">
               <For
                 each=move || messages.clone()
                 key=|message| message.message_sender_id
                 view=move |cx, item: MergedMessages| {
                   view! {
                     cx,
                      <MessageBox message=item.clone() is_last=(last == item.message_id)/>
                    }}/>
            <div class="pt-24" />
        </div>
    }
}

#[component]
fn MessageForm<F>(cx: Scope, current_id: F) -> impl IntoView
where
    F: Fn() -> i32 + 'static,
{
    let _input_ref = create_node_ref::<html::Input>(cx);
    let image_ref = create_node_ref::<html::Input>(cx);

    let id = current_id();
    let on_submit_callback = move |event: SubmitEvent| {
        event.prevent_default();
        event.stop_propagation();

        let body = _input_ref.get().unwrap().value();

        spawn_local(async move {
            if let Some(files) = image_ref.get().unwrap().files() {
                let list = gloo_file::FileList::from(files);
                if let Some(file) = list.first() {
                    let file = Some(gloo_file::futures::read_as_bytes(file).await.unwrap());
                    let _ = handle_message_input(cx, id, None, file).await;
                } else {
                    let _ = handle_message_input(cx, id, Some(body), None).await;
                }
            }
        })
    };

    view! {cx,
         <form on:submit=on_submit_callback class="py-4 px-4 bg-white border-t flex items-center gap-2 lg:gap-4 w-full ">
             <label for="submission">
                     <Icon icon=TbIcon::TbPhotoFilled class="text-sky-500"
                     style="font-size: 32px; stroke: currentColor; fill: currentColor"/>
             </label>
                 <input type="file" _ref=image_ref id="submission" class="hidden"/>
             <div class="flex items-center gap-2 lg:gap-4 w-full">
                 <MessageInput _input_ref/>
             </div>
             <button type="submit" class="rounded-full p-2 bg-sky-500 cursor-pointer hover:bg-sky-600 transition">
                 <Icon icon=HiIcon::HiPaperAirplaneOutlineLg width="18px" class="text-white" style="stroke: white; fill: white"/>
             </button>
         </form>
    }
}

#[component]
fn MessageInput(cx: Scope, _input_ref: NodeRef<html::Input>) -> impl IntoView {
    view! {cx,
        <div class="relative w-full">
            <input required=false placeholder="Write a message..." _ref=_input_ref
              class="text-black font-light py-2 px-4 bg-neutral-100 w-full rounded-full focus:outline-none">
            </input>
        </div>
    }
}

#[component]
fn MessageBox(cx: Scope, message: MergedMessages, is_last: bool) -> impl IntoView {
    let is_own =
        move || use_context::<UserContext>(cx).unwrap().id.get() == message.message_sender_id;

    let image_modal_context = create_rw_signal(cx, false);

    let seen_list: String = message
        .seen_status
        .into_iter()
        .filter(|users| users.seen_id.unwrap() != message.message_sender_id)
        .map(|messages| messages.first_name.unwrap() + " " + &messages.last_name.unwrap() + " ")
        .collect();

    let message_image = message.message_image.clone();
    let image_status = create_local_resource(
        cx,
        move || message_image.clone(),
        move |message_image| async move {
            if let Some(image) = message_image {
                ImageEnum::Some(find_image(cx, image).await)
            } else {
                ImageEnum::None
            }
        },
    );

    let image_signal = create_rw_signal(cx, String::new());

    let message_class = format!(
        "text-sm w-fit overflow-hidden {} {}",
        if is_own() {
            "bg-sky-500 text-white"
        } else {
            "bg-gray-100"
        },
        if message.message_image.is_some() {
            "rounded-md p-0 "
        } else {
            "rounded-full py-2 px-3"
        }
    );

    view! {cx,
        <div class=move || format!( "flex gap-3 p-4 {}", if is_own() { "justify-end" } else { "" })>
            <div class=move|| if is_own() { "order-2" } else { "" }>
                <Avatar id=message.message_sender_id/>
            </div>
            <div class=move || format!( "flex flex-col gap-2 {}", if is_own() { "items-end" } else { "" })>
                <div class="flex items-center gap-1">
                    <div class="text-sm text-gray-500">
                        {message.first_name + " " + &message.last_name}
                    </div>
                    <div class="text-xs text-gray-400">
                        {
                            format_created_at(message.created_at)
                        }
                    </div>
                </div>
                <div class=message_class>
                        {
                            if let Some(image) = message.message_image {
                            image_signal.set(image);
                            view!{cx,
                                <>
                                <Suspense fallback=loading_fallback(cx)>
                                 {move || image_status.read(cx).map(|status|
                                     match status {
                                             ImageEnum::Some(Ok(ImageAvailability::Found)) =>
                                                  view!{cx,
                                                      <>
                                                         <ImageModal src=move || image_signal.get() context=image_modal_context/>
                                                         <img on:click=move |_| image_modal_context.set(true) alt="Image"
                                                          src=move || image_signal.get() class="object-cover cursor-pointer hover:scale-110
                                                          transition translate w-auto max-w-[288px] max-h-[288px]"/>
                                                      </>
                                                  },
                                             _ =>
                                                  view!{cx,
                                                      <>
                                                          <Icon icon=LuIcon::LuImageOff width="36px" height="36px" class="text-white"/>
                                                      </>
                                                  },
                                     })}
                                </Suspense>
                                </>
                            }
                            } else {
                                view!{cx,
                                    <>
                                    {message.message_body}
                                    </>
                                }
                            }
                        }
                </div>
                {move || {
                    (is_last && is_own() && seen_list.len().gt(&0)).then(||
                    view!{cx,
                        <div class="text-xs font-light text-gray-500">
                            {format!("Seen by {}", seen_list)}
                        </div>
                    }
                )}}
            </div>
        </div>
    }
}

#[component]
fn ProfileDrawer<F, FN, 'a>(
    cx: Scope,
    is_open: FN,
    on_close: F,
    data: &'a ConversationMeta,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + 'static,
    FN: Fn() -> bool + 'static + Clone,
{
    let other_user = data.other_users.clone();
    let cloned_data = data.clone();
    let title = create_memo(cx, move |_| match cloned_data.is_group.eq(&1) {
        true => cloned_data.name.clone().unwrap(),
        false => other_user.first().unwrap().0.clone(),
    });
    let data_clone = data.clone();
    let status_text = create_memo(cx, move |_| {
        if data_clone.is_group.gt(&0) {
            format!("{} members", data_clone.count)
        } else {
            "Active".to_string()
        }
    });

    let drawer_context = use_context::<DrawerContext>(cx).unwrap();

    view! {cx,
        <div class=move || format!("transition ease-in delay-300 {}", if is_open() {"block"} else {"hidden"})>
            <div class="relative z-40">
                <div class="fixed inset-0 bg-black bg-opacity-40"/>
            <div class="fixed inset-0 overflow-hidden">
                <div class="fixed inset-0 overflow-hidden">
                    <div class="pointer-events-none fixed inset-y-0 right-0 flex max-w-screen pl-10">
                         <div class="pointer-events-auto w-screen max-w-md">
                             <div class="transition delay-300 translate flex h-screen flex-col overflow-y-scroll py-6 bg-white shadow-xl">
                                 <div class="px-4 sm:px-6">
                                     <div class="flex items-start justify-end">
                                         <div class="ml-3 flex h-7 items-center">
                                             <button type="button" class="rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none
                                                focus:ring-2 focus:ring-sky-500 focus:ring-offset-2">
                                                 <span class="sr-only">"Close Panel"</span>
                                                          <Icon icon=IoIcon::IoClose width="24px" height="24px" on:click=on_close/>
                                             </button>
                                         </div>
                                     </div>
                                 </div>
                                <div class="relative mt-6 flex-1 px-4 sm:px-6">
                                    <div class="flex flex-col items-center">
                                        <div class="mb-2">
                                             {
                                                 let conversation = data;
                                                 match conversation.is_group.eq(&1) {
                                                     true => view!{cx, <><AvatarGroup user_ids=conversation.other_users.iter().map(|(_,_,id)| *id).collect()/></> },
                                                     false => view!{cx, <><Avatar id=conversation.other_users.first().unwrap().2/></> }
                                                  }
                                             }
                                        </div>
                                        <div>
                                            {title}
                                        </div>
                                        <div class="text-sm text-gray-500">
                                            {move || status_text}
                                        </div>
                                        <div class="flex gap-10 my-8">
                                            <div on:click=move |_| () class="flex flex-col gap-3 items-center hover:opacity-75 cursor-pointer" on:click=move |_| drawer_context.status.set(true)>
                                                <div class="w-10 h-10 bg-neutral-100 rounded-full flex items-center justify-center">
                                                          <Icon icon=IoIcon::IoTrash width="20px" height="24px"/>
                                                </div>
                                                <div class="text-sm font-light text-neutral-600">
                                                    "Delete"
                                                </div>
                                            </div>
                                        </div>
                                        <div class="w-full pb-5 pt-5 sm:px-0 sm:pt-0">
                                            <dl class="space-y-8 px-4 sm:space-y-6 sm:px-6">
                                                {data.is_group.eq(&1).then(|| {
                                                    let data = data.clone();
                                                    view!{cx,
                                                         <div>
                                                             <dt class="text-sm font-medium text-gray-500 sm:w-40 sm:flex-shrink-0">
                                                                 "Emails"
                                                             </dt>
                                                             <dd class="mt-1 text-sm text-gray-900 sm:col-span-2">
                                                                  <For
                                                                    each=move || data.clone().other_users
                                                                    key=|users| users.clone().0
                                                                    view=move |cx, user: (String, String, i32)| {
                                                                      view! {
                                                                        cx,
                                                                          <ul>
                                                                             <li>
                                                                                 {format!("{} - {}", user.0, user.1)}
                                                                             </li>
                                                                          </ul>
                                                                       }}/>
                                                                                                               // {data.other_users.first().unwrap().1.clone()}
                                                             </dd>
                                                         </div>
                                                    }
                                                 })}
                                                {data.is_group.eq(&0).then(|| {
                                                    view!{cx,
                                                    <div>
                                                        <dt class="text-sm font-medium text-gray-500 sm:w-40 sm:flex-shrink-0">
                                                            "Email"
                                                        </dt>
                                                        <dd class="mt-1 text-sm text-gray-900 sm:col-span-2">
                                                            {data.other_users.first().unwrap().1.clone()}
                                                        </dd>
                                                    </div>
                                                    }
                                                 })}
                                                {data.is_group.eq(&0).then(|| {
                                                    let data = data.clone();
                                                    view!{cx,
                                                        <>
                                                        <hr />
                                                        <div>
                                                            <dt class="text-sm font-medium text-gray-500 sm:w-40 sm:flex-shrink-0">
                                                                "Joined"
                                                            </dt>
                                                            <dd class="mt-1 text-sm text-gray-900 sm:col-span-2">
                                                                <time dateTime=data.clone().created_at>
                                                                    {data.clone().created_at}
                                                                </time>
                                                            </dd>
                                                        </div>
                                                        </>
                                                    }
                                                  })
                                                }
                                            </dl>
                                        </div>
                                    </div>
                                </div>
                             </div>
                         </div>
                    </div>
                </div>
            </div>
            </div>
        </div>
    }
}

#[component]
fn Modal(cx: Scope, children: Children, context: RwSignal<bool>) -> impl IntoView {
    let on_close = move |_| context.set(false);
    view! {cx,
    <div class=move || format!("absolute h-screen w-screen inset-0 z-50 bg-gray-500 bg-opacity-50 {}", if context.get() {"block"} else {"hidden"})>
     <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div class="bg-gray-500 bg-opacity-50"></div>
      <div class="absolute bottom-1/2 bg-white rounded-lg shadow-xl p-4 sm:p-6 mb-4 w-1/2 translate-x-1/2 translate-y-1/2">
        <div class="absolute hidden sm:block pr-4 pt-4 text-right top-0 right-0">
          <button
            type="button"
            className="rounded-md bg-white text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
            on:click=on_close
          >
            <span class="sr-only">"Close"</span>
            <Icon icon=IoIcon::IoClose class="h-6 w-6 text-gray-900"/>
          </button>
        </div>
        {children(cx)}
      </div>
    </div>
    </div>
    }
}

#[component]
fn Button<F>(
    cx: Scope,
    on_click: F,
    children: Children,
    button_type: &'static str,
    disabled: ButtonVal,
    color: &'static str,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + 'static,
{
    let disabled_val = move || match disabled {
        ButtonVal::Bool(val) => val,
        ButtonVal::RwSignal(val) => val.get(),
    };

    view! {cx,
        <button
          on:click=on_click type=button_type disabled=disabled_val.clone()
          class=move || format!("flex justify-center rounded-md px-3 py-2 text-sm
                font-semibold focus-visible:outline focus-visible:outline-2 
                focus-visible:outline-offset-2 focus-visible:outline-sky-600
                text-white {} {}", color, if disabled_val() {"hidden"} else {""})>
          {children(cx)}
        </button>
    }
}

#[component]
fn ConfirmModal(cx: Scope) -> impl IntoView {
    let drawer_context = use_context::<DrawerContext>(cx).unwrap().status;
    let on_click = move |_| {
        spawn_local(async move {
            delete_conversations(cx, get_current_id(cx)())
                .await
                .unwrap()
        })
    };

    view! {cx,
        <Modal context=drawer_context>
            <div class="sm:flex sm:items-start">
                <div class="mx-auto flex h-12 w-12 flex-shrink-0 items-center rounded-full justify-center bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                    <Icon icon=FiIcon::FiAlertTriangle class="h-6 w-6 text-red-600"/>
                </div>
            <div class="mt-3 text-center sm:ml-4 sm:mt-0 sm:text-left">
                <div class="text-base font-semibold leading-6 text-gray-900">
                    "Delete Conversation"
                </div>
                <div class="mt-2">
                    <p class="text-sm text-gray-500">
                        "Are you sure you want to delete this conversation? This action CANNOT be undone"
                    </p>
                </div>
            </div>
            </div>
        <div class="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
            <Button on_click=on_click button_type="button" disabled=ButtonVal::Bool(false) color="bg-rose-500 hover:bg-rose-600 focus-visible:outline-rose-600">
                "Delete"
            </Button>
        </div>
        </Modal>
    }
}

#[component]
fn SettingsModal(cx: Scope, settings_modal_setter: RwSignal<bool>) -> impl IntoView {
    let user = create_local_resource(
        cx,
        move || (),
        move |_| async move { get_user(cx).await.unwrap() },
    );

    let image_ref = create_node_ref::<Input>(cx);
    let first_name_ref = create_node_ref::<Input>(cx);
    let last_name_ref = create_node_ref::<Input>(cx);
    let button_signal = create_rw_signal(cx, true);
    let disable_signal = create_rw_signal(cx, false);

    let clear_val = move |_: web_sys::MouseEvent| {
        first_name_ref.get_untracked().unwrap().set_value("");
        last_name_ref.get_untracked().unwrap().set_value("");
        image_ref.get_untracked().unwrap().set_value("");
    };

    let submit = move |_: MouseEvent| {
        disable_signal.set(true);

        let first_name_val = match first_name_ref.get().unwrap().value().as_str() {
            "" => None,
            _ => Some(first_name_ref.get().unwrap().value()),
        };

        let last_name_val = match last_name_ref.get().unwrap().value().as_str() {
            "" => None,
            _ => Some(last_name_ref.get().unwrap().value()),
        };

        spawn_local(async move {
            if let Some(files) = image_ref.get_untracked().unwrap().files() {
                let list = gloo_file::FileList::from(files);
                if let Some(file) = list.first() {
                    let file = gloo_file::futures::read_as_bytes(file).await.unwrap();
                    upload_user_info(cx, Some(file), first_name_val, last_name_val)
                        .await
                        .unwrap();
                } else {
                    upload_user_info(cx, None, first_name_val, last_name_val)
                        .await
                        .unwrap();
                }
            }
            clear_val(MouseEvent::new("click").unwrap());
            disable_signal.set(false);
            settings_modal_setter.set(false);
        })
    };

    view! {cx,
        <Modal context=settings_modal_setter>
            <form>
                <div class="space-y-12">
                    <div class="border-b border-gray-900/10 pb-12">
                        <h2 class="text-base font-semibold leading-7 text-gray-900">
                        </h2>
                        <p class="mt-1 text-sm leading-6 text-gray-600">
                            "Edit your public information."
                        </p>
                        <Suspense fallback=||()>
                        {move || user.read(cx).map(|user|
                            view!{cx,
                                <div class="mt-10 flex flex-col gap-y-8">
                                    <UserInput id="first_name" _ref=first_name_ref input_type="text" label="First Name" required=false disabled=ButtonVal::RwSignal(disable_signal) placeholder=user.first_name/>
                                </div>
                                <div class="mt-10 flex flex-col gap-y-8">
                                    <UserInput id="last_name" _ref=last_name_ref input_type="text" label="Last Name" required=false disabled=ButtonVal::RwSignal(disable_signal) placeholder=user.last_name/>
                                </div>
                                <div class="mt-10 flex flex-col gap-y-3">
                                    <label class="block text-sm font-medium leading-6 text-gray-900">
                                        "Photo"
                                    </label>
                                        {
                                            if let Some(image) = user.image {
                                                 view!{cx,
                                                     <>
                                                     <img width="48px" height="48px" class="rounded-full" src=move || "/".to_string() + &image/>
                                                     </>
                                                 }
                                            } else {
                                                 view!{cx,
                                                     <>
                                                        <Icon icon=BiIcon::BiUserCircleSolid
                                                        class="w-10 h-10 text-gray-400"/>
                                                     </>
                                                     }
                                                   }
                                                }
                                        <div class="flex gap-x-3">
                                              <Button on_click=move |_| image_ref.get().unwrap().click() button_type="button" disabled=ButtonVal::Bool(false) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                                                "Upload"
                                              </Button>
                                            <input _ref=image_ref type="file" class="hidden" id="upload" name="upload" on:change=move |_| button_signal.set(false)>
                                            </input>
                                        </div>
                                </div>
                            })}
                        </Suspense>
                    </div>
                    <div class="mt-6 flex items-center justify-end gap-x-6">
                         <Button on_click=clear_val button_type="button" disabled=ButtonVal::Bool(false) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                          "Cancel"
                        </Button>
                         <Button on_click=submit button_type="button" disabled=ButtonVal::Bool(false) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                          "Save Changes"
                        </Button>
                    </div>
                </div>
            </form>
        </Modal>
    }
}

#[component]
fn UserInput(
    cx: Scope,
    id: &'static str,
    label: &'static str,
    input_type: &'static str,
    required: bool,
    disabled: ButtonVal,
    placeholder: String,
    _ref: NodeRef<Input>,
) -> impl IntoView {
    let disabled_val = move || match disabled {
        ButtonVal::Bool(val) => val,
        ButtonVal::RwSignal(val) => val.get(),
    };

    view! {cx,
     <div>
          <label for=id class=" block text-sm font-medium leading-6 text-gray-900 " >
            {label}
          </label>
          <div class="mt-2">
            <input id=id type=input_type name=id disabled=disabled_val() required=required _ref=_ref
              class=move || format!("form-input block w-full rounded-md border-0 py-1.5
                text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 
                focus:ring-2 focus:outline-none focus:ring-inset pl-4 focus:ring-sky-600 sm:text-sm sm:leading-6 {}",
                if disabled_val() {"opacity-50 cursor-default"} else {""}) placeholder=placeholder/>
          </div>
        </div>
    }
}

#[component]
fn GroupChatModal(cx: Scope, context: RwSignal<bool>) -> impl IntoView {
    let disable_signal = create_rw_signal(cx, false);
    let input_signal = create_rw_signal(cx, vec![(view! {cx, <div/>}, 0)]);
    let name_ref = create_node_ref::<Input>(cx);
    let form_ref = create_node_ref::<html::Form>(cx);
    let action = create_server_action::<CreateGroupConversation>(cx);
    let input_ref = create_node_ref::<Input>(cx);
    let clear_input = move |_| {
        name_ref.get().unwrap().set_value("");
        input_ref.get().unwrap().set_value("");
        input_signal.set(vec![(view! {cx, <div/>}, 0)])
    };
    let err_result_signal = create_rw_signal(cx, String::new());
    view! {cx,
        <Modal context>
            <ActionForm action node_ref=form_ref>
                <div class="space-y-12">
                    <div class="border-b border-gray-900/10 pb-12">
                        <h2 class="text-base font-semibold leading-7 text-gray-900">
                            "Create a group chat"
                         </h2>
                        <p class="mt-1 text-sm leading-6 text-gray-600">
                            "Create a chat with more than two people"
                        </p>
                        <p>
                            {move || err_result_signal}
                        </p>
                        <div class="mt-10 flex flex-col gap-y-8">
                            <UserInput id="name" label="Group Name" input_type="text" required=true disabled=ButtonVal::RwSignal(disable_signal) placeholder=String::from("Group Name...") _ref=name_ref/>
                            <input name="is_group" value="true" class="hidden"/>
                            <Select disabled=disable_signal label="Members" _ref=input_ref input_signal/>
                        </div>
                    </div>
                </div>
            <div class="mt-6 flex items-center justify-end gap-x-6">
                 <Button on_click=clear_input button_type="button" disabled=ButtonVal::RwSignal(disable_signal) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                    "Cancel"
                 </Button>
                 <Button on_click=move |_| {
                        match form_ref.get().unwrap().submit() {
                            Ok(_) => context.set(false),
                            Err(e) => err_result_signal.set(e.as_string().unwrap())
                        }
                    } button_type="submit" disabled=ButtonVal::RwSignal(disable_signal) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                    "Create"
                 </Button>
            </div>
            </ActionForm>
        </Modal>
    }
}

#[component]
fn Select(
    cx: Scope,
    disabled: RwSignal<bool>,
    label: &'static str,
    _ref: NodeRef<Input>,
    input_signal: RwSignal<Vec<(HtmlElement<Div>, i32)>>,
) -> impl IntoView {
    let hidden_state = create_rw_signal(cx, true);

    let users = create_local_resource(
        cx,
        move || hidden_state.get(),
        move |_| async move { get_users(cx).await.unwrap() },
    );
    view! {cx,
        <div class="z-[100]">
            <label class="block text-sm font-medium leading-6 text-gray-900">
                {label}
            </label>
            <div class="mt-2 relative">
              <input type="text" class=move || format!("w-full py-2 px-4 border border-gray-300 rounded-md
                focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500
                text-transparent select-none selection:bg-none {}", if disabled.get() {"opacity-50"} else {""})
                placeholder="Select an option"
                name="other_users"
                _ref=_ref
                on:click=move |_| hidden_state.update(|val| *val = !*val )/>
                     <For
                       each=move || input_signal.get()
                       key=|input| input.1
                       view=move |_cx, item: (HtmlElement<Div>, i32)| {
                         view! {
                           _cx,
                             {item.0}
                        }
                     }/>
              <ul class=move || format!("absolute z-10 mt-1 w-full bg-white border border-gray-300 rounded-md shadow-lg {}", if hidden_state.get() {"hidden"} else {"block"})>
                <Suspense fallback=||()>
                     {move || users.read(cx).map(|options|
                         view!{cx,
                             <For
                               each=move || options.clone()
                               key=|user| user.id
                               view=move |cx, item: UserModel| {
                                      let li_ref = create_node_ref::<html::Li>(cx);
                                      let input = _ref.get().unwrap();
                                          (! input.value().contains(&(item.id.to_string()))).then(|| {
                                            view! {
                                              cx,
                                               <li value=item.id _ref=li_ref class="text-sm z-[9999] px-4 py-2 cursor-pointer hover:bg-gray-100" on:click=move |_| {
                                                    let input_ref = input.clone();
                                                    let input = input.clone();
                                                    hidden_state.set(true);
                                                    let link = li_ref.get().unwrap();
                                                    let link_ref = link.clone();
                                                    let value = move || {
                                                    match input_ref.value().chars().last() {
                                                            Some(char) => {
                                                                if char == ',' {
                                                                    (input_ref.value() + &link_ref.value().to_string(), link_ref.value().to_string())
                                                                } else {
                                                                    (link_ref.value().to_string() + "," + &input_ref.value(), link_ref.value().to_string() + ",")
                                                                }
                                                            },
                                                            None => {
                                                                    (link_ref.value().to_string() + "," + &input_ref.value(), link_ref.value().to_string() + ",")
                                                            }
                                                        }};

                                                    input.set_value(&(value().0));
                                                    input_signal.update(|val| {
                                                    val.push((
                                                    view!{cx,
                                                            <div class="flex mt-2 gap-x-3 text-sm border-gray-300 rounded-md bg-sky-200 p-2 w-fit" id=item.id>
                                                                {link.inner_text()}
                                                                <Icon icon=IoIcon::IoClose class="h-3 w-3" on:click=move |_| {
                                                                    input_signal.update(|val| {
                                                                        let index = val.iter().position(|(_, id)| *id == link.value()).unwrap();
                                                                        val.remove(index);
                                                                    });
                                                                    (!input_signal.get().iter().any(|(_, id)| *id == link.value())).then(|| {
                                                                        input.set_value(&(input.value().replace(&(link.value().to_string() + ","), "")));
                                                                        input.set_value(&(input.value().replace(&(link.value().to_string()), "")));
                                                                    });
                                                                }/>
                                                            </div>
                                                    },item.id)
                                                )})}>
                                                    {item.first_name + " " + &item.last_name}
                                               </li>
                                         }
                                         })
                                     }/>
                         })}
                </Suspense>
            </ul>
        </div>
        </div>
    }
}

#[component]
fn AvatarGroup(cx: Scope, user_ids: Vec<i32>) -> impl IntoView {
    let mut user_ids = user_ids;
    user_ids.truncate(3);
    let user_ids = user_ids.into_iter().enumerate().collect::<Vec<_>>();
    view! {cx,
        <div class="relative h-11 w-11">
               <For
                 each=move || user_ids.clone()
                 key=|(position,_)| *position
                 view=move |cx, user: (usize, i32)| {
                    let mut position_map: HashMap<i8, &str> = HashMap::new();
                    position_map.insert(0, "top-0 left-[12px]");
                    position_map.insert(1, "bottom-0");
                    position_map.insert(2, "bottom-0 right-0");
                    let image = create_local_resource(cx, ||(), move |_| async move {get_image(cx, user.1).await});
                          view! {
                            cx,
                              <div class=move || format!("absolute flex rounded-full overflow-y-auto h-[21px] w-[21px] justify-center {}",
                                  position_map.get(&(user.0 as i8)).unwrap())>
                               <Suspense fallback=loading_fallback(cx)>
                                  {move ||
                                      image.read(cx).map(|image| {
                                        let image = image.unwrap_or_default();
                                        if let Some(image) = image {
                                        let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(image);
                                        let data_uri = format!("data:image/png;base64, {}", base64_encoded_image);
                                              view! {cx,
                                                  <>
                                                       <img src=data_uri alt="Avatar" fill />
                                                  </>
                                                }
                                          } else {
                                              view! {cx,
                                                  <>
                                                            <Icon icon=BiIcon::BiUserCircleSolid
                                                            class="text-gray-400 h-[21px] w-[21px]"/>
                                                  </>
                                          }
                                          }})
                                  }
                               </Suspense>
                            </div>
                           }}/>
        </div>
    }
}

#[component]
fn ImageModal<F>(cx: Scope, context: RwSignal<bool>, src: F) -> impl IntoView
where
    F: Fn() -> String + 'static,
{
    view! {cx,
        <Modal context>
            <div class="max-w-[80%] max-h-[80%]">
                <img alt="image" class="object-cover" src=src/>
            </div>
        </Modal>
    }
}
