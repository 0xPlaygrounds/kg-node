#[derive(Clone, Debug, Default)]
pub enum RelationDirection {
    From,
    To,
    #[default]
    Both,
}
