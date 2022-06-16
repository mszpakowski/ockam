use crate::abac::policy::Conditional;
use crate::abac::types::*;

use ockam_core::Result;
use ockam_core::{async_trait, compat::boxed::Box};

/// The `AbacAttributeStorage` trait provides an interface for the
/// persistance and retrieval of ABAC attributes.
///
/// `AbacAttributeStorage` follows the logical schema:
///
/// > [`Subject`]  ||--|{ [`Attribute`] ||--|| (`key`, [`Value`])
///
/// ([schema syntax](https://mermaid-js.github.io/mermaid/#/entityRelationshipDiagram?id=syntax))
#[async_trait]
pub trait AbacAttributeStorage {
    /// Delete all attributes for the given ABAC [`Subject`].
    async fn del_subject(&self, s: &Subject) -> Result<()>;

    /// Return all attributes for the given ABAC [`Subject`].
    async fn get_subject_attributes(&self, s: &Subject) -> Result<Attributes>;

    /// Set the attributes for a given ABAC [`Subject`].
    ///
    /// Any pre-existing attribute entries for the subject will be
    /// replaced.
    async fn set_subject<I>(&self, s: Subject, attributes: I) -> Result<()>
    where
        I: IntoIterator<Item = Attribute> + Send + 'static;
}

/// The `AbacAuthorization` trait provides an interface for making an
/// authorization decision based on a given [`Subject`], [`Resource`],
/// [`Action`] request triple.
#[async_trait]
pub trait AbacAuthorization {
    /// Perform an authorization check for the given ABAC [`Subject`],
    /// [`Resource`] and [`Action`].
    async fn is_authorized(&self, s: &Subject, r: &Resource, a: &Action) -> Result<bool>;
}

/// The `AbacPolicyStorage` trait provides an interface for the
/// persistance and retrieval of ABAC policies.
///
/// `AbacPolicyStorage` follows the logical schema:
///
/// > [`Resource`] ||--|{ `policy` ||--|| ([`Action`], [`Conditional`])
///
/// ([schema syntax](https://mermaid-js.github.io/mermaid/#/entityRelationshipDiagram?id=syntax))
#[async_trait]
pub trait AbacPolicyStorage {
    /// Delete all policy entries associated with the given ABAC
    /// [`Resource`].  [`Resource`].
    async fn del_policy(&self, r: &Resource) -> Result<()>;

    /// Return the [`Conditional`] policy entry for a given ABAC
    /// [`Resource`] and [`Action`] .
    async fn get_policy(&self, r: &Resource, a: &Action) -> Result<Option<Conditional>>;

    /// Set a [`Conditional`] policy entry for a given ABAC
    /// [`Resource`] and [`Action`] .
    ///
    /// Any pre-existing [`Action`] entries associated with the
    /// [`Resource`] will be replaced.
    async fn set_policy(&self, r: Resource, a: Action, c: &Conditional) -> Result<()>;
}
