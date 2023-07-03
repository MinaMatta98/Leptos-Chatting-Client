use base64::engine::general_purpose;
use base64::Engine;
use leptos::html::{Div, Input};
use leptos::{prelude::*, *};
use leptos_icons::*;
use leptos_router::{use_navigate, ActionForm};
use web_sys::MouseEvent;

use crate::app::pages::components::anciliary::{loading_fallback, Button, ButtonVal, UserInput};
use crate::app::pages::conversation::get_current_id;
use crate::app::{
    pages::{components::avatar::base_64_encode_uri, websocket::HandleWebSocket, UserContext},
    DrawerContext, MessageDrawerContext,
};

use crate::server_function::{
    delete_conversations, get_users, login_status, upload_user_info, CreateGroupConversation,
    UserModel,
};

use super::avatar::{self, *};

#[component]
pub fn Modal(cx: Scope, children: Children, context: RwSignal<bool>) -> impl IntoView {
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
            <Icon icon=Icon::from(IoIcon::IoClose) class="h-6 w-6 text-gray-900"/>
          </button>
        </div>
        {children(cx)}
      </div>
    </div>
    </div>
    }
}

#[component]
pub fn GroupChatModal(cx: Scope, context: RwSignal<bool>) -> impl IntoView {
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
                            <UserInput id="name" label="Group Name" input_type="text" required=true disabled=ButtonVal::RwSignal(disable_signal) placeholder=String::from("Group Name...") _node_ref=name_ref/>
                            <input name="is_group" value="true" class="hidden"/>
                            <Select disabled=disable_signal label="Members" input_ref=input_ref input_signal/>
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
pub fn ImageModal(cx: Scope, context: RwSignal<bool>, src: String) -> impl IntoView {
    view! {cx,
        <Modal context>
            <div class="max-w-[80%] max-h-[80%]">
                <img alt="image" class="object-cover" src=src/>
            </div>
        </Modal>
    }
}

#[component]
pub fn ConfirmModal(cx: Scope) -> impl IntoView {
    let drawer_context = move || use_context::<DrawerContext>(cx).unwrap().status;
    let on_click = move |_| {
        spawn_local(async move {
            delete_conversations(cx, get_current_id(cx)())
                .await
                .unwrap();
            drawer_context().set(false);
            use_context::<MessageDrawerContext>(cx)
                .unwrap()
                .status
                .update(|val| *val = !*val);
            queue_microtask(move || {
                let _ = use_navigate(cx)("/conversations", Default::default());
            })
        })
    };

    view! {cx,
        <Modal context=drawer_context()>
            <div class="sm:flex sm:items-start">
                <div class="mx-auto flex h-12 w-12 flex-shrink-0 items-center rounded-full justify-center bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                    <Icon icon=Icon::from(FiIcon::FiAlertTriangle) class="h-6 w-6 text-red-600"/>
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
pub fn SettingsModal(cx: Scope, settings_modal_setter: RwSignal<bool>) -> impl IntoView {
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
                    let file = Box::new(gloo_file::futures::read_as_bytes(file).await.unwrap());
                    upload_user_info(cx, Some(*file.clone()), first_name_val, last_name_val)
                        .await
                        .unwrap();

                    let base64_encoded_image =
                        general_purpose::STANDARD_NO_PAD.encode(*file.clone());
                    let data_uri = base_64_encode_uri(base64_encoded_image);
                    let id = if let 0 = use_context::<UserContext>(cx).unwrap().id.get_untracked() {
                        login_status(cx).await.unwrap().id
                    } else {
                        use_context::<UserContext>(cx).unwrap().id.get_untracked()
                    };

                    HandleWebSocket::handle_sink_stream(
                        avatar::IconData {
                            user_id: id,
                            data: data_uri,
                        },
                        id,
                    )
                    .await;
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

    let status = create_resource(cx, || (), move |_| async move { login_status(cx).await });
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
                        {move || status.read(cx).map(|user| {
                            let user = user.unwrap();
                            view!{cx,
                                <div class="mt-10 flex flex-col gap-y-8">
                                    <UserInput id="first_name" _node_ref=first_name_ref input_type="text" label="First Name" required=false disabled=ButtonVal::RwSignal(disable_signal) placeholder=user.first_name/>
                                </div>
                                <div class="mt-10 flex flex-col gap-y-8">
                                    <UserInput id="last_name" _node_ref=last_name_ref input_type="text" label="Last Name" required=false disabled=ButtonVal::RwSignal(disable_signal) placeholder=user.last_name/>
                                </div>
                                <div class="mt-10 flex flex-col gap-y-3">
                                    <label class="block text-sm font-medium leading-6 text-gray-900">
                                        "Photo"
                                    </label>
                                    <Avatar id=user.id/>
                                        <div class="flex gap-x-3">
                                              <Button on_click=move |_| image_ref.get().unwrap().click() button_type="button" disabled=ButtonVal::Bool(false) color="bg-sky-500 hover:bg-sky-600 focus-visible:outline-sky-600">
                                                "Upload"
                                              </Button>
                                            <input _ref=image_ref type="file" class="hidden" id="upload" name="upload" on:change=move |_| button_signal.set(false)>
                                            </input>
                                        </div>
                                </div>
                            }})}
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
fn Select(
    cx: Scope,
    disabled: RwSignal<bool>,
    label: &'static str,
    input_ref: NodeRef<Input>,
    input_signal: RwSignal<Vec<(HtmlElement<Div>, i32)>>,
) -> impl IntoView {
    let hidden_state = create_rw_signal(cx, true);

    let users = create_local_resource(
        cx,
        move || hidden_state.get(),
        move |_| async move { get_users(cx).await },
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
                node_ref=input_ref
                on:click=move |_| hidden_state.update(|val| *val = !*val )/>
                     <For
                       each=move || input_signal.get()
                       key=|input| input.1
                       view=move |cx, item: (HtmlElement<Div>, i32)| {
                         view! {
                           cx,
                             {item.0}
                        }
                     }/>
              <ul class=move || format!("absolute z-10 mt-1 w-full bg-white border border-gray-300 rounded-md shadow-lg {}", if hidden_state.get() {"hidden"} else {"block"})>
                <Suspense fallback=loading_fallback(cx)>
                     {move || users.read(cx).map(|options|
                         view!{cx,
                             <For
                               each=move || options.clone().unwrap()
                               key=|user| user.id
                               view=move |cx, item: UserModel| {
                                      let li_ref = create_node_ref::<html::Li>(cx);
                                      let input = input_ref.get().unwrap();
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
                                                                <Icon icon=Icon::from(IoIcon::IoClose) class="h-3 w-3" on:click=move |_| {
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
