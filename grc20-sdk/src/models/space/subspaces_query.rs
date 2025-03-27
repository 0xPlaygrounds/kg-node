use std::collections::HashSet;

use async_stream::stream;
use futures::{pin_mut, Stream, StreamExt};

use grc20_core::{
    error::DatabaseError,
    indexer_ids,
    mapping::{query_utils::QueryStream, relation_node, PropFilter},
    neo4rs,
};

/// Query to find all subspaces of a given space
pub struct SubspacesQuery {
    neo4j: neo4rs::Graph,
    space_id: String,
    limit: usize,
    skip: Option<usize>,
    max_depth: Option<usize>,
}

impl SubspacesQuery {
    pub(crate) fn new(neo4j: neo4rs::Graph, space_id: String) -> Self {
        Self {
            neo4j,
            space_id,
            limit: 100,
            skip: None,
            max_depth: Some(1),
        }
    }

    /// Limit the number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Skip a number of results
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Limit the depth of the search
    pub fn max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }
}

// impl QueryStream<Entity<Space>> for SubspacesQuery {
//     async fn send(
//         self,
//     ) -> Result<impl Stream<Item = Result<Entity<Space>, DatabaseError>>, DatabaseError> {
//         // Find all parent space relations where this space is the parent
//         let relations_stream = relation_node::find_many(&self.neo4j)
//             .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
//             .from_id(PropFilter::default().value(self.space_id.clone()))
//             .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
//             .limit(self.limit)
//             .send()
//             .await?;

//         // Convert the stream of relations to a stream of spaces
//         let neo4j = self.neo4j.clone();
//         let space_stream = relations_stream
//             .map(move |relation_result| {
//                 let neo4j = neo4j.clone();
//                 async move {
//                     let relation = relation_result?;
//                     entity::find_one(&neo4j, &relation.to, indexer_ids::INDEXER_SPACE_ID, None)
//                         .send()
//                         .await?
//                         .ok_or_else(|| {
//                             DatabaseError::NotFound(format!(
//                                 "Space with ID {} not found",
//                                 relation.to
//                             ))
//                         })
//                 }
//             })
//             .buffered(10); // Process up to 10 spaces concurrently

//         Ok(space_stream)
//     }
// }

impl QueryStream<String> for SubspacesQuery {
    async fn send(
        self,
    ) -> Result<impl Stream<Item = Result<String, DatabaseError>>, DatabaseError> {
        let mut visited = HashSet::new();
        let mut queue = vec![(self.space_id.clone(), 0)]; // (space_id, depth)

        // Add initial space to visited set
        visited.insert(self.space_id.to_string());

        // Create and return the stream
        let stream = stream! {
            // Process queue until empty
            while let Some((current_space, depth)) = queue.pop() {
                // Check if we've reached max depth
                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        continue;
                    }
                }

                // Get immediate subspaces
                let subspaces = immediate_subspaces(&self.neo4j, &current_space, self.limit).await?;
                pin_mut!(subspaces);

                // Process each subspace
                while let Some(subspace_result) = subspaces.next().await {
                    match subspace_result {
                        Ok(subspace) => {
                            // Skip if already visited (handles cycles)
                            if !visited.insert(subspace.clone()) {
                                continue;
                            }

                            // Yield the subspace ID
                            yield Ok(subspace.clone());

                            // Add to queue for further processing
                            queue.push((subspace, depth + 1));
                        },
                        Err(e) => yield Err(e),
                    }
                }
            }
        };

        Ok(stream.skip(self.skip.unwrap_or(0)).take(self.limit))
    }
}

async fn immediate_subspaces(
    neo4j: &neo4rs::Graph,
    space_id: &str,
    limit: usize,
) -> Result<impl Stream<Item = Result<String, DatabaseError>>, DatabaseError> {
    // Find all parent space relations where this space is the parent
    let relations_stream = relation_node::find_many(neo4j)
        .relation_type(PropFilter::default().value(indexer_ids::PARENT_SPACE))
        .to_id(PropFilter::default().value(space_id))
        .space_id(PropFilter::default().value(indexer_ids::INDEXER_SPACE_ID))
        .limit(limit)
        .send()
        .await?;

    // Convert the stream of relations to a stream of spaces
    let space_stream =
        relations_stream.map(move |relation_result| relation_result.map(|relation| relation.from));

    Ok(space_stream)
}
