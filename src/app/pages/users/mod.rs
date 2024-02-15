use leptos::*;
use leptos_router::use_navigate;

use crate::{
    app::pages::{
        components::anciliary::{loading_fallback, EmptyState, Sidebar, UserContexts},
        Avatar, ICONVEC, SINKVEC, STREAMVEC,
    },
    server_function::{
        routes::associated_conversation, routes::conversation_action, routes::get_users, UserModel,
    },
};

#[component]
pub fn Users(cx: Scope) -> impl IntoView {
    UserContexts::init_users(cx);
    leptos::on_cleanup(cx, || {
        ICONVEC.write().clear();
        STREAMVEC.write().clear();
        SINKVEC::send_clear();
    });
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
    let users_arr = create_local_resource(cx, || (), move |_| async move { get_users(cx).await });
    let on_click = move |id: i32, cx: Scope| {
        spawn_local(async move {
            conversation_action(cx, vec![id], false, None)
                .await
                .unwrap();
            let conversation_id = associated_conversation(cx, id).await.unwrap();
            queue_microtask(move || {
                use_navigate(cx)(
                    &format!("/conversations/{conversation_id}"),
                    Default::default(),
                )
                .unwrap()
            });
        });
    };

    view! {cx,
        <Suspense fallback=loading_fallback(cx)>
            {move || users_arr.read(cx).map(|items| {
                let items = items.unwrap();
                view!{cx,
                        <For
                          each=move || items.clone()
                          key=|items| items.id
                          view=move |cx, item: UserModel| {
                                        view!{cx,
                                             <div class="w-full relative flex
                                                 items-center space-x-3 bg-white
                                                 p-3 hover:bg-neutral-100 rounded-lg
                                                 transition cursor-pointer"
                                                 on:click=move |_| on_click(item.id, cx)>
                                                     <Avatar id=item.id/>
                                                     <div class="min-w-0 flex-1">
                                                         <div class="focus:outline-none">
                                                             <div class="flex justify-between items-center mb-1">
                                                                 <p class="text-sm font-medium text-gray-900">
                                                                     {format!("{} {}", item.first_name, item.last_name)}
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
