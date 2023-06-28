use std::collections::HashMap;

use fancy_regex::Regex;
use leptos::{
    html::{Div, Input},
    *,
};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use web_sys::SubmitEvent;
mod callback;
mod form_items;
pub mod pages;
mod validation;

use crate::{
    app::{
        callback::{Callback, SignupSetters},
        form_items::{Banner, InputProps, InputValidation, ValidationGetter},
        pages::{ConversationId, Conversations, EmptyState, Users},
    },
    server_function::UserLogin,
};

use self::pages::components::avatar::UserIcon;

#[derive(Debug, Clone)]
pub struct EmailContext {
    pub email: ReadSignal<String>,
    pub email_setter: WriteSignal<String>,
}

#[derive(Debug, Clone)]
pub struct DrawerContext {
    pub status: RwSignal<bool>,
}

#[derive(Debug, Clone)]
pub struct MessageDrawerContext {
    pub status: RwSignal<bool>,
}

#[derive(Debug, Clone)]
pub struct SideBarContext {
    pub status: RwSignal<bool>,
}

#[derive(Debug, Clone)]
pub struct SeenContext {
    pub status: RwSignal<Vec<SeenContextInner>>,
}

#[derive(Debug, Clone)]
pub struct SeenContextInner {
    pub conversation_id: i32,
    pub last_message_id: i32,
}

#[derive(Debug, Clone)]
pub struct IsOpen {
    pub status: RwSignal<bool>,
}

#[derive(Debug, Clone)]
pub struct SignupContext {
    status: ReadSignal<bool>,
    status_setter: WriteSignal<bool>,
}

#[derive(Debug, Clone)]
pub struct IconVec {
    icons: RwSignal<HashMap<i32, UserIcon>>
}

#[derive(Serialize, Deserialize, Validate, Clone, Debug, PartialEq)]
pub struct SignupSchema {
    #[serde(rename = "firstName")]
    pub first_name: NameSchema,
    #[serde(rename = "lastName")]
    pub last_name: NameSchema,
    pub email: EmailSchema,
    pub password: PasswordSchema,
    #[serde(rename = "phoneNumber")]
    pub phone_number: PhoneSchema,
}

#[derive(Serialize, Deserialize, Validate, Clone, Debug, PartialEq)]
pub struct NameSchema {
    #[validate(length(min = 4))]
    #[validate(custom = "NameSchema::validate_name")]
    pub entry: String,
}

impl NameSchema {
    fn validate_name(name: &str) -> Result<(), ValidationError> {
        if Regex::new(r"^[^\s\p{P}]+$")
            .unwrap()
            .is_match(name)
            .unwrap()
        {
            Ok(())
        } else {
            Err(ValidationError::new("Name does not meet the requirements"))
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Clone, Debug, PartialEq)]
pub struct EmailSchema {
    #[validate(email)]
    pub entry: String,
}

#[derive(Clone)]
pub struct FormItemSchema<'a> {
    field: &'a str,
    input_type: &'a str,
    name: &'a str,
    reference: NodeRef<Input>,
    validator: WriteSignal<leptos::HtmlElement<Div>>,
    validation_getter: ReadSignal<leptos::HtmlElement<Div>>,
    display_case: Vec<AppState>,
}

#[derive(Clone)]
pub struct BannerSchema<'a> {
    banner_string: &'a str,
    display_case: AppState,
    href: &'a str,
}

#[derive(Serialize, Deserialize, Validate, Clone, Debug, PartialEq)]
pub struct PasswordSchema {
    #[validate(custom = "PasswordSchema::validate_password")]
    pub entry: String,
}

impl PasswordSchema {
    fn validate_password(password: &str) -> Result<(), ValidationError> {
        let regex_pattern = Regex::new(r"^(?=.*[A-Z])(?=.*[0-9])(?=.*[$!@*]).{8,}$").unwrap();
        if regex_pattern.is_match(password).unwrap() {
            Ok(())
        } else {
            Err(ValidationError::new(
                "Password does not meet the requirements",
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Clone, Debug, PartialEq)]
pub struct PhoneSchema {
    #[validate(phone)]
    pub entry: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FormValidation {
    Success { random_string: Option<String> },
    Error,
    EmailPresent,
    PhonePresent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VerificationValidation {
    Success,
    IncorrectValidationCode,
    EmailNotPresent,
    ServerError,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum VerifyPassword {
    Success(UserLogin),
    IncorrectCredentials,
    ServerError,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AppState {
    Login,
    Signup,
    Validate,
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let (email_context, email_context_setter) = create_signal(cx, String::from(""));
    let (signup_context, signup_context_setter) = create_signal(cx, false);

    provide_context(
        cx,
        EmailContext {
            email: email_context,
            email_setter: email_context_setter,
        },
    );

    provide_context(
        cx,
        SignupContext {
            status: signup_context,
            status_setter: signup_context_setter,
        },
    );

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/zing.css"/>

        // sets the document title
        <Title text="ZING!"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=|cx| view! { cx, <HomePage toggle=AppState::Signup/> } ssr = SsrMode::Async/>
                    <Route path="/login" view=|cx| view! { cx, <HomePage toggle=AppState::Login/> } ssr = SsrMode::Async/>
                    <Route path="/validate" view=|cx| view! { cx, <HomePage toggle=AppState::Validate/> } ssr = SsrMode::Async/>
                    <Route path="/user" view=|cx| view! { cx, <Users /> } ssr = SsrMode::Async/>
                    <Route path="/conversations" view=|cx| view! { cx, <Conversations/>  } ssr = SsrMode::Async>
                        <Route path=":id" view=|cx| view! { cx, <ConversationId/> } ssr = SsrMode::Async/>
                        <Route path="/" view=|cx| view! { cx,
                                                     <div>
                                                        <div class=move || format!("lg:pl-80 h-screen
                                                             lg:block {}", if use_context::<IsOpen>(cx)
                                                                .unwrap().status.get() {"block"} else {"hidden"})>
                                                            <EmptyState />
                                                        </div>
                                                     </div>
                        } ssr = SsrMode::Async/>
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope, toggle: AppState) -> impl IntoView {
    // Creates a reactive value to update the button

    create_effect(cx, move |_| {
        if toggle == AppState::Validate
            && use_context::<crate::app::EmailContext>(cx)
                .unwrap()
                .email
                .get()
                == ""
        {
            queue_microtask(move || use_navigate(cx)("/login", Default::default()).unwrap());
        }
    });

    view! { cx,
    <div class="flex relative justify-center bg-amber-600" >
        <h1 class="text-8xl absolute top-20 xl:top-16">"ZING!"</h1>
    </div>
    <div class="flex w-screen h-screen bg-[url('/Strike.svg')]
            relative bg-no-repeat bg-cover bg-center bg-amber-600 
            flex relative items-center justify-center">
        <div class="flex w-100 h-100">
            <Form toggle/>
        </div>
    </div>
    }
}

#[component]
fn FormItem(
    cx: Scope,
    field: &'static str,
    input_type: &'static str,
    name: &'static str,
    _reference: NodeRef<Input>,
    validator: WriteSignal<leptos::HtmlElement<Div>>,
    toggle: AppState,
) -> impl IntoView {
    let (background_color, background_color_setter) = create_signal(cx, "border-amber-300");

    let (label_background_color, label_background_color_setter) = create_signal(cx, "bg-amber-300");

    view! {cx,
        <div class="flex w-100 sm:text-xs md:text-xs py-0.5">
        <label for=field.clone() class=move || format!("flex basis-3/6 sm:basis-3/6 md:basis-2/6 rounded-s-lg py-1 px-1 justify-center {} sm:py-1 ml-3 lg:ml-7 lg:text-sm md:text-xs", label_background_color())>
                {field.clone().to_string() + " :"}</label>
            <div dir="rtl" class=move || format!("flex rounded-s-lg border-2 {} justify-center basis:-3/6 md:basis-4/6 sm:basis-3/6 mr-5 sm:py-0 sm:px-0", background_color())>
            <input _ref=_reference name=name dir="ltr" type=input_type placeholder=field.clone().to_string() + "..." id=field
                class="w-100 decoration-2 focus:border-0 focus:outline-none decoration-amber-600 xs:py-0 xs:px-0 mx-5" style="width: 100%"
                on:input=Callback::on_keypress_callback(cx, background_color_setter, validator, label_background_color_setter, toggle)
                value=move ||
                {
                    let email = use_context::<EmailContext>(cx).unwrap().email.get();
                    if ! email.is_empty() && name == "email"
                    {
                        Some(email)
                    }
                    else {
                        None
                }}></input>
        </div>
        </div>
    }
}

#[component]
fn Form(cx: Scope, toggle: AppState) -> impl IntoView {
    let first_name = create_node_ref::<Input>(cx);
    let last_name = create_node_ref::<Input>(cx);
    let email = create_node_ref::<Input>(cx);
    let verification = create_node_ref::<Input>(cx);
    let password = create_node_ref::<Input>(cx);
    let phone_number = create_node_ref::<Input>(cx);
    let (loading_indicator, loading_indicator_setter) = create_signal(cx, String::from("hidden"));
    let (database_connection_result, database_connection_result_setter) =
        create_signal(cx, String::from("Establishing Database Connection..."));

    let (success_signal, success_signal_value) =
        create_signal(cx, view! {cx, <div class="hidden"></div>});

    let hidden_div = view! {cx, <div class="hidden"></div>};
    let (first_name_validation, first_name_validation_setter) =
        create_signal(cx, hidden_div.clone());
    let (last_name_validation, last_name_validation_setter) = create_signal(cx, hidden_div.clone());
    let (email_validation, email_validation_setter) = create_signal(cx, hidden_div.clone());
    let (verification_validation, verification_validation_setter) =
        create_signal(cx, hidden_div.clone());
    let (password_validation, password_validation_setter) = create_signal(cx, hidden_div.clone());
    let (phone_validation, phone_validation_setter) = create_signal(cx, hidden_div);
    let on_submit = |cx: Scope, toggle: AppState| match toggle {
        AppState::Signup => Callback::on_submit_callback(
            cx,
            loading_indicator_setter,
            SignupSetters {
                first_name,
                last_name,
                email,
                password,
                phone_number,
            },
            success_signal_value,
            database_connection_result_setter,
        ),
        AppState::Login => Callback::on_login_callback(cx, success_signal_value, email, password),
        AppState::Validate => Callback::on_validate_callback(
            cx,
            verification,
            loading_indicator_setter,
            database_connection_result_setter,
            success_signal_value,
        ),
    };

    let vec = form_items::InputItems::init(
        InputProps {
            first_name,
            last_name,
            email,
            password,
            phone_number,
            verification,
        },
        InputValidation {
            first_name_validation_setter,
            last_name_validation_setter,
            email_validation_setter,
            password_validation_setter,
            phone_validation_setter,
            verification_validation_setter,
        },
        ValidationGetter {
            first_name_validation,
            last_name_validation,
            email_validation,
            password_validation,
            phone_validation,
            verification_validation,
        },
    );

    view! {cx,
            <FormItemDecorator toggle callback=on_submit(cx, toggle)>
                <RenderBanner toggle/>
                <EmailValidation toggle/>
                <LoadingStatus database_connection_result
                success_signal loading_indicator/>
                <FormInputItems toggle items=vec/>
            </FormItemDecorator>
    }
}

#[component]
fn RenderBanner(cx: Scope, toggle: AppState) -> impl IntoView {
    view! {cx,
           <div class="w-100 grid grid-cols-1 grid-rows-1
           place-content-between items-center justify-center
           bg-gradient-to-r from-amber-400 via-amber-500
           to-amber-400 rounded-xl h-10 mx-5">
               {
                   let vec = Banner::init();
                    view! {
                         cx,
                           <For each=move || vec.clone()
                             key=|vec| vec.href view=move |cx, item: BannerSchema| {
                               view! { cx,
                                   <Show when=move || item.display_case == toggle fallback = |_| ()>
                                         <A class="mx-5 text-center h-100 text-white rounded-lg py-1 xs:py-0 xs:px-0" href=item.href>
                                             {item.banner_string}
                                         </A>
                                   </Show>
                                    }}
                            />
                    }}
            </div>
    }
}

#[component]
fn FormItemDecorator<F>(
    cx: Scope,
    toggle: AppState,
    children: Children,
    callback: F,
) -> impl IntoView
where
    F: Fn(SubmitEvent) + 'static,
{
    view! {cx,
               <div class="card hover:ring-2 hover:ring-gray-900 z-50">
                  <div class="card absolute -translate-x-4 translate-y-4
                    justify-center items-center hover:ring-2 hover:ring-amber-500
                    active:ring-none delay-150 transition z-20">
                        <form action=move || if toggle == AppState::Signup {"/signup"}
                        else if toggle == AppState::Login {"/login"} else {"/valid"}
                        method="post"
                        class="m-auto w-100 grid grid-cols-1 grid-rows-7
                            col-span-full row-span-full xl:space-y-6 lg:space-y-5 text-xs
                            md:space-y-3 md:text-md sm:space-y-2 sm:text-sm xs:text-xs
                            2xl:space-y-8 2xl:text-md xl:text-md"
                        style="width: 100%"
                        on:submit=callback
                  >
                    {children(cx)}

                                <button type="submit" class="bg-amber-500 h-10 rounded-xl px-3 text-white
                                    hover:bg-amber-400 hover:-translate-y-1 transition mx-5 animate-pulse">
                                        { match toggle {
                                             AppState::Login => "Login",
                                             AppState::Signup => "Welcome",
                                             AppState::Validate => "Validate",
                                        }}
                                </button>
                        </form>
                   </div>
            </div>
    }
}

#[component]
fn EmailValidation(cx: Scope, toggle: AppState) -> impl IntoView {
    view! {cx,
            <Show when=move || toggle == AppState::Validate fallback = |_| ()>
                <div class=move || format!("flex flex-col items-center px-4 py-2
                font-semibold mx-6 text-sm shadow rounded-md text-white bg-slate-600
                hover:bg-slate-400
                transition ease-in-out duration-150 cursor-not-allowed")>
                    <p>
                    {move || format!("An email has been sent to {} for verification.", use_context::<EmailContext>(cx).unwrap().email.get())}
                    </p>
                    <p>
                         "Input the verification code in the section below:"
                    </p>
                </div>
            </Show>
            { move || use_context::<SignupContext>(cx).unwrap().status.get().then(||
                        view! {cx,
                            <Show when=move || toggle == AppState::Login fallback=|_|()>
                                <div class=move || format!("inline-flex items-center justify-center
                                    px-4 py-2 font-semibold mx-6 leading-6 text-sm shadow rounded-md
                                    text-white bg-teal-500 hover:bg-teal-300 transition ease-in-out
                                duration-150 cursor-not-allowed")>
                                <p>"Successful Signup"</p>
                                </div>
                            </Show>
                        })
            }
    }
}

#[component]
fn LoadingStatus(
    cx: Scope,
    database_connection_result: ReadSignal<String>,
    success_signal: ReadSignal<HtmlElement<Div>>,
    loading_indicator: ReadSignal<String>,
) -> impl IntoView {
    view! {
    cx,

        <div class=move || format!("inline-flex items-center
                   px-4 py-2 font-semibold mx-6 leading-6
                   text-sm shadow rounded-md text-white bg-amber-500
                   hover:bg-amber-300
                   transition ease-in-out duration-150 cursor-not-allowed {}",
                   loading_indicator.get())>
                        <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        {database_connection_result}
        </div>
                        {success_signal}
    }
}

#[component]
fn FormInputItems(
    cx: Scope,
    toggle: AppState,
    items: Vec<FormItemSchema<'static>>,
) -> impl IntoView {
    view! {
      cx,
        <For
          each=move || items.clone()
          key=|vec| vec.name
          view=move |cx, item: FormItemSchema| {
            view! {
              cx,
                <Show when=move || item.display_case.contains(&toggle) fallback = |_| ()>
                  <div class="flex flex-col">
                      <FormItem field=item.field input_type=item.input_type name=item.name _reference=item.reference validator=item.validator toggle/>
                        <Show when=move || matches!(toggle, AppState::Signup | AppState::Validate) fallback = |_| ()>
                              {move || item.validation_getter}
                        </Show>
                  </div>
                </Show>
            }
          }
        />
    }
}
