# WIP Leptos-Chatting-Client
This repository demonstrates the use of the following technologies:
* [Actix-web](https://actix.rs/): Actix based backend for request management.
* [Leptos-RS](https://github.com/leptos-rs/leptos): Server Side Rendering and Hydration framework utilizing web-assembly.
* [Cargo-Leptos](https://github.com/leptos-rs/cargo-leptos): Project Building Managed via Cargo Leptos.
* [Tokio-RS](https://tokio.rs/): Asynchronous Runtime
* [Wasm-Bindgen](https://github.com/rustwasm/wasm-bindgen): JSCast Bindings via web-assembly.
* [Sea-Orm](https://github.com/SeaQL/sea-orm): Asynchronous Object Relational Mapping (ORM) used for the management of MySQL Databases.
* [Tailwind-Css](https://tailwindcss.com/): Styles on the go.
* [Redis](https://redis.io/): User Session Management via Redis value stores.

## Features
* User Authentication and Verification
* Database Management w/ CRUD and SQL Join Statements. User Password hashing achieved via argon2.
* Asynchronous api calls
* React-like, fine grained reactive environment.
* Tailwind CSS compilation


## Preview
### Conversations
![Converesations](./Demo/conversations.png)
![Converesation-Info](./Demo/conversations-info.png)
![Converesation-Deletion](./Demo/conversations-deletion.png)
### Group Chatting
![Creation](./Demo/group-chat-1.png)
![Multi User Conversation](./Demo/group-chat-2.png)
### Group Chatting
![Graceful-Suspension](./Demo/Graceful-suspension.png)

## Building
To build this repository within a container, where cargo and mariadb are not installed, simply run the following command within the root directory of this project in an environment where docker is installed:

```bash
docker build -t zing .
```

Note that this building process involves compiling the release version of the project (heavily optimized) and **will** take upwards of 15-20 minutes to compile. With a ryzen 7950x3D (16 core, 32 thread CPU), this compiles in approximately 5-7 minutes.

## Running
To run this project after compilation, run the following command:

```bash
docker run -p 8000:8000 zing
```
