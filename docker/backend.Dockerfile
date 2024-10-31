FROM rust:1.70 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/src/app/target/release/icn-backend /usr/local/bin/
EXPOSE 8000
CMD ["icn-backend"]
