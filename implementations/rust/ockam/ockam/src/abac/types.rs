use core::fmt;
use ockam_core::compat::{
    collections::BTreeMap,
    string::{String, ToString},
};
use ockam_identity::IdentityIdentifier;
use serde::{Deserialize, Serialize};

/// An ABAC `Subject` entity.
///
/// `Subject` will usually map to an entity performing an
/// authorization request such as a user id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Subject {
    identifier: Identity,
    attributes: BTreeMap<Key, Value>,
}

/// FIXME ockam_identity::IdentityIdentifier ???
pub type Identity = String;

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

// TODO remove
impl From<u64> for Subject {
    fn from(identifier: u64) -> Self {
        Self {
            identifier: format!("{:x}", identifier),
            attributes: BTreeMap::default(),
        }
    }
}

impl From<IdentityIdentifier> for Subject {
    fn from(_identity: IdentityIdentifier) -> Self {
        Self {
            // FIXME test value for demo
            identifier: format!("{:x}", 0x0000_0000_0000_0001),
            attributes: BTreeMap::default(),
        }
    }
}

impl Subject {
    /// Extend a `Subject` with the given attributes.
    ///
    /// Any pre-existing attributes with matching [`Key`]s will be
    /// overwritten.
    ///
    /// TODO move this to the Attributes impl
    pub fn extend<A>(&mut self, attributes: A)
    where
        A: IntoIterator<Item = Attribute> + Send + 'static,
    {
        self.attributes.extend(attributes);
    }

    /// Return a reference to the `identitfier` field.
    pub fn identifier(&self) -> &Identity {
        &self.identifier
    }

    /// Return a reference to the `attributes` field.
    pub fn attributes(&self) -> &BTreeMap<Key, Value> {
        &self.attributes
    }
}

/// An ABAC `Resource` entity.
///
/// `Resource` maps to the given resource being placed under access
/// control such as a file or network path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Resource {
    name: String,
    attributes: BTreeMap<Key, Value>,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<&str> for Resource {
    fn from(s: &str) -> Self {
        Self {
            name: s.to_string(),
            attributes: BTreeMap::default(),
        }
    }
}

impl Resource {
    /// Extend a `Resource` with the given attributes.
    ///
    /// Any pre-existing attributes with matching [`Key`]s will be
    /// overwritten.
    pub fn extend<A>(&mut self, attributes: A)
    where
        A: IntoIterator<Item = Attribute> + Send + 'static,
    {
        self.attributes.extend(attributes);
    }

    /// Return a reference to the `name` field.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Return a reference to the `attributes` field.
    pub fn attributes(&self) -> &BTreeMap<Key, Value> {
        &self.attributes
    }
}

/// An ABAC `Action` entity.
///
/// `Action` corresponds to the action the requesting `Subject` wants
/// to perform on a `Resource`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Action {
    method: String, // TODO s/String/Method
    attributes: BTreeMap<Key, Value>,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.method)
    }
}

// TODO delete
impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Self {
            method: s.to_string(),
            attributes: BTreeMap::default(),
        }
    }
}

impl From<Method> for Action {
    fn from(method: Method) -> Self {
        Self {
            method: method.into(),
            attributes: BTreeMap::default(),
        }
    }
}

/// HTTP verbs
/// TODO conversion to/from ockam_api::Method
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Method {
    /// HTTP GET
    Get,
    /// HTTP POST
    Post,
    /// HTTP PUT
    Put,
    /// HTTP DELETE
    Delete,
    /// HTTP PATCH
    Patch,
}

// TODO delete
impl From<Method> for String {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => "GET".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put => "PUT".to_string(),
            Method::Delete => "DELETE".to_string(),
            Method::Patch => "PATCH".to_string(),
        }
    }
}

impl Action {
    /// Extend a `Resource` with the given attributes.
    ///
    /// Any pre-existing attributes with matching [`Key`]s will be
    /// overwritten.
    pub fn extend<A>(&mut self, attributes: A)
    where
        A: IntoIterator<Item = Attribute> + Send + 'static,
    {
        self.attributes.extend(attributes);
    }
}

/// A set of ABAC `Attribute`s
pub type Attributes = BTreeMap<Key, Value>;

/// An ABAC `Attribute`
///
/// ABAC attributes are tuples consisting of a string representing the
/// attribute name and the [`Value`] of the attribute.
pub type Attribute = (Key, Value);

/// A `Key` for an attribute `Value` in a set of `Attributes`
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Key(String);

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key(s.to_string())
    }
}

impl From<&Key> for String {
    fn from(key: &Key) -> Self {
        key.0.clone()
    }
}

impl core::ops::Deref for Key {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Primitive value types used to construct ABAC attributes and
/// conditionals.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Value {
    /// A string
    S(String),
    /// A signed integer
    I(i64),
    /// A boolean
    B(bool),
}

/// Create a new ABAC [`Value::S`] string value.
pub fn string<S: Into<String>>(s: S) -> Value {
    Value::S(s.into())
}

/// Create a new ABAC [`Value::I`] signed integer value.
pub fn int(n: i64) -> Value {
    Value::I(n)
}

/// Create a new ABAC [`Value::B`] boolean value.
pub fn bool(b: bool) -> Value {
    Value::B(b)
}
