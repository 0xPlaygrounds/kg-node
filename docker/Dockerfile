FROM rust:1.87.0 AS builder

WORKDIR /kg-node
COPY . .
RUN apt-get update && apt-get upgrade -y
RUN apt-get install libssl-dev protobuf-compiler -y
RUN CFLAGS='-std=gnu17' cargo build --release --bin sink --bin mcp-server

# Run image
FROM debian:bookworm-slim AS run

ENV neo4j_uri ""
ENV neo4j_user ""
ENV neo4j_pass ""

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y libssl-dev netcat-traditional
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Sink image
FROM run AS sink

ENV SUBSTREAMS_API_TOKEN ""
ENV SUBSTREAMS_ENDPOINT_URL ""
ENV SUBSTREAMS_START_BLOCK ""

COPY --from=builder /kg-node/target/release/sink .
COPY --from=builder /kg-node/geo-substream.spkg .

CMD ./sink \
    --no-versioning \
    --no-governance \
    --neo4j-uri $neo4j_uri \
    --neo4j-user $neo4j_user \
    --neo4j-pass $neo4j_pass

# # GraphQL API image
# FROM run AS api
# COPY --from=builder /kg-node/target/release/api .

# CMD ./api \
#     --neo4j-uri $neo4j_uri \
#     --neo4j-user $neo4j_user \
#     --neo4j-pass $neo4j_pass

# MCP server image
FROM run AS mcp
COPY --from=builder /kg-node/target/release/mcp-server .

CMD ./mcp-server \
    --neo4j-uri $neo4j_uri \
    --neo4j-user $neo4j_user \
    --neo4j-pass $neo4j_pass
