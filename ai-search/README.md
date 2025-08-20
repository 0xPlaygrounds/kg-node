# README ai-search

## API
ai-search is a REST API that can be queried at the adress 0.0.0.0:3000.
The are 2 available routes to query the knowledge graph using natural language.

The first route is /question that takes a question in the body as a String and give the anser back quicker. It uses less Gemini calls and is faster.
The second route is /question_ai that takes a question in the body as a String and gives an answer back. It rellies more on Gemini calls and is slower, but has more precision.


## Start command
```
cargo run --bin ai-search -- --neo4j-uri neo4j://localhost:7687 --neo4j-user neo4j --neo4j-pass neo4j --gemini-api-key <Your gemini api key>

```