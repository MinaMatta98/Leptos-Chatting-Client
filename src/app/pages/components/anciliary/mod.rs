use leptos::{html::Input, *};
use leptos_icons::*;
use leptos_router::*;

use crate::{
    app::{
        pages::{conversation::ConversationParams, Avatar, SettingsModal, ICONVEC, SINKVEC},
        IsOpen, SideBarContext,
    },
    server_function::{self, login_status, UserLogin},
};

use super::avatar::STREAMVEC;

#[derive(PartialEq, Clone, Debug)]
pub struct UserContext {
    pub id: RwSignal<i32>,
    pub email: RwSignal<String>,
    pub first_name: RwSignal<String>,
    pub last_name: RwSignal<String>,
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
            icon: leptos_icons::Icon::Hi(HiIcon::HiChatBubbleOvalLeftEllipsisSolidMd),
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
            icon: leptos_icons::Icon::Hi(HiIcon::HiUserCircleSolidMd),
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
            icon: leptos_icons::Icon::Bi(BiIcon::BiChevronLeftSquareSolid),
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

pub fn loading_fallback(cx: Scope) -> Box<dyn Fn() -> View> {
    Box::new(move || {
        view! {cx,
                <div class="relative flex items-center justify-center rounded-full
                    max-w-[48px] min-w-[21px] min-h-[21px] font-semibold text-sm shadow
                    text-white bg-indigo-400 hover:bg-indigo-500 transition ease-in-out
                    duration-150 cursor-not-allowed">
                         <svg class="animate-spin max-h-[48px] max-w-[48px] min-w-[22px] min-h-[22px] text-white z-50" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                             <circle class="opacity-25" cx="12" cy="12" r="10" stroke="white" stroke-width="4"></circle>
                             <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                         </svg>
                </div>
        }.into_view(cx)
    })
}
pub struct UserContexts;

impl UserContexts {
    pub fn init_users(cx: Scope) {
        provide_context(
            cx,
            UserContext {
                id: create_rw_signal(cx, 0),
                email: create_rw_signal(cx, String::from("")),
                first_name: create_rw_signal(cx, String::from("")),
                last_name: create_rw_signal(cx, String::from("")),
            },
        );

        provide_context(
            cx,
            SideBarContext {
                status: create_rw_signal(cx, false),
            },
        );
        provide_context(
            cx,
            IsOpen {
                status: create_rw_signal(cx, false),
            },
        );
    }
}

#[derive(Clone)]
pub enum ButtonVal {
    Bool(bool),
    RwSignal(RwSignal<bool>),
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
pub fn Sidebar(cx: Scope, children: Children) -> impl IntoView {
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

    let settings_modal_setter = create_rw_signal(cx, false);

    let status = create_local_resource(cx, || (), move |_| async move { login_status(cx).await });

    view! {cx,
        <Suspense fallback=||()>
                {move || status.read(cx).map(|status| {

                     let UserLogin {email, id, first_name, last_name} = status.expect("Error obtaining user context");
                     let user_context = use_context::<UserContext>(cx).unwrap();
                     user_context.email.set(email);
                     user_context.id.set(id);
                     user_context.first_name.set(first_name);
                     user_context.last_name.set(last_name);

                     view!{cx,
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
                         <nav class="mt-4 flex flex-col
                             justify-center items-center">
                                 <div class="cursor-pointer
                                     hover:opacity-75 transition" on:click=move |_| settings_modal_setter.set(true)>
                                 <Avatar id/>
                                 </div>
                         </nav>
                    </div>
                   }
                }
            )}
        </Suspense>
    }
}

#[component]
fn MobileFooter(cx: Scope) -> impl IntoView {
    let params = use_params::<ConversationParams>(cx);

    view! {cx,
            {move ||
                    params().map(|params| params.id).is_err().then(|| {
                        view! {cx,
                            <div class="fixed justify-between
                            w-screen bottom-0 z-40 flex items-center
                            bg-white border-t-[1px] lg:hidden">
                                 <For each=move || SidebarIcon::init(cx)
                                   key=|vec| vec.href.to_string()
                                   view=move |cx, item: SidebarIcon| {
                                     view! {cx, <MobileItem item /> }
                                 }/>
                            </div>
                        }
                    })
            }
    }
}

#[component]
fn MobileItem(cx: Scope, item: SidebarIcon<'static>) -> impl IntoView {
    view! {cx,
        <A href=item.href class=move || format!("group flex gap-x-3 text-sm
            leading-6 font-semibold w-full justify-center p-4 hover:bg-gray-100 {}", (item.active)(cx))
            on:click=move |_|
            if let Some(function) = &item.on_click {
                    function(cx);
                    ICONVEC.write().clear();
                    STREAMVEC.write().clear();
                    SINKVEC::send_clear();
            }
            >
            <Icon icon=item.icon class="h-6 w-6"
                style="fill: currentColor"
            />
        </A>
    }
}

#[component]
fn DesktopItem(cx: Scope, item: SidebarIcon<'static>) -> impl IntoView {
    view! { cx,
         <A on:click=move |_|
            if let Some(function) = &item.on_click {
                    function(cx);
                    ICONVEC.write().clear();
                    STREAMVEC.write().clear();
                    SINKVEC::send_clear();
            }
                href=item.href
                  class=move || format!("group flex gap-x-3 rounded-md p-4 text-sm leading-6 font-semibold
                     text-gray-800 hover:text-black hover:bg-gray-100 {}", (item.active)(cx))>
                  <Icon icon=item.icon class="h-6 w-6 shrink-0"
                    style="fill: currentColor"
                  />
         </A>
    }
}

#[component]
pub fn Button<F>(
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
        <label for="upload">
        <button
          on:click=on_click type=button_type disabled=disabled_val.clone()
          class=move || format!("flex justify-center rounded-md px-3 py-2 text-sm
                font-semibold focus-visible:outline focus-visible:outline-2 
                focus-visible:outline-offset-2 focus-visible:outline-sky-600
                text-white {} {}", color, if disabled_val() {"hidden"} else {""})>
          {children(cx)}
        </button>
        </label>
    }
}

#[component]
pub fn UserInput(
    cx: Scope,
    id: &'static str,
    label: &'static str,
    input_type: &'static str,
    required: bool,
    disabled: ButtonVal,
    placeholder: String,
    _node_ref: NodeRef<Input>,
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
            <input id=id type=input_type name=id disabled=disabled_val() required=required node_ref=_node_ref
              class=move || format!("form-input block w-full rounded-md border-0 py-1.5
                text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 
                focus:ring-2 focus:outline-none focus:ring-inset pl-4 focus:ring-sky-600 sm:text-sm sm:leading-6 {}",
                if disabled_val() {"opacity-50 cursor-default"} else {""}) placeholder=placeholder/>
          </div>
    </div>
    }
}
