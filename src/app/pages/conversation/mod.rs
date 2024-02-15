use leptos::*;
use leptos_icons::*;
use leptos_router::*;
use std::borrow::Cow;
use web_sys::SubmitEvent;

use base64::{engine::general_purpose, Engine as _};
use chrono::SubsecRound;
use fancy_regex::Regex;

use crate::{
    app::{
        pages::components::anciliary::{loading_fallback, EmptyState, Sidebar},
        DrawerContext, IsOpen, MessageDrawerContext, SeenContext, SeenContextInner,
    },
    server_function::{
        self,
        routes::{
            find_image, get_conversations, handle_seen, login_status, validate_conversation,
            view_messages,
        },
        ConversationMeta, ImageAvailability, MergedConversation, MergedMessages, SeenMessageFacing,
        UserLogin,
    },
};

use super::{
    avatar::*,
    components::{
        anciliary::UserContexts,
        avatar::{ICONVEC, STREAMVEC},
    },
    get_image,
    modal::*,
    GroupChatModal, HandleWebSocket, UserContext, UserInputHandler,
};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ImageEnum {
    Some(std::result::Result<server_function::ImageAvailability, leptos::ServerFnError>),
    None,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct Message {
    pub message: Option<String>,
    pub image: Option<String>,
    pub conversation_id: i32,
    pub user_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub seen: Option<Vec<(String, String)>>,
    pub message_id: i32,
}
impl UserContexts {
    fn init_conversation(cx: Scope) {
        provide_context(
            cx,
            SeenContext {
                status: create_rw_signal(cx, Vec::new()),
            },
        );

        provide_context(
            cx,
            DrawerContext {
                status: create_rw_signal(cx, false),
            },
        );

        provide_context(
            cx,
            MessageDrawerContext {
                status: create_rw_signal(cx, false),
            },
        );
    }

    fn init_all(cx: Scope) {
        Self::init_users(cx);
        Self::init_conversation(cx);
    }
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ConversationParams {
    pub id: usize,
    pub none: String,
}

pub fn get_current_id(cx: Scope) -> impl Fn() -> i32 + 'static + Copy {
    move || {
        use_params::<ConversationIdParams>(cx)
            .get()
            .map(|params| params.id)
            .unwrap_or_default()
    }
}

fn format_created_at(created_at: String) -> String {
    let created_at = created_at.trim_end_matches(" UTC").trim();
    let time =
        chrono::NaiveTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S").unwrap_or_default();
    time.format("%-I:%M %p").to_string()
}

#[derive(Params, PartialEq, Clone, Debug, Eq)]
struct ConversationIdParams {
    id: i32,
}

#[component]
pub fn Conversations(cx: Scope) -> impl IntoView {
    UserContexts::init_all(cx);
    let status = create_local_resource(cx, || (), move |_| async move { login_status(cx).await });
    use_context::<IsOpen>(cx).unwrap().status.set(false);

    leptos::on_cleanup(cx, || {
        ICONVEC.write().clear();
        STREAMVEC.write().clear();
        SINKVEC::send_clear();
    });

    view! {cx,
            <Suspense fallback=||()>
                {move || status.read(cx).map(|status| {
                    let user_context = use_context::<UserContext>(cx).unwrap();
                    let UserLogin {email, id, first_name, last_name} = status.expect("Error obtaining user context");
                    user_context.email.set(email);
                    user_context.id.set(id);
                    user_context.first_name.set(first_name);
                    user_context.last_name.set(last_name);
                view!{cx,
                    <ConversationsLayout>
                        <Outlet/>
                    </ConversationsLayout>
                }
                })}
            </Suspense>
    }
}

#[component]
fn ConversationsLayout(cx: Scope, children: Children) -> impl IntoView {
    let conversations = create_resource(
        cx,
        move || {
            use_context::<MessageDrawerContext>(cx)
                .unwrap()
                .status
                .get()
        },
        move |_| async move { get_conversations(cx).await },
    );

    let group_chat_context = create_rw_signal(cx, false);

    view! {cx,
        <Sidebar>
            <div class="h-screen">
                <Suspense fallback=||()>
                {move || conversations.read(cx).map(|val|{
                view!{cx,
                            <GroupChatModal context=group_chat_context/>
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
                                                <Icon icon=Icon::from(AiIcon::AiUserAddOutlined) class="text-20 text-gray-500"/>
                                            </div>
                                        </div>
                                     <For
                                       each=move || val.clone().unwrap()
                                       key=|val| val.conversation_id
                                       view=move |cx, item: MergedConversation| {
                                        view! {cx,
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
    let secondary_cloned_item = item.clone();

    let message_signal = create_rw_signal(cx, String::new());

    if let Some(message) = item.conversation.messages.last() {
        if let Some(message_body) = &message.message_body {
            message_signal.set(message_body.to_owned())
        } else if message.message_image.is_some() {
            message_signal.set(String::from("Sent an image"))
        } else {
            message_signal.set(String::from("Started a conversation"))
        }
    } else {
        message_signal.set(String::from("Started a conversation"))
    };

    if let Some(message) = cloned_item.conversation.messages.last() {
        seen_status.set(message.seen_status.iter().any(|messages| {
            messages.seen_id.unwrap() == use_context::<UserContext>(cx).unwrap().id.get()
        }))
    }
    let query = move || {
        use_location(cx)
            .pathname
            .get()
            .split('/')
            .last()
            .unwrap()
            .eq(&cloned_item.conversation_id.to_string())
    };

    spawn_local(async move {
        HandleWebSocket::handle_split_stream::<String, Message>(
            cx,
            item.conversation_id,
            Some(message_signal),
            "ws://localhost:8000/ws/",
            |signal, value: Message| {
                match value.image {
                    Some(_) => *signal.unwrap() = String::from("Image Sent in Chat"),
                    None => *signal.unwrap() = value.message.unwrap(),
                };
            },
        )
        .await
    });

    view! {cx,
        <A href=format!("{}", &cloned_item.conversation_id.to_string())
                class=move || format!("w-full relative flex items-center space-x-3 hover:bg-neutral-100 rounded-lg transition cursor-pointer p-3 {}",
                if query() {"bg-neutral-100"} else {"bg-white"})>
                <Suspense fallback=||()>
                    {let cloned_item = cloned_item.clone();
                        move ||
                        match cloned_item.conversation.is_group {
                        true => view!{cx, <><AvatarGroup user_ids=cloned_item.conversation.user_ids.clone()/></> },
                        false => view!{cx, <>
                            <Avatar id=*cloned_item.conversation.user_ids.iter().find(|&&user| user != use_context::<UserContext>(cx).unwrap().id.get()).unwrap()/></> }
                        }
                    }
                </Suspense>
            <div class="min-w-0 flex-1">
                <div class="focus:outline-none">
                    <div class="flex justify-between items-center mb-1">
                        <p class="text-md font-medium font-bold text-gray-900">
                            {
                                match secondary_cloned_item.conversation.is_group {
                                    true => secondary_cloned_item.conversation.name.unwrap(),
                                    false => secondary_cloned_item.conversation.first_name + " " + &secondary_cloned_item.conversation.last_name
                                }
                            }
                        </p>
                        <p>
                        </p>
                    </div>
                <p class=move || format!("text-sm {}", if seen_status.get()
                        {"text-gray-500"} else {"text-black font-medium"})>
                                {move || message_signal}
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
    let conversation = create_resource(cx, current_id, move |current_id| async move {
        validate_conversation(cx, current_id).await
    });

    let messages = create_resource(cx, current_id, move |current_id| async move {
        view_messages(cx, current_id).await
    });

    create_effect(cx, move |_| {
        spawn_local(async move {
            handle_seen(cx, current_id()).await.unwrap();
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
                                            {move || messages.read(cx).map(|messages| {
                                                view!{cx,
                                                    <Body messages=messages.unwrap()
                                                    />
                                                }
                                            })}
                                        <MessageForm/>
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
        <ProfileDrawer data=conversation.first().unwrap() is_open=move || drawer_status.get() on_close=move |_| drawer_status.set(false)/>
        <div class="bg-white w-full flex border-b-[1px] sm:px-4
            py-3 px-4 lg:px-6 justify-between items-center shadow-sm">
            <div class="flex gap-3 items-center">
                <A href="/conversations"
                    class="lg:hidden block text-sky-500
                    hover:text-sky-600 transition cursor-pointer">
                         <Icon icon=Icon::from(HiIcon::HiChevronLeftSolidLg) style="font-size: 16px; stroke: currentColor" on:click=move |_| {
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
    }
}

#[component]
fn Body(cx: Scope, messages: Vec<MergedMessages>) -> impl IntoView {
    let messages_signal = create_rw_signal(cx, Vec::new());

    let seen_context = use_context::<SeenContext>(cx).unwrap().status;
    let id = get_current_id(cx)();

    let boxed_messages = Box::new(messages);

    let messages = boxed_messages.clone();

    let last = move || {
        if let Some(index) = seen_context
            .get()
            .iter()
            .position(|context| context.conversation_id == get_current_id(cx)())
        {
            seen_context.get().get(index).unwrap().last_message_id
        } else {
            let last_message = if let Some(last_message) = boxed_messages.clone().iter().last() {
                last_message.message_id
            } else {
                0
            };

            seen_context.update(|seen_vec| {
                seen_vec.push(SeenContextInner {
                    conversation_id: id,
                    last_message_id: last_message,
                })
            });
            last_message
        }
    };

    let last_clone = last.clone();
    spawn_local(async move {
        HandleWebSocket::handle_split_stream::<Vec<MergedMessages>, Message>(
            cx,
            get_current_id(cx)(),
            Some(messages_signal),
            "ws://localhost:8000/ws/",
            move |message_vec, value: Message| {
                seen_context.update(|last| {
                    last.iter_mut()
                        .find(|context| context.conversation_id == id)
                        .map(|context| context.last_message_id += 1)
                        .unwrap_or_default()
                });
                message_vec.unwrap().push(MergedMessages {
                    first_name: value.first_name.clone(),
                    last_name: value.last_name.clone(),
                    created_at: chrono::Utc::now().trunc_subsecs(0).to_string(),
                    message_sender_id: value.user_id,
                    message_body: value.message.clone(),
                    message_image: value.image,
                    message_conversation_id: value.conversation_id,
                    seen_status: value
                        .seen
                        .unwrap()
                        .iter()
                        .map(|(first_name, last_name)| SeenMessageFacing {
                            seen_id: Some(value.user_id),
                            message_id: None,
                            first_name: Some(first_name.to_string()),
                            last_name: Some(last_name.to_string()),
                        })
                        .collect(),
                    message_id: last() + 1,
                })
            },
        )
        .await;
    });

    view! {cx,
            <div class="flex-1 overflow-y-auto ">
                 <For
                      each=move || *messages.clone()
                      key=|val| val.message_id
                      view=move |cx, item: MergedMessages| {
                      view! { cx,
                         <MessageBox message=item.clone()
                              is_last=(last_clone() == item.message_id)
                          />
                      }
                }/>
                    {move ||
                        messages_signal.get().iter().map(|item|
                            view! {cx,
                                <MessageBox message=item.clone() is_last=(
                                    if let Some(messages) = messages_signal.get().last() {
                                        messages.message_id == item.message_id
                                    } else {
                                        false
                                    }
                                )/>
                            }
                        ).collect_view(cx)
                    }
             <div id="bottom_ref" class="pt-24"/>
        </div>
    }
}

#[component]
fn MessageForm(cx: Scope) -> impl IntoView {
    let _input_ref = create_node_ref::<html::Input>(cx);
    let image_ref = create_node_ref::<html::Input>(cx);

    let on_submit_callback = move |event: SubmitEvent| {
        event.prevent_default();
        event.stop_propagation();

        spawn_local(async move {
            UserInputHandler::handle_message(cx, image_ref, _input_ref, get_current_id(cx)()).await;
            image_ref.get_untracked().unwrap().set_value("");
            _input_ref.get_untracked().unwrap().set_value("");
        });
    };

    view! {cx,
         <form on:submit=on_submit_callback class="py-4 px-4 bg-white border-t flex items-center gap-2 lg:gap-4 w-full ">
             <label for="submission">
                     <Icon icon=Icon::from(TbIcon::TbPhotoFilled) class="text-sky-500"
                     style="font-size: 32px; stroke: currentColor; fill: currentColor"/>
             </label>
                 <input type="file" _ref=image_ref id="submission" class="hidden"/>
             <div class="flex items-center gap-2 lg:gap-4 w-full">
                 <MessageInput _input_ref/>
             </div>
             <button type="submit" class="rounded-full p-2 bg-sky-500 cursor-pointer hover:bg-sky-600 transition">
                 <Icon icon=Icon::from(HiIcon::HiPaperAirplaneOutlineLg) width="18px" class="text-white" style="stroke: white; fill: white"/>
             </button>
         </form>
    }
}

#[component]
fn MessageInput(cx: Scope, _input_ref: NodeRef<html::Input>) -> impl IntoView {
    view! {cx,
        <div class="relative w-full">
            <input required=false placeholder="Write a message..." node_ref=_input_ref
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

    let regex = Regex::new(&format!(
        r"^(.*?){}(.*?){}(.*)$",
        message.first_name, message.last_name
    ))
    .unwrap();

    let seen_list: Cow<str> = message
        .seen_status
        .into_iter()
        .map(|messages| messages.first_name.unwrap() + " " + &messages.last_name.unwrap() + " ")
        .collect();

    let seen_list = match seen_list {
        Cow::Borrowed(s) => regex.replace(s, "$1$3").into_owned(),
        Cow::Owned(s) => regex.replace(&s, "$1$3").into_owned(),
    };

    let message_image = message.message_image.clone();
    let image_status = create_resource(
        cx,
        move || message_image.clone(),
        move |message_image| async move {
            if let Some(image) = message_image {
                match IMAGEVEC
                    .read()
                    .iter()
                    .any(|image_vec| image_vec.path == image.clone())
                {
                    true => ImageEnum::Some(Ok(ImageAvailability::Found)),
                    false => ImageEnum::Some(find_image(cx, image).await),
                }
            } else {
                ImageEnum::None
            }
        },
    );

    let image_signal = create_rw_signal(cx, view! {cx, <><div/></>});

    let message_class = format!(
        "text-sm w-fit overflow-hidden {} {}",
        if is_own() && message.message_image.is_none() {
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
            <div class=move || format!("flex flex-col gap-2 {}", if is_own() { "items-end" } else { "" })>
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
                            view!{cx,
                                <>
                                <Suspense fallback=loading_fallback(cx)>
                                 {let image = image.clone();
                                    move || image_status.read(cx).map(|status| {
                                    let image = image.clone();
                                     match status {
                                             ImageEnum::Some(Ok(ImageAvailability::Found)) => {
                                                match IMAGEVEC.read().iter().find(|&image_vec| image_vec.path == image.clone()) {
                                                    Some(image) => {
                                                        image_signal.set(
                                                            view!{cx,
                                                                   <>
                                                                       <ImageModal src=image.image.clone() context=image_modal_context/>
                                                                       <img on:click=move |_| image_modal_context.set(true) alt="Image"
                                                                        src=image.image.clone() class="object-cover cursor-pointer hover:scale-110
                                                                        transition translate w-auto max-w-[288px] max-h-[288px]"/>
                                                                   </>
                                                            })
                                                    }
                                                    None => {
                                                            let cloned_image = image.clone();
                                                            let retrieved_vec = create_resource(cx, move || cloned_image.clone(), move |image| async move {get_image(cx, image.clone()).await.unwrap()});
                                                            image_signal.set(
                                                                    view!{cx,
                                                                        <>
                                                                             <Suspense fallback=loading_fallback(cx)>
                                                                                 {let image = image.clone();
                                                                                 move || retrieved_vec.read(cx).map(|vec_u8| {
                                                                                     let base64_encoded_image = general_purpose::STANDARD_NO_PAD.encode(vec_u8.unwrap());
                                                                                     let data_uri = base_64_encode_uri(base64_encoded_image);
                                                                                     IMAGEVEC.write().push(UserImage {
                                                                                         path: image.clone(),
                                                                                         image: data_uri.clone()
                                                                                     });
                                                                                     view!{cx,
                                                                                         <ImageModal src=data_uri.clone() context=image_modal_context/>
                                                                                         <img on:click=move |_| image_modal_context.set(true) alt="Image"
                                                                                          src=data_uri.clone() class="object-cover cursor-pointer hover:scale-110
                                                                                          transition translate w-auto max-w-[288px] max-h-[288px]"/>
                                                                                     }
                                                                                 })}
                                                                             </Suspense>
                                                                        </>
                                                                    }
                                                        )
                                                        }
                                                    }
                                                  view!{cx,
                                                      <>
                                                         {move || image_signal.get()}
                                                      </>
                                                  }
                                                },
                                             _ => {
                                                  view!{cx,
                                                      <>
                                                          <Icon icon=Icon::from(LuIcon::LuImageOff) width="36px" height="36px" class="text-white"/>
                                                      </>
                                                  }
                                                }
                                     }})}
                                </Suspense>
                                </>
                            }} else {
                                view!{cx,
                                    <>
                                        {message.message_body}
                                    </>
                                }
                            }
                        }
                </div>
                {move || {
                    (is_last && is_own() && seen_list.trim().len().gt(&0)).then(||
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
                                                          <Icon icon=Icon::from(IoIcon::IoClose) width="24px" height="24px" on:click=on_close/>
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
                                                          <Icon icon=Icon::from(IoIcon::IoTrash) width="20px" height="24px"/>
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
