# Use the official Rust nightly image as the base image
FROM rustlang/rust:nightly

# Set the working directory inside the container
WORKDIR /app

VOLUME ./storage/
# Copy the project files into the container
COPY . .

# Install system dependencies
RUN apt update 

RUN apt-get install -y \
	sudo\
	lsb-release\
	cmake\
	nasm\
	expect\
	wget\
	curl\
	gpg\
	gnupg\
    pkg-config\
    libssl-dev\
    redis\
	mariadb-server

# Alloc. Env. Vars
ENV MYSQL_USERNAME 'jenkins'
ENV MYSQL_PASSWORD 'hilton'
ENV DATABASE_URL "mysql://$MYSQL_USERNAME:$MYSQL_PASSWORD@localhost:3306"
ENV DB_NAME 'ZING'

# Build the project via Cargo Leptos
RUN service mariadb start
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos
# RUN cargo +nightly build --release
RUN cargo leptos build --release

# Expose the necessary ports <=== Documentation Only. Run with -p 8000:8000
EXPOSE 8000

# Run the project
CMD ["cargo", "leptos", "watch", "--release"]
