FROM rust:1.88-bookworm@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0 AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked \
    && mkdir -p /runtime/state /runtime/homepage

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
LABEL org.opencontainers.image.source="https://github.com/EckPhi/runtipi-fleet-dashboard" \
      org.opencontainers.image.title="Runtipi Fleet Dashboard" \
      org.opencontainers.image.description="Collect Runtipi companion inventories and render Homepage configuration" \
      org.opencontainers.image.licenses="MIT"
COPY --from=build /src/target/release/runtipi-fleet-dashboard /usr/local/bin/runtipi-fleet-dashboard
COPY --from=build --chown=65532:65532 /runtime/state /state
COPY --from=build --chown=65532:65532 /runtime/homepage /homepage
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/runtipi-fleet-dashboard"]
