use super::{AppState, BannerSchema, FormItemSchema};
use leptos::{
    html::{Div, Input},
    *,
};

pub struct InputProps {
    pub first_name: NodeRef<Input>,
    pub last_name: NodeRef<Input>,
    pub email: NodeRef<Input>,
    pub password: NodeRef<Input>,
    pub phone_number: NodeRef<Input>,
    pub verification: NodeRef<Input>,
}

pub struct InputValidation {
    pub first_name_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
    pub last_name_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
    pub email_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
    pub password_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
    pub phone_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
    pub verification_validation_setter: WriteSignal<leptos::HtmlElement<Div>>,
}

pub struct ValidationGetter {
    pub first_name_validation: ReadSignal<leptos::HtmlElement<Div>>,
    pub last_name_validation: ReadSignal<leptos::HtmlElement<Div>>,
    pub email_validation: ReadSignal<leptos::HtmlElement<Div>>,
    pub password_validation: ReadSignal<leptos::HtmlElement<Div>>,
    pub phone_validation: ReadSignal<leptos::HtmlElement<Div>>,
    pub verification_validation: ReadSignal<leptos::HtmlElement<Div>>,
}

pub struct InputItems;
impl<'a> InputItems {
    pub fn init(
        props: InputProps,
        validation_props: InputValidation,
        getter_props: ValidationGetter,
    ) -> Vec<FormItemSchema<'a>> {
        let first_name_schema = FormItemSchema {
            field: "First Name",
            input_type: "text",
            name: "firstName",
            reference: props.first_name,
            validator: validation_props.first_name_validation_setter,
            validation_getter: getter_props.first_name_validation,
            display_case: vec![AppState::Signup],
        };

        let last_name_schema = FormItemSchema {
            field: "Last Name",
            input_type: "text",
            name: "firstName",
            reference: props.last_name,
            validator: validation_props.last_name_validation_setter,
            validation_getter: getter_props.last_name_validation,
            display_case: vec![AppState::Signup],
        };

        let email_schema = FormItemSchema {
            field: "Email",
            input_type: "email",
            name: "email",
            reference: props.email,
            validator: validation_props.email_validation_setter,
            validation_getter: getter_props.email_validation,
            display_case: vec![AppState::Login, AppState::Signup],
        };

        let password_schema = FormItemSchema {
            field: "Password",
            input_type: "password",
            name: "password",
            reference: props.password,
            validator: validation_props.password_validation_setter,
            validation_getter: getter_props.password_validation,
            display_case: vec![AppState::Login, AppState::Signup],
        };

        let verification_code_schema = FormItemSchema {
            field: "Verification Code",
            input_type: "text",
            name: "verificationCode",
            reference: props.verification,
            validator: validation_props.verification_validation_setter,
            validation_getter: getter_props.verification_validation,
            display_case: vec![AppState::Validate],
        };

        let phone_number_schema = FormItemSchema {
            field: "Phone Number",
            input_type: "tel",
            name: "phoneNumber",
            reference: props.phone_number,
            validator: validation_props.phone_validation_setter,
            validation_getter: getter_props.phone_validation,
            display_case: vec![AppState::Signup],
        };
        vec![
            first_name_schema,
            last_name_schema,
            email_schema,
            password_schema,
            verification_code_schema,
            phone_number_schema,
        ]
    }
}

pub struct Banner;

impl Banner {
    pub fn init<'a>() -> Vec<BannerSchema<'a>> {
        let login_banner = BannerSchema {
            banner_string: "Not a member? Signup Instead",
            display_case: AppState::Login,
            href: "/",
        };

        let signup_banner = BannerSchema {
            banner_string: "Already a member? Log In Instead",
            display_case: AppState::Signup,
            href: "/login",
        };

        let verification_banner = BannerSchema {
            banner_string: "Verify Email",
            display_case: AppState::Validate,
            href: "/validate",
        };

        vec![login_banner, signup_banner, verification_banner]
    }
}
