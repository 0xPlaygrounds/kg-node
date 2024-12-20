use sdk::{
    blocks::{DataBlock, DataBlockType, TextBlock},
    pb::grc20,
    relation::create_relationship,
    system_ids,
};

pub struct Template {
    id: String,
    name: String,
    blocks: Vec<grc20::Triple>,
    types: Vec<String>,
    #[allow(dead_code)]
    foreign_types: Vec<String>,
    additional_data: Vec<grc20::Triple>,
}

pub fn non_profit() -> impl Iterator<Item = Template> {
    vec![
        Template {
            id: system_ids::NONPROFIT_SPACE_CONFIGURATION_TEMPLATE.to_string(),
            name: "Nonprofit Space Configuration Template".to_string(),
            blocks: std::iter::empty()
                .chain(TextBlock::create_triples(
                    system_ids::NONPROFIT_SPACE_CONFIGURATION_TEMPLATE,
                    "## Welcome to our nonprofit!",
                    Some("a0"),
                ))
                .chain(TextBlock::create_triples(
                    system_ids::NONPROFIT_SPACE_CONFIGURATION_TEMPLATE,
                    "We're thrilled to have you here. At our core, we are driven by a passionate commitment to positive change. As a community, we believe in the power of collective action to make a difference, no matter how big or small. Together, we can create meaningful impact and contribute to a better world. Thank you for joining us on this journey towards a brighter future.",
                    Some("a1"),
                ))
                .collect(),
            types: vec![
                system_ids::NONPROFIT_TYPE.to_string(),
                system_ids::SPACE_CONFIGURATION.to_string(),
                system_ids::PROJECT_TYPE.to_string(),
            ],
            foreign_types: vec![],
            additional_data: vec![],
        },
        Template {
            id: system_ids::NONPROFIT_POSTS_PAGE_TEMPLATE.to_string(),
            name: "Nonprofit Posts Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::NONPROFIT_POSTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Posts"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::NONPROFIT_POSTS_PAGE_TEMPLATE,
                system_ids::POSTS_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::NONPROFIT_PROJECTS_PAGE_TEMPLATE.to_string(),
            name: "Nonprofit Projects Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::NONPROFIT_PROJECTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Projects"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::NONPROFIT_PROJECTS_PAGE_TEMPLATE,
                system_ids::PROJECTS_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::NONPROFIT_TEAM_PAGE_TEMPLATE.to_string(),
            name: "Nonprofit Team Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::NONPROFIT_TEAM_PAGE_TEMPLATE,
                DataBlockType::Collection,
                None,
                Some("Team"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::NONPROFIT_TEAM_PAGE_TEMPLATE,
                system_ids::PROJECTS_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
            None,
            ).collect(),
        },
        Template {
            id: system_ids::NONPROFIT_FINANCES_PAGE_TEMPLATE.to_string(),
            name: "Nonprofit Finances Page Template".to_string(),
            blocks: std::iter::empty()
                .chain(TextBlock::create_triples(
                    system_ids::NONPROFIT_FINANCES_PAGE_TEMPLATE,
                    "Welcome to the finance summary of this nonprofit.",
                    Some("a0"),
                ))
                .chain(DataBlock::create_triples(
                    system_ids::NONPROFIT_FINANCES_PAGE_TEMPLATE,
                    DataBlockType::Geo,
                    Some("a1"),
                    Some("Finance Summaries"),
                ))
                .collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::NONPROFIT_FINANCES_PAGE_TEMPLATE,
                system_ids::FINANCES_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                None,
            ).collect(),
        },
    ].into_iter()
}

pub fn company() -> impl Iterator<Item = Template> {
    vec![
        Template {
            id: system_ids::COMPANY_SPACE_CONFIGURATION_TEMPLATE.to_string(),
            name: "Company Space Configuration Template".to_string(),
            blocks: std::iter::empty()
                .chain(TextBlock::create_triples(
                    system_ids::COMPANY_SPACE_CONFIGURATION_TEMPLATE,
                    "## Welcome to our company!",
                    Some("a0"),
                ))
                .chain(TextBlock::create_triples(
                    system_ids::COMPANY_SPACE_CONFIGURATION_TEMPLATE,
                    "We're dedicated to pushing boundaries and fostering innovation. With a focus on excellence and a passion for progress, we strive to make a positive impact in everything we do. From our talented team to our cutting-edge solutions, we're committed to delivering unparalleled quality and service to our customers.",
                    Some("a1"),
                ))
                .chain(DataBlock::create_triples(
                    system_ids::COMPANY_SPACE_CONFIGURATION_TEMPLATE,
                    DataBlockType::Collection,
                    Some("a2"),
                    Some("Goals"),
                ))
                .collect(),
            types: vec![
                system_ids::SPACE_CONFIGURATION.to_string(),
                system_ids::COMPANY_TYPE.to_string(),
            ],
            foreign_types: vec![],
            additional_data: vec![],
        },
        Template {
            id: system_ids::COMPANY_POSTS_PAGE_TEMPLATE.to_string(),
            name: "Company Posts Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_POSTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Posts"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_POSTS_PAGE_TEMPLATE,
                system_ids::POSTS_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::COMPANY_EVENTS_PAGE_TEMPLATE.to_string(),
            name: "Company Events Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_EVENTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Events"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_EVENTS_PAGE_TEMPLATE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                system_ids::EVENTS_PAGE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::COMPANY_JOBS_PAGE_TEMPLATE.to_string(),
            name: "Company Jobs Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_JOBS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Job openings"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_JOBS_PAGE_TEMPLATE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                system_ids::JOBS_PAGE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::COMPANY_PRODUCTS_PAGE_TEMPLATE.to_string(),
            name: "Company Products Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_PRODUCTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Products"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_PRODUCTS_PAGE_TEMPLATE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                system_ids::PRODUCTS_PAGE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::COMPANY_SERVICES_PAGE_TEMPLATE.to_string(),
            name: "Company Services Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_SERVICES_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Services"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_SERVICES_PAGE_TEMPLATE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                system_ids::SERVICES_PAGE,
                None,
            ).collect(),
        },
        Template {
            id: system_ids::COMPANY_TEAM_PAGE_TEMPLATE.to_string(),
            name: "Company Team Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::COMPANY_TEAM_PAGE_TEMPLATE,
                DataBlockType::Collection,
                None,
                Some("Team members"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::COMPANY_TEAM_PAGE_TEMPLATE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                system_ids::TEAM_PAGE,
                None,
            ).collect(),
        },
    ].into_iter()
}

pub fn person() -> impl Iterator<Item = Template> {
    vec![
        Template {
            id: system_ids::PERSON_SPACE_CONFIGURATION_TEMPLATE.to_string(),
            name: "Person Space Configuration Template".to_string(),
            blocks: std::iter::empty()
                .chain(TextBlock::create_triples(
                    system_ids::PERSON_SPACE_CONFIGURATION_TEMPLATE,
                    "## Welcome to my personal space",
                    Some("a0"),
                ))
                .chain(TextBlock::create_triples(
                    system_ids::PERSON_SPACE_CONFIGURATION_TEMPLATE,
                    "This space is where I compile my interests, posts, collections, and a summary of myself, along with anything else I'd like to share with the broader Geo community.",
                    Some("a1"),
                ))
                .chain(DataBlock::create_triples(
                    system_ids::PERSON_SPACE_CONFIGURATION_TEMPLATE,
                    DataBlockType::Collection,
                    Some("a2"),
                    Some("Goals"),
                ))
                .chain(DataBlock::create_triples(
                    system_ids::PERSON_SPACE_CONFIGURATION_TEMPLATE,
                    DataBlockType::Collection,
                    Some("a3"),
                    Some("Skills"),
                ))
                .collect(),
            types: vec![
                system_ids::SPACE_CONFIGURATION.to_string(),
                system_ids::PERSON_TYPE.to_string(),
            ],
            foreign_types: vec![],
            additional_data: vec![],
        },
        Template {
            id: system_ids::PERSON_POSTS_PAGE_TEMPLATE.to_string(),
            name: "Person Posts Page Template".to_string(),
            blocks: DataBlock::create_triples(
                system_ids::PERSON_POSTS_PAGE_TEMPLATE,
                DataBlockType::Geo,
                None,
                Some("Posts"),
            ).collect(),
            types: vec![system_ids::PAGE_TYPE.to_string()],
            foreign_types: vec![],
            additional_data: create_relationship(
                system_ids::PERSON_POSTS_PAGE_TEMPLATE,
                system_ids::POSTS_PAGE,
                system_ids::PAGE_TYPE_ATTRIBUTE,
                None,
            ).collect(),
        },
    ]
    .into_iter()
}

pub fn templates_ops() -> impl Iterator<Item = grc20::Triple> {
    std::iter::empty()
        .chain(non_profit())
        .chain(company())
        .chain(person())
        .flat_map(|template| {
            std::iter::empty()
                // Set attribute: Template.Name
                .chain(std::iter::once(grc20::Triple {
                    entity: template.id.clone(),
                    attribute: system_ids::NAME.to_string(),
                    value: Some(grc20::Value {
                        r#type: grc20::ValueType::Text as i32,
                        value: template.name,
                    }),
                }))
                // Create relation: Template > TYPES > *
                .chain(template.types.into_iter().flat_map(move |type_id| {
                    create_relationship(&template.id, &type_id, system_ids::TYPES, None)
                }))
                // Create relation: Template > BLOCKS > *
                .chain(template.blocks)
                // Add additional data triples
                .chain(template.additional_data)
        })
}
