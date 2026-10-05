FROM rust:1.88-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/runtipi-fleet-dashboard /usr/local/bin/runtipi-fleet-dashboard
ENTRYPOINT ["/usr/local/bin/runtipi-fleet-dashboard"]
