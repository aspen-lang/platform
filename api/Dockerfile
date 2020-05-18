FROM alpine as builder

RUN apk update && apk add libgcc gcc musl musl-dev libstdc++

RUN wget -O - https://sh.rustup.rs | sh -s - --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/src/api

RUN mkdir src
RUN echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

RUN rm target/release/deps/api*

COPY . .
RUN cargo build --release

FROM alpine

ENV PORT 80
ENV ROCKET_PORT $PORT

COPY --from=builder /usr/src/api/target/release/api /api

CMD ["/api"]
