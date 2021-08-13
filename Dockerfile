# mostly based on https://github.com/zupzup/rust-docker-web/blob/main/debian/Dockerfile

FROM rust:latest as eth-proxy-builder

RUN USER=root cargo new --bin eth-proxy
WORKDIR ./eth-proxy
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs target/release/deps/eth_proxy*
ADD . ./
RUN cargo build --release

FROM debian:buster-slim

ARG APP=/usr/lib/app

EXPOSE 8080

ENV APP_USER=appuser \
    HOST=0.0.0.0 \
    PORT=8080 \
    ETHERSCAN_DOMAIN=api.etherscan.io

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=eth-proxy-builder /eth-proxy/target/release/eth-proxy ${APP}/eth-proxy

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./eth-proxy"]