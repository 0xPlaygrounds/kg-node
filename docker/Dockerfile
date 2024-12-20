FROM rust:1.81.0 AS builder

WORKDIR /kg-node
COPY . .
RUN apt-get update && apt-get upgrade -y
RUN apt-get install libssl-dev protobuf-compiler -y
RUN cargo build --release --bin sink --bin api

# Run image
FROM debian:bookworm-slim AS run

ENV neo4j_uri ""
ENV neo4j_user ""
ENV neo4j_pass ""

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y libssl-dev
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Sink image
FROM run AS sink

ENV SUBSTREAMS_API_TOKEN ""
ENV SUBSTREAMS_ENDPOINT_URL ""

COPY --from=builder /kg-node/target/release/sink .
COPY --from=builder /kg-node/geo-substream.spkg .

CMD ./sink \
    --reset-db \
    --neo4j-uri $neo4j_uri \
    --neo4j-user $neo4j_user \
    --neo4j-pass $neo4j_pass

# GraphQL API image
FROM run AS api
COPY --from=builder /kg-node/target/release/api .

CMD ./api \
    --neo4j-uri $neo4j_uri \
    --neo4j-user $neo4j_user \
    --neo4j-pass $neo4j_pass
