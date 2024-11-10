FROM rust:alpine3.20 AS build-frontend
WORKDIR /build_rustmas
RUN apk add musl-dev && apk add libressl-dev && apk add g++ \
	&& rustup target add wasm32-unknown-unknown \
	&& cargo install trunk --version 0.21.4 \
	&& cargo install wasm-bindgen-cli --version 0.2.93 \
    && cargo install wasm-opt
COPY . .
RUN cd webui && trunk build --features visualizer,local

FROM nginx:mainline-alpine3.20 AS rustmas-frontend
WORKDIR /usr/share/nginx/html
RUN rm 50x.html
COPY --from=build-frontend /build_rustmas/webui/dist* .
