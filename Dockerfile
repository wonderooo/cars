FROM rust:1.90-bookworm AS chef
WORKDIR /cars
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    make \
    cmake \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /cars/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --features kafka-setup,minio-setup,postgres-setup

FROM debian:bookworm-slim AS runtime
WORKDIR /cars
RUN useradd -m cars-user
RUN chown -R cars-user:cars-user /cars
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    build-essential \
    libssl-dev \
    chromium \
    && rm -rf /var/lib/apt/lists/*
ENV CONFIG_PATH=config-docker.yaml
USER cars-user
COPY --from=builder /cars/target/release ./
COPY --from=builder /cars/config-docker.yaml ./

FROM runtime AS proxy
RUN mv proxy proxy-bin
EXPOSE 8100
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./proxy/log.txt | grep -q 'app started'"
ENTRYPOINT ["./proxy-bin"]

FROM runtime AS imgsync
RUN mv imgsync imgsync-bin
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./imgsync/log.txt | grep -q 'app started'"
ENTRYPOINT ["./imgsync-bin"]

FROM runtime AS sched
RUN mv sched sched-bin
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./sched/log.txt | grep -q 'app started'"
ENTRYPOINT ["./sched-bin"]

FROM runtime AS browser
RUN mv browser browser-bin
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./browser/log.txt | grep -q 'app started'"
ENTRYPOINT ["./browser-bin"]

FROM runtime AS persister
RUN mv persister persister-bin
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./persister/log.txt | grep -q 'app started'"
ENTRYPOINT ["./persister-bin"]

FROM runtime AS api
RUN mv api api-bin
EXPOSE 8081
HEALTHCHECK --interval=10s --timeout=3s \
  CMD sh -c "cat ./api/log.txt | grep -q 'app started'"
ENTRYPOINT ["./api-bin"]

FROM runtime AS kafka-setup
RUN mv kafka kafka-bin
ENTRYPOINT ["./kafka-bin"]

FROM runtime AS minio-setup
RUN mv minio minio-bin
ENTRYPOINT ["./minio-bin"]

FROM runtime AS postgres-setup
RUN mv postgres postgres-bin
ENTRYPOINT ["./postgres-bin"]