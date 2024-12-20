// See https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/substream/sink/bootstrap-root.ts

use sdk::{network_ids, pb::grc20, relation::create_relationship, system_ids};

use super::{bootstrap_templates, constants};

// (entity_id, name)
const NAMES: &[(&str, &str)] = &[
    (system_ids::TYPES, "Types"),
    (system_ids::NAME, "Name"),
    (system_ids::ATTRIBUTE, "Attribute"),
    (system_ids::RELATION_TYPE_ATTRIBUTE, "Relation Type"),
    (system_ids::ATTRIBUTES, "Attributes"),
    (system_ids::SCHEMA_TYPE, "Type"),
    (system_ids::TEMPLATE_ATTRIBUTE, "Template"),
    (system_ids::VALUE_TYPE, "Value type"),
    (system_ids::RELATION_TYPE, "Relation"),
    (system_ids::TEXT, "Text"),
    (system_ids::CHECKBOX, "Checkbox"),
    (system_ids::IMAGE, "Image"),
    (system_ids::IMAGE_URL_ATTRIBUTE, "Image URL"),
    (system_ids::DATE, "Date"),
    (system_ids::WEB_URL, "Web URL"),
    (system_ids::DESCRIPTION, "Description"),
    (system_ids::SPACE_CONFIGURATION, "Space"),
    (system_ids::SOURCE_SPACE_ATTRIBUTE, "Source space"),
    (system_ids::VERIFIED_SOURCE_ATTRIBUTE, "Verified Source"),
    (system_ids::FOREIGN_TYPES, "Foreign Types"),
    // Data blocks
    (system_ids::VIEW_TYPE, "View"),
    (system_ids::DATA_BLOCK, "Data Block"),
    (system_ids::VIEW_ATTRIBUTE, "View"),
    (system_ids::GALLERY_VIEW, "Gallery View"),
    (system_ids::TABLE_VIEW, "Table View"),
    (system_ids::LIST_VIEW, "List View"),
    (system_ids::SHOWN_COLUMNS, "Shown Columns"),
    (system_ids::TEXT_BLOCK, "Text Block"),
    (system_ids::IMAGE_BLOCK, "Image Block"),
    (system_ids::BLOCKS, "Blocks"),
    (system_ids::FILTER, "Filter"),
    (system_ids::SPACE_FILTER, "Space filter"),
    (system_ids::MARKDOWN_CONTENT, "Markdown Content"),
    (system_ids::PLACEHOLDER_IMAGE, "Placeholder Image"),
    (system_ids::PLACEHOLDER_TEXT, "Placeholder Text"),
    (system_ids::PERSON_TYPE, "Person"),
    (system_ids::AVATAR_ATTRIBUTE, "Avatar"),
    (system_ids::COVER_ATTRIBUTE, "Cover"),
    (system_ids::ACCOUNTS_ATTRIBUTE, "Accounts"),
    (system_ids::NETWORK_TYPE, "Network"),
    (system_ids::NETWORK_ATTRIBUTE, "Network"),
    (system_ids::ADDRESS_ATTRIBUTE, "Address"),
    (system_ids::ACCOUNT_TYPE, "Account"),
    (network_ids::ETHEREUM, "Ethereum"),
    (system_ids::BROADER_SPACES, "Broader Spaces"),
    (
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
        "Relation Value Types",
    ),
    (system_ids::RELATION, "Relation"),
    (system_ids::RELATION_INDEX, "Index"),
    (system_ids::RELATION_TO_ATTRIBUTE, "To entity"),
    (system_ids::RELATION_FROM_ATTRIBUTE, "From entity"),
    (system_ids::DATA_SOURCE_ATTRIBUTE, "Data Source"),
    (
        system_ids::DATA_SOURCE_TYPE_RELATION_TYPE,
        "Data Source Type",
    ),
    (system_ids::COLLECTION_DATA_SOURCE, "Collection Data Source"),
    (system_ids::ALL_OF_GEO_DATA_SOURCE, "Geo Data Source"),
    (system_ids::QUERY_DATA_SOURCE, "Query Data Source"),
    (system_ids::COLLECTION_ITEM_RELATION_TYPE, "Collection Item"),
    // Templates + Space Layouts
    (system_ids::NONPROFIT_TYPE, "Nonprofit"),
    (system_ids::PROJECT_TYPE, "Project"),
    (system_ids::COMPANY_TYPE, "Company"),
    (system_ids::PAGE_TYPE, "Page"),
    (system_ids::PAGE_TYPE_ATTRIBUTE, "Page type"),
    (system_ids::POSTS_PAGE, "Posts page"),
    (system_ids::PROJECTS_PAGE, "Projects page"),
    (system_ids::FINANCES_PAGE, "Finances page"),
    (system_ids::TEAM_PAGE, "Team page"),
    (system_ids::JOBS_PAGE, "Jobs page"),
    (system_ids::EVENTS_PAGE, "Events page"),
    (system_ids::SERVICES_PAGE, "Services page"),
    (system_ids::PRODUCTS_PAGE, "Products page"),
];

// (attribute_id, value_type_id)
const ATTRIBUTES: &[(&str, &str)] = &[
    (system_ids::TYPES, system_ids::RELATION),
    (system_ids::TEMPLATE_ATTRIBUTE, system_ids::RELATION),
    (system_ids::ATTRIBUTES, system_ids::RELATION),
    (system_ids::RELATION_TYPE_ATTRIBUTE, system_ids::RELATION),
    (system_ids::VALUE_TYPE, system_ids::RELATION),
    (system_ids::DESCRIPTION, system_ids::TEXT),
    (system_ids::NAME, system_ids::TEXT),
    (system_ids::SOURCE_SPACE_ATTRIBUTE, system_ids::RELATION),
    (system_ids::VERIFIED_SOURCE_ATTRIBUTE, system_ids::CHECKBOX),
    // Data blocks
    (system_ids::VIEW_ATTRIBUTE, system_ids::RELATION),
    (system_ids::FOREIGN_TYPES, system_ids::RELATION),
    (system_ids::MARKDOWN_CONTENT, system_ids::TEXT),
    (system_ids::BLOCKS, system_ids::RELATION),
    (system_ids::FILTER, system_ids::TEXT),
    (system_ids::PLACEHOLDER_IMAGE, system_ids::RELATION),
    (system_ids::PLACEHOLDER_TEXT, system_ids::TEXT),
    (
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
        system_ids::RELATION,
    ),
    (system_ids::RELATION_INDEX, system_ids::TEXT),
    (system_ids::RELATION_TO_ATTRIBUTE, system_ids::RELATION),
    (system_ids::RELATION_FROM_ATTRIBUTE, system_ids::RELATION),
    (system_ids::IMAGE_URL_ATTRIBUTE, system_ids::WEB_URL),
    (system_ids::BROADER_SPACES, system_ids::RELATION),
    (
        system_ids::DATA_SOURCE_TYPE_RELATION_TYPE,
        system_ids::RELATION,
    ),
    (system_ids::DATA_SOURCE_ATTRIBUTE, system_ids::RELATION),
    (
        system_ids::COLLECTION_ITEM_RELATION_TYPE,
        system_ids::RELATION,
    ),
    (system_ids::PAGE_TYPE, system_ids::RELATION),
    (system_ids::AVATAR_ATTRIBUTE, system_ids::IMAGE),
    (system_ids::COVER_ATTRIBUTE, system_ids::IMAGE),
    (system_ids::ACCOUNTS_ATTRIBUTE, system_ids::RELATION),
    (system_ids::NETWORK_ATTRIBUTE, system_ids::RELATION),
    (system_ids::ADDRESS_ATTRIBUTE, system_ids::TEXT),
];

// These types include the default types and attributes for a given type. There might be more
// attributes on a type than are listed here if they were later added by users.
// (type_id, [attribute_id])
const SCHEMA_TYPES: &[(&str, &[&str])] = &[
    (system_ids::SCHEMA_TYPE, &[system_ids::TEMPLATE_ATTRIBUTE]),
    (system_ids::VIEW_TYPE, &[]),
    (system_ids::TEXT, &[]),
    (system_ids::CHECKBOX, &[]),
    (system_ids::RELATION, &[]),
    (system_ids::IMAGE, &[system_ids::IMAGE_URL_ATTRIBUTE]),
    (system_ids::DATE, &[]),
    (system_ids::WEB_URL, &[]),
    (system_ids::ATTRIBUTE, &[system_ids::VALUE_TYPE]),
    (
        system_ids::SPACE_CONFIGURATION,
        &[system_ids::FOREIGN_TYPES, system_ids::BLOCKS],
    ),
    (system_ids::IMAGE_BLOCK, &[system_ids::IMAGE_URL_ATTRIBUTE]),
    (system_ids::DATA_BLOCK, &[]),
    (system_ids::TEXT_BLOCK, &[system_ids::MARKDOWN_CONTENT]),
    (
        system_ids::PERSON_TYPE,
        &[system_ids::AVATAR_ATTRIBUTE, system_ids::COVER_ATTRIBUTE],
    ),
    (
        system_ids::ACCOUNT_TYPE,
        &[system_ids::NETWORK_ATTRIBUTE, system_ids::ADDRESS_ATTRIBUTE],
    ),
    (system_ids::NETWORK_TYPE, &[]),
    (system_ids::NONPROFIT_TYPE, &[]),
    (system_ids::PROJECT_TYPE, &[]),
    (system_ids::COMPANY_TYPE, &[]),
    (
        system_ids::RELATION_TYPE,
        &[
            system_ids::RELATION_INDEX,
            system_ids::RELATION_TO_ATTRIBUTE,
            system_ids::RELATION_FROM_ATTRIBUTE,
            system_ids::RELATION_TYPE_ATTRIBUTE,
        ],
    ),
];

const TYPES: &[(&str, &[&str])] = &[(network_ids::ETHEREUM, &[system_ids::NETWORK_TYPE])];

pub fn name_ops() -> impl Iterator<Item = grc20::Triple> {
    NAMES.iter().map(|(id, name)| grc20::Triple {
        entity: id.to_string(),
        attribute: system_ids::NAME.to_string(),
        value: Some(grc20::Value {
            r#type: grc20::ValueType::Text as i32,
            value: name.to_string(),
        }),
    })
}

pub fn attribute_ops() -> impl Iterator<Item = grc20::Triple> {
    ATTRIBUTES.iter().flat_map(|(attribute_id, _)| {
        create_relationship(attribute_id, system_ids::ATTRIBUTE, system_ids::TYPES, None)
    })
}

pub fn attribute_value_type_ops() -> impl Iterator<Item = grc20::Triple> {
    ATTRIBUTES.iter().flat_map(|(attribute_id, value_type_id)| {
        create_relationship(attribute_id, value_type_id, system_ids::VALUE_TYPE, None)
    })
}

pub fn type_ops() -> impl Iterator<Item = grc20::Triple> {
    SCHEMA_TYPES.iter().flat_map(|(type_id, _)| {
        create_relationship(type_id, system_ids::SCHEMA_TYPE, system_ids::TYPES, None)
    })
}

pub fn root_space_type() -> impl Iterator<Item = grc20::Triple> {
    create_relationship(
        constants::ROOT_SPACE_ID,
        system_ids::SPACE_CONFIGURATION,
        system_ids::TYPES,
        None,
    )
}

pub fn type_schema_ops() -> impl Iterator<Item = grc20::Triple> {
    SCHEMA_TYPES.iter().flat_map(|(type_id, attributes)| {
        attributes
            .iter()
            .flat_map(|attribute_id| {
                create_relationship(type_id, attribute_id, system_ids::ATTRIBUTES, None)
            })
            .collect::<Vec<_>>()
    })
}

pub fn entities_types_ops() -> impl Iterator<Item = grc20::Triple> {
    TYPES.iter().flat_map(|(entity_id, types_ids)| {
        types_ids
            .iter()
            .flat_map(|type_id| create_relationship(entity_id, type_id, system_ids::TYPES, None))
            .collect::<Vec<_>>()
    })
}

pub fn bootstrap() -> impl Iterator<Item = grc20::Op> {
    std::iter::empty()
        .chain(name_ops())
        .chain(attribute_ops())
        .chain(attribute_value_type_ops())
        .chain(type_ops())
        .chain(root_space_type())
        .chain(type_schema_ops())
        .chain(entities_types_ops())
        .chain(bootstrap_templates::templates_ops())
        .map(|op| grc20::Op {
            r#type: grc20::OpType::SetTriple as i32,
            triple: Some(op),
        })
}

// const RELATION_TYPES: &[(&str, &[&str])] = &[
//     (system_ids::TYPES, &[system_ids::ATTRIBUTES]),
//     (system_ids::ATTRIBUTES, &[system_ids::VALUE_TYPE]),
// ];
