use neo4rs::BoltType;

pub fn serde_value_to_bolt(value: serde_json::Value) -> BoltType {
    match value {
        serde_json::Value::Null => BoltType::Null(neo4rs::BoltNull),
        serde_json::Value::Bool(value) => BoltType::Boolean(neo4rs::BoltBoolean { value }),
        serde_json::Value::Number(number) if number.is_i64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_i64().expect("number should be an i64"),
            })
        }
        serde_json::Value::Number(number) if number.is_u64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_u64().expect("number should be a u64") as i64,
            })
        }
        serde_json::Value::Number(number) => BoltType::Float(neo4rs::BoltFloat {
            value: number.as_f64().expect("number should be an f64"),
        }),
        serde_json::Value::String(value) => BoltType::String(neo4rs::BoltString { value }),
        serde_json::Value::Array(vec) => {
            let values = vec.into_iter().map(serde_value_to_bolt).collect();
            BoltType::List(neo4rs::BoltList { value: values })
        }
        serde_json::Value::Object(map) => {
            let properties = map
                .into_iter()
                .filter(|(key, _)| key != "$type")
                .map(|(key, value)| {
                    (
                        neo4rs::BoltString { value: key },
                        serde_value_to_bolt(value),
                    )
                })
                .collect();
            BoltType::Map(neo4rs::BoltMap { value: properties })
        }
    }
}

// TODO: Re-enale these tests with `testcontainers` to run against a real Neo4j instance.
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_find_one_node() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let query = neo4rs::query("MATCH (n) RETURN n LIMIT 1");

//         let node: Option<serde_json::Value> = graph.find_one(query).await.unwrap();
//         println!("{:?}", node);
//     }

//     #[tokio::test]
//     async fn test_find_one_relation() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let query = neo4rs::query("MATCH () -[r]-> () RETURN r LIMIT 1");

//         let node: Option<serde_json::Value> = graph.find_one(query).await.unwrap();
//         println!("{:?}", node);
//     }

//     #[tokio::test]
//     async fn test_find_all_nodes() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let query = neo4rs::query("MATCH (n:`d7ab40920ab5441e88c35c27952de773`) RETURN n LIMIT 4");

//         let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
//         println!("{:?}", nodes);
//     }

//     #[tokio::test]
//     async fn test_find_all_relations() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let query =
//             neo4rs::query("MATCH () -[r:`01412f8381894ab1836565c7fd358cc1`]-> () RETURN r LIMIT 4");

//         let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
//         println!("{:?}", nodes);
//     }

//     #[tokio::test]
//     async fn test_find_all_relations_and_nodes() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let query = neo4rs::query("MATCH () -[r:`577bd9fbb29e4e2bb5f8f48aedbd26ac`]-> () MATCH (n:`3d0b19e0313843479687a351223efcc3`) RETURN n, r LIMIT 8");

//         let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
//         println!("{}", serde_json::to_string_pretty(&nodes).unwrap());
//     }

//     #[tokio::test]
//     async fn test_find_one_node_txn() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let mut txn = graph.start_txn().await.unwrap();

//         txn.run(neo4rs::query("CREATE (n:TestNode {name: \"potato\"})"))
//             .await
//             .unwrap();

//         let query = neo4rs::query("MATCH (n:TestNode) RETURN n LIMIT 1");

//         let node: Option<serde_json::Value> = txn.find_one(query).await.unwrap();

//         txn.rollback().await.unwrap();

//         println!("{:?}", node);
//     }

//     #[tokio::test]
//     async fn test_insert_one_txn() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let mut txn = graph.start_txn().await.unwrap();

//         #[derive(Deserialize, Debug, Serialize)]
//         #[serde(tag = "$type")]
//         struct Foo {
//             name: String,
//         }

//         let node = Foo {
//             name: "potato".to_string(),
//         };

//         txn.insert_one(node).await.unwrap();

//         let node: Option<Foo> = txn
//             .find_one(neo4rs::query("MATCH (n:Foo) RETURN n LIMIT 1"))
//             .await
//             .unwrap();

//         txn.rollback().await.unwrap();

//         println!("{:?}", node);
//     }

//     #[tokio::test]
//     async fn test_insert_many_txn() {
//         let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
//             .await
//             .unwrap();

//         let mut txn = graph.start_txn().await.unwrap();

//         #[derive(Deserialize, Debug, Serialize)]
//         #[serde(tag = "$type")]
//         struct Foo {
//             name: String,
//         }

//         let nodes = vec![
//             Foo {
//                 name: "potato".to_string(),
//             },
//             Foo {
//                 name: "pizza".to_string(),
//             },
//         ];

//         txn.insert_many(nodes).await.unwrap();

//         let node: Vec<Foo> = txn
//             .find_all(neo4rs::query("MATCH (n:Foo) RETURN n"))
//             .await
//             .unwrap();

//         txn.rollback().await.unwrap();

//         println!("{:?}", node);
//     }
// }
