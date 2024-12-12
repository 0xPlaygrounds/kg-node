// @generated
// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpfsMetadata {
    /// We version the data structured used to represent proposal metadata. Each
    /// proposal type has their own metadata and versioning that we can change
    /// independently of other proposal types.
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(enumeration="ActionType", tag="2")]
    pub r#type: i32,
    #[prost(string, tag="3")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Edit {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(enumeration="ActionType", tag="2")]
    pub r#type: i32,
    #[prost(string, tag="3")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="5")]
    pub ops: ::prost::alloc::vec::Vec<Op>,
    #[prost(string, repeated, tag="6")]
    pub authors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportEdit {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(enumeration="ActionType", tag="2")]
    pub r#type: i32,
    #[prost(string, tag="3")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="5")]
    pub ops: ::prost::alloc::vec::Vec<Op>,
    #[prost(string, repeated, tag="6")]
    pub authors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag="7")]
    pub created_by: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub created_at: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub block_hash: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub block_number: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub transaction_hash: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Op {
    #[prost(enumeration="OpType", tag="1")]
    pub r#type: i32,
    #[prost(message, optional, tag="2")]
    pub triple: ::core::option::Option<Triple>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Triple {
    #[prost(string, tag="1")]
    pub entity: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub attribute: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub value: ::core::option::Option<Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(enumeration="ValueType", tag="1")]
    pub r#type: i32,
    #[prost(string, tag="2")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Membership {
    #[prost(enumeration="ActionType", tag="1")]
    pub r#type: i32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub user: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Subspace {
    #[prost(enumeration="ActionType", tag="1")]
    pub r#type: i32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub subspace: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Import {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(enumeration="ActionType", tag="2")]
    pub r#type: i32,
    #[prost(string, tag="3")]
    pub previous_network: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub previous_contract_address: ::prost::alloc::string::String,
    #[prost(string, repeated, tag="5")]
    pub edits: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Options {
    #[prost(string, tag="1")]
    pub format: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub crop: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OpType {
    None = 0,
    SetTriple = 1,
    DeleteTriple = 2,
}
impl OpType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OpType::None => "NONE",
            OpType::SetTriple => "SET_TRIPLE",
            OpType::DeleteTriple => "DELETE_TRIPLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "NONE" => Some(Self::None),
            "SET_TRIPLE" => Some(Self::SetTriple),
            "DELETE_TRIPLE" => Some(Self::DeleteTriple),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ValueType {
    Unknown = 0,
    Text = 1,
    Number = 2,
    Checkbox = 3,
    Url = 4,
    Time = 5,
    Point = 6,
}
impl ValueType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ValueType::Unknown => "UNKNOWN",
            ValueType::Text => "TEXT",
            ValueType::Number => "NUMBER",
            ValueType::Checkbox => "CHECKBOX",
            ValueType::Url => "URL",
            ValueType::Time => "TIME",
            ValueType::Point => "POINT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "TEXT" => Some(Self::Text),
            "NUMBER" => Some(Self::Number),
            "CHECKBOX" => Some(Self::Checkbox),
            "URL" => Some(Self::Url),
            "TIME" => Some(Self::Time),
            "POINT" => Some(Self::Point),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ActionType {
    Empty = 0,
    AddEdit = 1,
    AddSubspace = 2,
    RemoveSubspace = 3,
    ImportSpace = 4,
    ArchiveSpace = 5,
    AddEditor = 6,
    RemoveEditor = 7,
    AddMember = 8,
    RemoveMember = 9,
}
impl ActionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ActionType::Empty => "EMPTY",
            ActionType::AddEdit => "ADD_EDIT",
            ActionType::AddSubspace => "ADD_SUBSPACE",
            ActionType::RemoveSubspace => "REMOVE_SUBSPACE",
            ActionType::ImportSpace => "IMPORT_SPACE",
            ActionType::ArchiveSpace => "ARCHIVE_SPACE",
            ActionType::AddEditor => "ADD_EDITOR",
            ActionType::RemoveEditor => "REMOVE_EDITOR",
            ActionType::AddMember => "ADD_MEMBER",
            ActionType::RemoveMember => "REMOVE_MEMBER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EMPTY" => Some(Self::Empty),
            "ADD_EDIT" => Some(Self::AddEdit),
            "ADD_SUBSPACE" => Some(Self::AddSubspace),
            "REMOVE_SUBSPACE" => Some(Self::RemoveSubspace),
            "IMPORT_SPACE" => Some(Self::ImportSpace),
            "ARCHIVE_SPACE" => Some(Self::ArchiveSpace),
            "ADD_EDITOR" => Some(Self::AddEditor),
            "REMOVE_EDITOR" => Some(Self::RemoveEditor),
            "ADD_MEMBER" => Some(Self::AddMember),
            "REMOVE_MEMBER" => Some(Self::RemoveMember),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)