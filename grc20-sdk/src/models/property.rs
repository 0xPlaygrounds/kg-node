use futures::TryStreamExt;
use grc20_core::{
    entity::{self, Entity},
    error::DatabaseError,
    mapping::{
        aggregation::{AggregationDirection, SpaceRanking},
        entity::EntityNodeRef,
        prop_filter, triple, QueryStream, RelationEdge,
    },
    neo4rs, relation, system_ids,
};

use crate::models::space::ParentSpacesQuery;

use super::{base_entity, space::SubspacesQuery, BaseEntity};

#[grc20_core::entity]
#[grc20(schema_type = system_ids::PROPERTY_TYPE)]
pub struct Property {
    #[grc20(attribute = system_ids::AGGREGATION_DIRECTION)]
    pub aggregation_direction: Option<AggregationDirection>,

    #[grc20(attribute = system_ids::NAME_ATTRIBUTE)]
    pub name: Option<String>,

    #[grc20(attribute = system_ids::DESCRIPTION_ATTRIBUTE)]
    pub description: Option<String>,

    #[grc20(attribute = system_ids::COVER_ATTRIBUTE)]
    pub cover: Option<String>,
}

pub async fn value_type(
    neo4j: &neo4rs::Graph,
    property_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
    strict: bool,
) -> Result<Option<Entity<BaseEntity>>, DatabaseError> {
    let property_id = property_id.into();
    let space_id = space_id.into();

    let value_type_rel = get_outbound_relations::<RelationEdge<EntityNodeRef>>(
        neo4j,
        system_ids::VALUE_TYPE_ATTRIBUTE,
        &property_id,
        &space_id,
        space_version,
        Some(1),
        None,
        strict,
    )
    .await?
    .send()
    .await?
    .try_collect::<Vec<_>>()
    .await?;

    if let Some(value_type_rel) = value_type_rel.first() {
        base_entity::find_one(neo4j, &value_type_rel.to, &space_id)
            .send()
            .await
    } else {
        Ok(None)
    }
}

pub async fn relation_value_type(
    neo4j: &neo4rs::Graph,
    property_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
    strict: bool,
) -> Result<Option<Entity<BaseEntity>>, DatabaseError> {
    let property_id = property_id.into();
    let space_id = space_id.into();

    let value_type_rel = get_outbound_relations::<RelationEdge<EntityNodeRef>>(
        neo4j,
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
        &property_id,
        &space_id,
        space_version,
        Some(1),
        None,
        strict,
    )
    .await?
    .send()
    .await?
    .try_collect::<Vec<_>>()
    .await?;

    if let Some(value_type_rel) = value_type_rel.first() {
        base_entity::find_one(neo4j, &value_type_rel.to, &space_id)
            .send()
            .await
    } else {
        Ok(None)
    }
}

async fn attribute_aggregation_direction(
    neo4j: &neo4rs::Graph,
    space_id: &str,
    attribute_id: &str,
) -> Result<Option<AggregationDirection>, DatabaseError> {
    // Hardcoded for now as the aggregation direction triples are not yet present
    // in the knowledge graph
    // Might be able to change this to actual queries later
    match attribute_id {
        // This is the "base case", unclear if it could be replaced with a query even
        // if present in the knowledge graph
        system_ids::AGGREGATION_DIRECTION => return Ok(Some(AggregationDirection::Down)),

        // These are hardcoded for now since they are not yet present in the knowledge graph
        system_ids::NAME_ATTRIBUTE => return Ok(Some(AggregationDirection::Down)),
        system_ids::DESCRIPTION_ATTRIBUTE => return Ok(Some(AggregationDirection::Down)),
        system_ids::PROPERTIES => return Ok(Some(AggregationDirection::Down)),
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE => {
            return Ok(Some(AggregationDirection::Down))
        }
        system_ids::VALUE_TYPE_ATTRIBUTE => return Ok(Some(AggregationDirection::Down)),
        _ => (),
    }

    // Get all spaces to query (just the given space if strict, or all parent spaces if not)
    let mut spaces_to_query = vec![SpaceRanking {
        space_id: space_id.to_string(),
        depth: 0,
    }];

    let parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.to_string())
        .max_depth(None)
        .send()
        .await?
        .try_collect::<Vec<_>>()
        .await?;

    spaces_to_query.extend(parent_spaces);

    // Note: This may not be necessary since the parent spaces are collected using DFS
    // (i.e. the parent spaces *should* be sorted by depth)
    spaces_to_query.sort_by_key(|ranking| ranking.depth);

    for SpaceRanking { space_id, .. } in spaces_to_query {
        let maybe_triple = triple::find_one(
            neo4j,
            system_ids::AGGREGATION_DIRECTION,
            attribute_id,
            space_id,
            None,
        )
        .send()
        .await?;

        if let Some(triple) = maybe_triple {
            let direction = AggregationDirection::try_from(triple.value)?;
            return Ok(Some(direction));
        }
    }

    Ok(None)
}

// TODO: Find a better place for this function
pub async fn get_triple(
    neo4j: &neo4rs::Graph,
    property_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
    strict: bool,
) -> Result<Option<triple::Triple>, DatabaseError> {
    let space_id = space_id.into();
    let entity_id = entity_id.into();
    let property_id = property_id.into();

    let mut spaces = spaces_for_property(neo4j, &property_id, &space_id, strict).await?;

    spaces.sort_by_key(|ranking| ranking.depth);

    for SpaceRanking { space_id, .. } in spaces {
        let maybe_triple = triple::find_one(
            neo4j,
            &property_id,
            &entity_id,
            &space_id,
            space_version.clone(),
        )
        .send()
        .await?;

        if let Some(triple) = maybe_triple {
            return Ok(Some(triple));
        }
    }

    Ok(None)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_outbound_relations<T>(
    neo4j: &neo4rs::Graph,
    property_id: impl Into<String>,
    entity_id: impl Into<String>,
    space_id: impl Into<String>,
    space_version: Option<String>,
    limit: Option<usize>,
    skip: Option<usize>,
    strict: bool,
) -> Result<relation::FindManyQuery<T>, DatabaseError> {
    let neo4j = neo4j.clone();
    let space_id = space_id.into();
    let entity_id = entity_id.into();
    let property_id = property_id.into();

    let spaces = spaces_for_property(&neo4j, &property_id, &space_id, strict)
        .await?
        .into_iter()
        .map(|ranking| ranking.space_id)
        .collect::<Vec<_>>();
    // spaces.sort_by_key(|(_, depth)| *depth);

    // TODO: Optimization: We can accept limit/skip parameters here and pass them to the query.
    // By counting the number of results we can determine if we need to continue to the next space
    // or if we have enough results already.
    // let stream = try_stream! {
    //     for (space_id, _) in spaces {
    //         let relations_stream = relation_node::FindManyQuery::new(&neo4j)
    //             .from_id(prop_filter::value(entity_id.clone()))
    //             .space_id(prop_filter::value(space_id.clone()))
    //             .relation_type(prop_filter::value(property_id.clone()))
    //             .version(space_version.clone())
    //             .send()
    //             .await?;

    //         pin_mut!(relations_stream);

    //         while let Some(relation) = relations_stream.try_next().await? {
    //             yield relation;
    //         }
    //     }
    // };

    Ok(relation::find_many::<T>(&neo4j)
        .filter(
            relation::RelationFilter::default()
                .from_(entity::EntityFilter::default().id(prop_filter::value(&entity_id)))
                .relation_type(
                    entity::EntityFilter::default().id(prop_filter::value(&property_id)),
                ),
        )
        .space_id(prop_filter::value_in(spaces))
        .version(space_version.clone())
        .limit(limit.unwrap_or(100))
        .skip(skip.unwrap_or(0)))
}

/// Returns the spaces from which the property is inherited
async fn spaces_for_property(
    neo4j: &neo4rs::Graph,
    property_id: impl Into<String>,
    space_id: impl Into<String>,
    strict: bool,
) -> Result<Vec<SpaceRanking>, DatabaseError> {
    let space_id = space_id.into();
    let property_id = property_id.into();

    let mut spaces = vec![SpaceRanking {
        space_id: space_id.clone(),
        depth: 0,
    }];

    if strict {
        return Ok(spaces);
    }

    match attribute_aggregation_direction(neo4j, &space_id, &property_id).await? {
        Some(AggregationDirection::Up) => {
            let subspaces = SubspacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            spaces.extend(subspaces);
            Ok(spaces)
        }
        Some(AggregationDirection::Down) => {
            let parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            spaces.extend(parent_spaces);
            Ok(spaces)
        }
        Some(AggregationDirection::Bidirectional) => {
            let subspaces = SubspacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            let parent_spaces = ParentSpacesQuery::new(neo4j.clone(), space_id.clone())
                .max_depth(None)
                .send()
                .await?
                .try_collect::<Vec<_>>()
                .await?;

            spaces.extend(subspaces);
            spaces.extend(parent_spaces);
            Ok(spaces)
        }
        None => Ok(spaces),
    }
}
