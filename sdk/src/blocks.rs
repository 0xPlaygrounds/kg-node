use crate::{ids::create_geo_id, pb::grc20, relation::create_relationship, system_ids};

pub struct DataBlock;

impl DataBlock {
    pub fn create_triples(
        from_id: &str,
        source_type: DataBlockType,
        position: Option<&str>,
        name: Option<&str>,
    ) -> impl Iterator<Item = grc20::Triple> {
        let new_block_id = create_geo_id();

        std::iter::empty()
            // Create relation: NewBlock > TYPES > DataBlock
            .chain(create_relationship(
                &new_block_id,
                system_ids::DATA_BLOCK,
                system_ids::TYPES,
                None,
            ))
            // Create relation: NewBlock > DATA_SOURCE_TYPE_RELATION_TYPE > SourceType
            .chain(create_relationship(
                &new_block_id,
                source_type.to_id(),
                system_ids::DATA_SOURCE_TYPE_RELATION_TYPE,
                None,
            ))
            // Create relation: Entity > BLOCKS > NewBlock
            .chain(create_relationship(
                from_id,
                &new_block_id,
                system_ids::BLOCKS,
                position,
            ))
            // Set attribute: NewBlock.Name
            .chain(name.map(|name| grc20::Triple {
                entity: new_block_id,
                attribute: system_ids::NAME.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Text.into(),
                    value: name.to_string(),
                }),
            }))
    }
}

pub enum DataBlockType {
    Collection,
    Geo,
    Query,
}

impl DataBlockType {
    pub fn to_id(&self) -> &str {
        match self {
            DataBlockType::Collection => system_ids::COLLECTION_DATA_SOURCE,
            DataBlockType::Geo => system_ids::ALL_OF_GEO_DATA_SOURCE,
            DataBlockType::Query => system_ids::QUERY_DATA_SOURCE,
        }
    }
}

pub struct TextBlock;

impl TextBlock {
    pub fn create_triples(
        from_id: &str,
        text: &str,
        position: Option<&str>,
    ) -> impl Iterator<Item = grc20::Triple> {
        let new_block_id = create_geo_id();

        std::iter::empty()
            // Create relation: NewBlock > TYPES > TextBlock
            .chain(create_relationship(
                &new_block_id,
                system_ids::TEXT_BLOCK,
                system_ids::TYPES,
                None,
            ))
            // Create relation: Entity > BLOCKS > NewBlock
            .chain(create_relationship(
                from_id,
                &new_block_id,
                system_ids::BLOCKS,
                position,
            ))
            // Set attribute: NewBlock.MarkdownContent
            .chain(std::iter::once(grc20::Triple {
                entity: new_block_id,
                attribute: system_ids::MARKDOWN_CONTENT.to_string(),
                value: Some(grc20::Value {
                    r#type: grc20::ValueType::Text.into(),
                    value: text.to_string(),
                }),
            }))
    }
}

// pub struct ImageBlock;

// impl ImageBlock {
//     pub fn new(from_id: &str, url: &str, position: Option<&str>) -> impl Iterator<Item = grc20::Triple> {
//         let new_block_id = create_geo_id();

//         std::iter::empty()
//             // Create relation: NewBlock > TYPES > ImageBlock
//             .chain(create_relationship(&new_block_id, system_ids::IMAGE_BLOCK, system_ids::TYPES, None))
//             // Create relation: Entity > BLOCKS > NewBlock
//             .chain(create_relationship(from_id, &new_block_id, system_ids::BLOCKS, position))
//             // Set attribute: NewBlock.Url
//             .chain(std::iter::once(grc20::Triple {
//                 entity: new_block_id,
//                 attribute: system_ids::URL.to_string(),
//                 value: Some(grc20::Value { r#type: grc20::ValueType::Text.into(), value: url.to_string() }),
//             }))
//     }
// }
