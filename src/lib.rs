#![feature(let_chains)]
#![feature(async_closure)]
#![feature(stmt_expr_attributes)]
#![feature(impl_trait_in_assoc_type)]

use cfg_if::cfg_if;
mod app;
pub mod server_function;
pub mod entities;
pub mod database;
pub mod migrator;
pub mod emailing;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use app::*;
      use leptos::*;

      console_error_panic_hook::set_once();

      leptos::mount_to_body(move |cx| {
          view! { cx, <App/> }
      });
    }
}
}
