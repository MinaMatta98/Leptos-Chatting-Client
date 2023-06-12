// use crate::{entities::users, server_function::GetUsers};
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;
use web_sys::MouseEvent;

use crate::{
    app::IsOpen,
    server_function::{
        self, conversation_action, get_conversations, get_users, validate_conversation,
        view_messages, ConversationMeta, MergedConversation, UserModel,
    },
};

#[derive(Params, PartialEq, Clone, Debug)]
struct ConversationIdParams {
    id: i32,
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
    pub id: RwSignal<String>,
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
fn EmptyState(cx: Scope) -> impl IntoView {
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
            <Icon icon=Icon::Ai(leptos_icons::AiIcon::AiUserOutlined)
            class="w-8 h-8 mx-auto"/>
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
            let _ = leptos_router::use_navigate(cx)(
                &("/conversations/".to_string() + &item.id.to_string()),
                Default::default(),
            );
        }
    }

    on_cleanup(cx, || {
        log!("cleaning up <UserBox/>");
    });

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
            <div>
               <div class=move || format!("lg:pl-80 h-screen
                    lg:block {}", if use_context::<IsOpen>(cx)
                .unwrap().status.get() {"block"} else {"hidden"})>
                   <EmptyState />
               </div>
            </div>
        </ConversationsLayout>
    }
}

#[component]
fn ConversationsLayout(cx: Scope, children: Children) -> impl IntoView {
    let conversation_status = create_rw_signal(cx, String::from(""));
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
                                            hover:opacity-75 transition cursor-pointer">
                                                <Icon icon=Icon::Ai(leptos_icons::AiIcon::AiUserAddOutlined) class="text-20 text-gray-500"/>
                                            </div>
                                        </div>
                                     <For
                                       each=move || val.clone()
                                       key=|val| val.conversation_id
                                       view=move |cx, item: MergedConversation| {
                                                {let item_clone = item.clone();
                                                      create_effect(cx, move |_| {
                                                      if let Some(message) = item_clone.clone().conversation.messages.last() {
                                                          if let Some(message_body) = &message.message_body {
                                                              conversation_status.set(message_body.to_owned())
                                                          } else if message.message_image.is_some() {
                                                              conversation_status.set(String::from("Sent a message"))
                                                          } else {
                                                              conversation_status.set(String::from("Started a conversation"))
                                                          }
                                                      }
                                                      else {
                                                          conversation_status.set(String::from("Started a conversation"))
                                                      }});
                                               }
                                         view! {
                                           cx,
                                            <ConversationBox item conversation_status/>
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
fn ConversationBox(
    cx: Scope,
    item: MergedConversation,
    conversation_status: RwSignal<String>,
) -> impl IntoView {
    let seen_status = create_rw_signal(cx, false);
    if let Some(message) = item.conversation.messages.last() {
        seen_status.set(message.seen_status)
    }
    // let last_message = item.;
    view! {cx,
        <div on:click=move |_| use_navigate(cx)(&("/conversations/".to_string() + &item.conversation_id.to_string()), Default::default()).unwrap()
            class=move || format!("w-full relative flex items-center space-x-3 hover:bg-neutral-100 rounded-lg transition cursor-pointer p-3 {}",
                if use_context::<IsOpen>(cx).unwrap().status.get() {"bg-neutral-100"} else {"bg-white"})>
            <Avatar />
            <div class="min-w-0 flex-1">
                <div class="focus:outline-none">
                    <div class="flex justify-between items-center mb-1">
                        <p class="text-md font-medium text-gray-900">
                            { item.conversation.first_name + " " + &item.conversation.last_name }
                        </p>
                        <p>

                        </p>
                    </div>
                <p class=move || format!("text-sm {}", if seen_status.get()
                        {"text-gray-500"} else {"text-black font-medium"})>
                    {move || conversation_status.get()}
                </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ConversationId(cx: Scope) -> impl IntoView {
    let current_id = use_params::<ConversationIdParams>(cx);
    let conversation = create_local_resource(
        cx,
        || (),
        move |_| async move {
            validate_conversation(cx, current_id.get().map(|params| params.id).unwrap()).await
        },
    );

    let messages = create_local_resource(
        cx,
        || (),
        move |_| async move {
            view_messages(cx, current_id.get().map(|params| params.id).unwrap()).await
        },
    );

    view! {cx,
        <Suspense fallback=||()>
         {move ||
            conversation.read(cx).map(|conversations| {
                if ! conversations.iter().len().gt(&0) {
                    // queue_microtask(move || {use_navigate(cx)("/user", Default::default()).unwrap();});
                    view!{cx,
                        <div class="lg:pl-80 h-screen">
                            <div class="h-screen flex flex-col">
                                <EmptyState/>
                            </div>
                        </div>
                    }
            } else {
                    view!{cx,
                        <div class="lg:pl-80 h-screen">
                            <div class="h-screen flex flex-col">
                                <Header conversation=conversation.read(cx).unwrap().unwrap()/>
                            </div>
                        </div>
                    }
            }
        })}
        </Suspense>
    }
}

#[component]
fn Header(cx: Scope, conversation: Vec<ConversationMeta>) -> impl IntoView {
    let conversation = conversation.first().unwrap();
    let status_text = {
        if conversation.is_group != 0 {
            conversation.count.to_string() + "members"
        } else {
            String::from("Active")
        }
    };
    view! {cx,
        <div class="bg-white w-full flex border-b-[1px] sm:px-4 
            py-3 px-4 lg:px-6 justify-between items-center shadow-sm">
            <div class="flex gap-3 items-center">
                <link href="/conversations" 
                    class="lg:hidden block text-sky-500
                    hover:text-sky-600 transition cursor-pointer">
                         <Icon icon=Icon::Hi(leptos_icons::HiIcon::HiChevronLeftSolidLg) style="size: 32px"/>
                </link>
            </div>
        </div>
    }
}
