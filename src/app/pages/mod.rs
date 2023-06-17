use leptos::{html::Dialog, *};
use leptos_icons::*;
use leptos_router::*;
use web_sys::{MouseEvent, SubmitEvent};

use crate::{
    app::IsOpen,
    server_function::{
        self, associated_conversation, conversation_action, find_image, get_conversations,
        get_users, handle_message_input, handle_seen, validate_conversation, view_messages,
        ConversationMeta, ImageAvailability, MergedConversation, MergedMessages, SeenMessageFacing,
        UserModel,
    },
};

#[derive(Params, PartialEq, Clone, Debug, Eq)]
struct ConversationIdParams {
    id: i32,
}

#[derive(Clone)]
pub enum ImageEnum {
    Some(std::result::Result<server_function::ImageAvailability, leptos::ServerFnError>),
    None,
}

pub struct SidebarIcon<'a> {
    _label: &'a str,
    href: &'a str,
    icon: Icon,
    active: Box<dyn Fn(Scope) -> &'a str>,
    on_click: Option<Box<dyn std::ops::Fn(Scope)>>,
}

impl<'a> SidebarIcon<'a> {
    pub fn init(cx: Scope) -> Vec<SidebarIcon<'a>> {
        let chat = SidebarIcon {
            _label: "Chat",
            href: "/conversations",
            icon: Icon::Hi(leptos_icons::HiIcon::HiChatBubbleOvalLeftEllipsisSolidMd),
            active: Box::new(move |_| {
                let path = use_location(cx).pathname.get();
                let id = use_context::<ConversationParams>(cx);
                (path == "conversations" || id.map(|id| id.id).is_some())
                    .then_some("bg-gray-100 text-black")
                    .map_or_else(|| "", |v| v)
            }),
            on_click: Some(Box::new(move |_| {
                queue_microtask(move || {
                    let _ = leptos_router::use_navigate(cx)("/conversations", Default::default());
                })
            })),
        };

        let users = SidebarIcon {
            _label: "Users",
            href: "/users",
            icon: Icon::Hi(leptos_icons::HiIcon::HiUserCircleSolidMd),
            active: Box::new(move |_| {
                (use_location(cx).pathname.get().as_str() == "/user")
                    .then_some("bg-gray-100 text-black")
                    .map_or_else(|| "", |v| v)
            }),
            on_click: Some(Box::new(move |_| {
                queue_microtask(move || {
                    let _ = leptos_router::use_navigate(cx)("/user", Default::default());
                })
            })),
        };

        let logout = SidebarIcon {
            _label: "Logout",
            href: "#",
            icon: Icon::Hi(leptos_icons::HiIcon::HiArrowLeftCircleOutlineLg),
            active: Box::new(move |_| ""),
            on_click: Some(Box::new(|cx| {
                create_resource(
                    cx,
                    || (),
                    async move |_| server_function::logout(cx).await.unwrap(),
                );
                queue_microtask(move || use_navigate(cx)("/", Default::default()).unwrap());
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

    view! {cx,
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
            })}
            </Suspense>
            <nav class="mt-4 flex flex-col
                justify-center items-center">
                    <div class="cursor-pointer
                        hover:opacity-75 transition">
                            <Avatar/>
                    </div>
            </nav>
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
        <li href=item.href class=move ||
            format!("group flex gap-x-3 text-sm
            leading-6 font-semibold w-full justify-center
            p-4 hover:bg-gray-100 {}", (item.active)(cx))
            style="color: gray"
            on:click=move |_| if let Some(function) =
            &item.on_click {function(cx)}>
            <Icon style="gray" icon=item.icon
                class="h-6 w-6"/>
        </li>
    }
}

#[component]
fn DesktopItem(cx: Scope, item: SidebarIcon<'static>) -> impl IntoView {
    view! { cx,
    <li on:click=move |_| if let Some(function)
             = &item.on_click {function(cx)} href=item.href
             class=move || format!("group flex gap-x-3
             rounded-md p-4 text-sm leading-6 font-semibold
             text-gray-500 hover:text-black hover:bg-gray-100 {}
             ", (item.active)(cx))>
    <Icon icon=item.icon class="h-6 w-6 shrink-0"
             style="color: red"/>
    </li>
    }
}

#[component]
fn Avatar(cx: Scope) -> impl IntoView {
    view! {cx,
        <div class="relative inline-block
            rounded-full
            h-9 w-9 md:h-11 md:w-11 text-slate-800">
            <Icon icon=Icon::Bi(leptos_icons::BiIcon::BiUserCircleSolid)
            class="w-10 h-10 mx-auto text-gray-400"/>
            <span class="absolute block rounded-full
            bg-green-500 ring-2 ring-white top-0
            right-0 h-2 w-2 md:h-3 md:w-3"/>
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
    fn callback(cx: Scope, item: UserModel) -> impl Fn(MouseEvent) {
        move |_event: MouseEvent| {
            // let conversation = create_server_action::<ConversationAction>(cx);
            // conversation.dispatch(ConversationAction {other_user: item.id});
            let _ = create_local_resource(
                cx,
                || (),
                move |_| async move { conversation_action(cx, item.id).await },
            );
            spawn_local(async move {
                let _ = leptos_router::use_navigate(cx)(
                    &("/conversations/".to_string()
                        + &associated_conversation(cx, item.id)
                            .await
                            .unwrap()
                            .unwrap()
                            .to_string()),
                    Default::default(),
                );
            })
        }
    }

    let users_arr = create_resource(cx, || (), move |_| async move { get_users(cx).await });
    view! {cx,
        <Suspense fallback=||()>
            {move || users_arr.read(cx).map(|items| {
                let items = items.unwrap();
                view!{cx,
                        <For
                          each=move || items.clone()
                          key=|items| items.id
                          view=move |cx, item: UserModel| {
                            view! {
                              cx,
                                <div class="w-full relative flex
                                    items-center space-x-3 bg-white
                                    p-3 hover:bg-neutral-100 rounded-lg
                                    transition cursor-pointer"
                                    on:click=callback(cx, item.clone())
                                    >
                                        <Avatar/>
                                        <div class="min-w-0 flex-1">
                                            <div class="focus:outline-none">
                                                <div class="flex justify-between items-center mb-1">
                                                    <p class="text-sm font-medium text-gray-900">
                                                        {item.first_name + " " + &item.last_name}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                </div>
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

    view! {cx,
        <Sidebar>
            <div class="h-screen">
                <Suspense fallback=||()>
                {move || conversations.read(cx).map(|val: Vec<MergedConversation>|{
                view!{cx,
                                <aside class=move || format!("fixed hidden inset-y-0 pb-20 lg:pb-0
                                    lg:left-20 lg:w-80 lg:block overflow-y-auto border-r
                                    border-gray-200 {}", if use_context::<IsOpen>(cx).unwrap().status.get()
                                    {"hidden"} else {"block w-full left-0"})>
                                    <div class="px-5">
                                        <div class="flex justify-between mb-4 pt-4">
                                            <div class="text-2xl font-bold text-neutral-800">
                                                "Messages"
                                            </div>
                                            <div class="rounded-full p-2 bg-gray-100 text-gray-600
                                            hover:opacity-75 transition cursor-pointer">
                                                <Icon icon=Icon::Ai(leptos_icons::AiIcon::AiUserAddOutlined) class="text-20 text-gray-500"/>
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
        <div on:click=move |_| use_navigate(cx)(&("/conversations/".to_string() + &cloned_item.conversation_id.to_string()), Default::default()).unwrap()
            class=move || format!("w-full relative flex items-center space-x-3 hover:bg-neutral-100 rounded-lg transition cursor-pointer p-3 {}",
                if query() {"bg-neutral-100"} else {"bg-white"})>
            <Avatar />
            <div class="min-w-0 flex-1">
                <div class="focus:outline-none">
                    <div class="flex justify-between items-center mb-1">
                        <p class="text-md font-medium font-bold text-gray-900">
                            { cloned_item.conversation.first_name + " " + &cloned_item.conversation.last_name }
                        </p>
                        <p>

                        </p>
                    </div>
                <p class=move || format!("text-sm {}", if seen_status.get()
                        {"text-gray-500"} else {"text-black font-medium"})>
                    <Suspense fallback=||()>
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
        </div>
    }
}

#[component]
pub fn ConversationId(cx: Scope) -> impl IntoView {
    let current_id = move || {
        use_params::<ConversationIdParams>(cx)
            .get()
            .map(|params| params.id)
            .unwrap()
    };

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
                                            view!{cx,
                                                <Body messages=messages.unwrap()/>
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
            status_text.set(banner_conversation.count.to_string() + "members")
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
                <link href="/conversations"
                    class="lg:hidden block text-sky-500
                    hover:text-sky-600 transition cursor-pointer">
                         <Icon icon=Icon::Hi(leptos_icons::HiIcon::HiChevronLeftSolidLg) style="font-size: 16px; stroke: currentColor"/>
                </link>
                <Avatar/>
                <div class="flex flex-col">
                    <div>
                        {move ||
                            conversation
                               .first()
                               .map(|conversation_getter| {
                                   if !conversation_getter.name.is_empty() && conversation_getter.is_group.ne(&0) {
                                       conversation_getter.name.clone()
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
            <input required=true placeholder="Write a message..." _ref=_input_ref
              class="text-black font-light py-2 px-4 bg-neutral-100 w-full rounded-full focus:outline-none">
            </input>
        </div>
    }
}

#[component]
fn MessageBox(cx: Scope, message: MergedMessages, is_last: bool) -> impl IntoView {
    let is_own =
        move || use_context::<UserContext>(cx).unwrap().id.get() == message.message_sender_id;

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
                <Avatar/>
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
                                <Suspense fallback=||()>
                                 {move || image_status.read(cx).map(|status|
                                     match status {
                                             ImageEnum::Some(Ok(ImageAvailability::Found)) =>
                                                  view!{cx,
                                                      <>
                                                         <img alt="Image" max-height="288" max-width="288" width="auto" src=move || image_signal.get() class="object-cover cursor-pointer hover:scale-110 transition translate"/>
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
    let title = create_memo(cx, move |_| other_user.first().unwrap().0.clone());
    let data_clone = data.clone();
    let status_text = create_memo(cx, move |_| {
        if data_clone.is_group.gt(&0) {
            format!("{} members", data_clone.count)
        } else {
            "Active".to_string()
        }
    });

    view! {cx,
        <div class=move || format!("transition ease-in delay-300 {}", if is_open() {"block"} else {"hidden"})>
            <div class="relative z-50">
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
                                            <Avatar/>
                                        </div>
                                        <div>
                                            {title}
                                        </div>
                                        <div class="text-sm text-gray-500">
                                            {move || status_text}
                                        </div>
                                        <div class="flex gap-10 my-8">
                                            <div on:click=move |_| () class="flex flex-col gap-3 items-center cursor-pointer hover:opacity-75">
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
                                                {data.is_group.eq(&0).then(|| {
                                                    view!{cx,
                                                    <div>
                                                        <dt class="text-sm font-medium text-gray-500 sm:w-40 sm:flex-shrink-0">
                                                            "Email"
                                                        </dt>
                                                        <dd class="empty-1 text-sm text-gray-900 sm:col-span-2">
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
                                                            <dd class="empty-1 text-sm text-gray-900 sm:col-span-2">
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
