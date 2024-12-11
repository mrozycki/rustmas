FROM rust:alpine3.20 AS build-backend
WORKDIR /build_rustmas
RUN apk add musl-dev && apk add libressl-dev
COPY . .
RUN cargo build --release -p animations \
  && cargo build --bin rustmas-webapi --release --no-default-features \
  && mkdir -p ./artifacts \
  && for plugin in ./animations/src/bin/*.rs; do plugin=${plugin%*.rs}; plugin=${plugin##*/}; cp ./target/release/${plugin} ./artifacts/; done \
  && cp ./target/release/rustmas-webapi ./artifacts/

FROM rust:alpine3.20 AS rustmas-backend
WORKDIR /rustmas
RUN mkdir -p ./target/release && mkdir -p ./plugins
COPY --from=build-backend /build_rustmas/artifacts/* ./target/release/
COPY --from=build-backend /build_rustmas/plugins ./plugins/
ENTRYPOINT ["/rustmas/target/release/rustmas-webapi"]
