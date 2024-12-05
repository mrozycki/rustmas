FROM rust:alpine3.20 AS build-backend
WORKDIR /build_rustmas
RUN apk add musl-dev && apk add libressl-dev && rustup target add wasm32-wasip2
COPY . .
RUN set -e; \
	cargo install --path animation-wrapper; \
	cargo build --bin rustmas-webapi --release --no-default-features; \
	mkdir -p target/animations; \
	for plugin in `find animations -type d -maxdepth 1`; do \
		cd ${plugin}; \
		crabwrap && mv *.crab ../../target/animations; \
		cd -; \
	done
	
FROM rust:alpine3.20 AS rustmas-backend
WORKDIR /rustmas
RUN mkdir -p ./target/release && mkdir -p ./target/animations
COPY --from=build-backend /build_rustmas/target/release/rustmas-webapi ./target/release
COPY --from=build-backend /build_rustmas/target/animations/* ./target/animations
ENTRYPOINT ["/rustmas/target/release/rustmas-webapi"]
