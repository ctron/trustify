FROM localhost/hermetic-rust:latest AS builder

RUN mkdir /src

COPY . /src

WORKDIR /src

RUN echo >> .cargo/config.toml # ensure we have a newline
RUN cat vendor.toml >> .cargo/config.toml

RUN env SWAGGER_UI_DOWNLOAD_URL=file:///src/vendor/swagger-ui.zip cargo build --release --no-default-features

FROM registry.access.redhat.com/ubi10/ubi:latest

COPY --from=builder /src/target/release/trustd /usr/local/bin/

CMD /usr/local/bin/trustd
