# GRC20 Neo4j Indexer
This repo contains a GRC20 indexer that uses Neo4j to store triple data. 

## Running the indexer
❗Important: Both methods require the `SUBSTREAMS_ENDPOINT_URL` (and optionally `SUBSTREAMS_API_TOKEN` if using a substreams provider with authentication) environment variables to be set.

### With `docker-compose`
```bash
cd docker/
docker compose up
```

### Without docker
### 1. Start Neo4j
```bash
docker run \
    --publish=7474:7474 --publish=7687:7687 \
    --volume=./data:/data \
    --env=NEO4J_AUTH=none \
    neo4j
```

### 2. Compile and run the indexer
In a separate terminal, run the following commands:
```bash
CFLAGS='-std=gnu17' cargo run --bin sink -- \
    --reset-db \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j
```

```bash
CFLAGS='-std=gnu17' cargo run --bin api -- \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j
```

## GRC20 CLI
Coming soon™️