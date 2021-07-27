FROM rust:1.53.0 AS server-builder
WORKDIR /usr/src/
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y clang musl-tools

RUN USER=root cargo new uncomment
WORKDIR /usr/src/uncomment
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM node:16.5.0 AS client-builder
WORKDIR /usr/src/uncomment
COPY package.json ./
RUN npm install
COPY webpack.config.js tsconfig.json ./
COPY client ./client
RUN npm run build

FROM alpine:3
WORKDIR /app
COPY --from=server-builder /usr/local/cargo/bin/uncomment .
COPY --from=client-builder /usr/src/uncomment/dist dist
EXPOSE 8080
VOLUME /db
ENV UNCOMMENT_DATABASE=sqlite:/db/data.db
ENV UNCOMMENT_LISTEN=0.0.0.0:8080
CMD ["./uncomment"]
