use leptos::{*, html::{Input, Div}};
use leptos_router::{use_navigate, NavigateOptions};
use wasm_bindgen::JsCast;
use web_sys::{HtmlFormElement, HtmlInputElement, Event};
use leptos_icons::*;

use crate::{app::{NameSchema, SignupSchema,
    EmailSchema, PhoneSchema, EmailContext,
    FormValidation, validation::ValidationSchema,
    PasswordSchema, VerifyPassword},
    server_function::{ConfirmSubscription,
    self, Login, sign_up}};

use super::{VerificationValidation, SignupContext, AppState};

pub struct Callback;

pub struct SignupSetters {
    pub first_name: NodeRef<Input>,
    pub last_name: NodeRef<Input>,
    pub email: NodeRef<Input>,
    pub password: NodeRef<Input>,
    pub phone_number: NodeRef<Input>,
}

impl Callback {
    pub fn on_submit_callback(
        cx: Scope, loading_indicator_setter: WriteSignal<String>,
        info: SignupSetters, success_signal_value: WriteSignal<leptos::HtmlElement<Div>>,
        database_connection_result_setter: WriteSignal<String>) -> Box<dyn std::ops::Fn(web_sys::SubmitEvent)> 
    {
        
    Box::new(move |event: web_sys::SubmitEvent| {
        event.stop_propagation();
        event.prevent_default();

        use_context::<EmailContext>(cx).unwrap().email_setter.set(info.email.get().unwrap().value());
        log!("{:?}", event.target().unwrap().dyn_into::<HtmlFormElement>().unwrap().action().contains("signup"));
        let sign_up_schema = SignupSchema {
            first_name: NameSchema {
                entry: info.first_name.get().unwrap().value(),
            },
            last_name: NameSchema {
                entry: info.last_name.get().unwrap().value(),
            },
            email: EmailSchema {
                entry: info.email.get().unwrap().value(),
            },
            password: crate::app::PasswordSchema {
                entry: info.password.get().unwrap().value(),
            },
            phone_number: PhoneSchema {
                entry: info.phone_number.get().unwrap().value(),
            },
        };

let signup = create_local_resource(cx, move || sign_up_schema.clone(), move |sign_up_schema| {
    async move {
        sign_up(cx, sign_up_schema).await
    }
});
        // let signup = create_server_action::<crate::server_function::SignUp>(cx);
        // signup.dispatch(crate::server_function::SignUp { form: sign_up_schema } );

                loading_indicator_setter.set(String::from(""));
                success_signal_value.set(view!{cx,
                    <div>
                        <Suspense fallback=move || ()>
                        <div class="flex justify-center items-center">
                        { move || signup.read(cx).map(|val| {
                        view!{cx, <p class="text-center mx-2">
                            { match val {
                                    Ok(FormValidation::Success {random_string}) => { 
                                    database_connection_result_setter.set(String::from("Sending Verification Email..."));
                                    let verification = create_server_action::<crate::server_function::VerifyEmail>(cx);
                                    verification.dispatch(crate::server_function::VerifyEmail { first_name: 
                                        info.first_name.get().unwrap().value(), email: info.email.get().unwrap().value(),
                                        random_string: random_string.unwrap()} );

                                    view!{cx,
                                        <div>
                                            <Suspense fallback = || ()>
                                            {move ||
                                                verification.value().get().map(|val| {
                                                    view!{cx, {
                                                        {
                                                        loading_indicator_setter.set(String::from("hidden"));
                                                        }
                                                    {
                                                        queue_microtask(move || use_navigate(cx)("/validate", NavigateOptions{
                                                        replace: false,
                                                        resolve: false,
                                                        scroll: false,
                                                        state: Default::default()}).unwrap());
                                                    }
                                                        val.unwrap()                                           
                                                    }}
                                            })}
                                            </Suspense>
                                        </div>
                                    }
                                },
                                     Ok(FormValidation::Error) => {
                                    loading_indicator_setter.set(String::from("hidden"));
                                    view!{cx, 
                                         <div class="flex items-center">"Ensure that all input criteria has been met"
                                             <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                                        </div>
                                    }
                                     },
                                     Ok(FormValidation::EmailPresent) => {
                                        loading_indicator_setter.set(String::from("hidden"));
                                        view!{cx, <div class="flex items-center">"Email is already registered on the server"
                                             <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                                            </div>
                                        }
                                    },
                                     Ok(FormValidation::PhonePresent) => {
                                        loading_indicator_setter.set(String::from("hidden"));
                                        view!{cx, <div class="items-center">"Phone number is already in use"
                                             <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                                            </div>
                                        }
                                    },
                                     Err(_) => {
                                        loading_indicator_setter.set(String::from("hidden"));
                                        view!{cx, <div class="items-center">"Server error has occured. Please try again later"
                                             <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                                            </div>
                                        }
                                    }
                            }}
                            </p>
                        }})}
                    </div>
                    </Suspense>
                    </div>
                });
    })
    }

    pub fn on_validate_callback(cx: Scope, verification: NodeRef<Input>,
        loading_indicator_setter: WriteSignal<String>, 
        database_connection_result_setter: WriteSignal<String>, 
        success_signal_value: WriteSignal<leptos::HtmlElement<Div>>)-> Box<dyn Fn(web_sys::SubmitEvent)> {
        
    Box::new(move |event: web_sys::SubmitEvent| {
        event.stop_propagation();
        event.prevent_default();
        let input = verification.get().unwrap().value();
        let email = use_context::<EmailContext>(cx).unwrap().email.get();
        let validation = create_server_action::<ConfirmSubscription>(cx);

        validation.dispatch(ConfirmSubscription{
            email,
            input
        });

        loading_indicator_setter(String::from(""));
        database_connection_result_setter(String::from("Checking for email validation..."));

        success_signal_value(view! {cx,
           <div>
            <Suspense fallback = || ()>
            {move || validation.value().get().map(|val|
            view!{cx, <div class="flex text-center justify-center">
            {
             loading_indicator_setter(String::from("hidden"));
                    match val {
                    Ok(val) => {
                        match val {
                            VerificationValidation::Success => {
                                    use_context::<SignupContext>(cx).unwrap().status_setter.set(true);
                                    let navigate = use_navigate(cx);
                                    queue_microtask(move || navigate("/login", Default::default()).unwrap());
                                    "Successful Signup"
                                },
                            VerificationValidation::IncorrectValidationCode => "Incorrect Validation Code",
                            VerificationValidation::EmailNotPresent => "Email is not present. Signup again",
                            VerificationValidation::ServerError => "Server Error has occured. Try again later",
                        }
                    },
                    Err(_) => {
                        "Server Error has occured. Try again later."
                    }
                }}
                <Icon icon=AiIcon::AiCloseCircleFilled width="16px" height="16px" style="color: red"/>
            </div>
            }
            )}
            </Suspense>
            </div>
        })

    })
    }

    pub fn on_keypress_callback(cx: Scope, background_color_setter: WriteSignal<&str>,
    validator: WriteSignal<leptos::HtmlElement<Div>>, label_background_color_setter: WriteSignal<&str>, toggle: AppState
    ) -> Box<dyn Fn(web_sys::Event)>  {
    Box::new(move |event: Event| {
        let element = event
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let input = element.value();

        fn validate(
            cx: Scope,
            schema: ValidationSchema,
            setter: WriteSignal<leptos::HtmlElement<leptos::html::Div>>,
            color_setter: WriteSignal<&str>,
            label_color_setter: WriteSignal<&str>,
            toggle: AppState
        ) -> bool {
           if toggle != AppState::Login {
            match schema.validate() {
                Ok(_) => {
                    setter.set(view! {cx, <div class="hidden"> </div> });
                    color_setter.set("border-amber-300");
                    label_color_setter.set("bg-amber-300");
                    true
                }
                Err(_) => {
                    setter.set(view! {cx, <div class="flex justify-center items-center">
                        <p class="text-center text-sm mx-1">{schema.return_error()}</p>
                            <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                        </div>
                    });
                    color_setter.set("border-red-500");
                    label_color_setter.set("bg-red-500");
                    false
                }
            }
        } else {
            true
        }
        }

        match element.name().as_str() {
            "firstName" => {
                let entry = ValidationSchema::NameSchema(NameSchema { entry: input });
                validate(
                    cx,
                    entry,
                    validator,
                    background_color_setter,
                    label_background_color_setter,
                    toggle
                );
            }
            "lastName" => {
                let entry = ValidationSchema::NameSchema(NameSchema { entry: input });
                validate(
                    cx,
                    entry,
                    validator,
                    background_color_setter,
                    label_background_color_setter,
                    toggle
                );
            }
            "email" => {
                let email_schema = EmailSchema { entry: input };
                let entry = ValidationSchema::EmailSchema(email_schema.clone());

                spawn_local(async move {
                    match validate(
                        cx,
                        entry,
                        validator,
                        background_color_setter,
                        label_background_color_setter,
                        toggle
                    ) {
                        true => {
                            match server_function::cred_validation(cx, Some(email_schema), None)
                                .await
                                .unwrap()
                            {
                                FormValidation::Success { random_string: _ } => {}
                                _ => {
                                    if toggle != AppState::Login {
                                        background_color_setter.set("border-red-500");
                                        label_background_color_setter.set("bg-red-500");
                                        validator.set(view! {cx,
                                                  <div class="flex justify-center">
                                                        <p class="text-center text-sm mx-1">"That email address is already registered"</p>
                                                        <Icon icon=AiIcon::AiCloseCircleFilled width="16px" height="16px" style="color: red"/>
                                                  </div>
                                        });
                                    }
                                   }
                            }
                        }
                        false => {}
                    }
                });
            }
            "password" => {
                let entry = ValidationSchema::PasswordSchema(PasswordSchema { entry: input });
                validate(
                    cx,
                    entry,
                    validator,
                    background_color_setter,
                    label_background_color_setter,
                    toggle
                );
            }
            "phoneNumber" => {
                let entry = ValidationSchema::PhoneSchema(PhoneSchema {
                    entry: input.clone(),
                });
                let phone_schema = PhoneSchema { entry: input };

                spawn_local(async move {
                    match validate(
                        cx,
                        entry,
                        validator,
                        background_color_setter,
                        label_background_color_setter,
                        toggle
                    ) {
                        true => {
                            match server_function::cred_validation(cx, None, Some(phone_schema))
                                .await
                                .unwrap()
                            {
                                FormValidation::Success { random_string: _ } => {}
                                _ => {
                                    background_color_setter.set("border-red-500");
                                    label_background_color_setter.set("bg-red-500");
                                    validator.set(view! {cx,
                                              <div class="flex justify-center">
                                                  <p class="text-center text-sm mx-1">"That phone number is already registered"</p>
                                                  <Icon icon=AiIcon::AiCloseCircleFilled width="12px" height="12px" style="color: red"/>
                                              </div>
                                              });
                                }
                            }
                        }
                        false => {}
                    }
                });
            }
            _ => {}
        }
    })
        
    }
    pub fn on_login_callback(cx: Scope, success_signal_value: WriteSignal<leptos::HtmlElement<Div>>, email: NodeRef<Input>, password: NodeRef<Input>) -> Box<dyn Fn(web_sys::SubmitEvent)> {  
    Box::new(
        move |event: web_sys::SubmitEvent| {
            event.stop_propagation();
            event.prevent_default();
            let login_status = create_server_action::<Login>(cx);
            login_status.dispatch(Login {
                email: email.get().unwrap().value(),
                password: password.get().unwrap().value(),
            });

            success_signal_value.set(
                view! {cx,
                    <div>
                    <Suspense fallback = || ()>
                        {move || login_status.value().get().map(|val| {
                            view!{cx,
                                {match val {
                                    Ok(val) => match val {
                                            VerifyPassword::Success => {
                                            queue_microtask(move || use_navigate(cx)("/user", Default::default()).unwrap());
                                                view!{cx, <p class="text-center">"Successful Login"</p>}
                                            },
                                            VerifyPassword::IncorrectCredentials => view!{cx, <p class="text-center">"Incorrect Credentials"</p>},
                                            VerifyPassword::ServerError => view!{cx, <p class="text-center">"Server Error. Please Try again later."</p>}
                                        },
                                    Err(_) => view!{cx, <p class="text-center">"Server Error. Please Try again later."</p>}
                                    }
                                }
                            }})}
                    </Suspense>
                    </div>
                })
            })
        }
}
