FROM ubuntu:22.04
COPY ./target/release/win-containers-registry ./target/release/win-containers-registry
ENTRYPOINT ["./target/release/win-containers-registry"]