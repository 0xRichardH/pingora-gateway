####################################################################################################
## Builder
####################################################################################################
FROM rust:1.80 AS builder

RUN apt-get update && apt-get install -y cmake
RUN update-ca-certificates

# Create appuser
ENV USER=gateway
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /gateway

COPY ./ .


# We no longer need to use the x86_64-unknown-linux-musl target
RUN cargo build --release

RUN strip -s /gateway/target/release/pingora-gateway

####################################################################################################
## Final image
####################################################################################################
FROM gcr.io/distroless/cc-debian12

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /gateway

# Copy our build
COPY --from=builder /gateway/target/release/pingora-gateway ./

# Use an unprivileged user.
USER gateway:gateway

CMD ["/gateway/pingora-gateway"]
