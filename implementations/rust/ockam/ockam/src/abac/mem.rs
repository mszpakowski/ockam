//! An in-memory ABAC backend implementation.

use core::fmt::{self, Debug, Formatter};

use super::error::AbacError;
use super::{
    AbacAttributeStorage, AbacAuthorization, AbacPolicyStorage, Action, Attribute, Attributes,
    Conditional, Identity, Key, Resource, Subject, Value,
};
use ockam_core::Result;
use ockam_core::{
    async_trait,
    compat::{boxed::Box, collections::BTreeMap, sync::Arc, sync::RwLock},
};

/// `Memory` is an in-memory ABAC backend implementation for use by
/// tests and code examples.
#[derive(Default)]
pub struct Memory {
    /// [`Inner`] implementation of the ABAC traits
    pub(crate) inner: Arc<RwLock<Inner>>,
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Memory")
    }
}

impl Memory {
    /// Create a new `Memory` ABAC backend.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::new())),
        }
    }
}

/// `Inner` provides implementations of the [`AbacAttributeStorage`],
/// [`AbacAuthorization`] and [`AbacPolicyStorage`] trait methods for
/// use by the [`Memory`] ABAC backend.
#[derive(Default)]
pub struct Inner {
    /// subject maps to a set of key-value attributes
    subjects: BTreeMap<Identity, BTreeMap<Key, Value>>,
    /// policies map a resource to a set of actions subject to conditions
    policies: BTreeMap<Resource, BTreeMap<Action, Conditional>>,
}

impl Inner {
    /// Create a new `Inner`.
    fn new() -> Self {
        Inner::default()
    }

    /// Implementation for [`AbacAttributeStorage::del_subject`]
    fn del_subject(&mut self, s: &Subject) {
        self.subjects.remove(s.identifier());
    }

    /// Implementation for [`AbacAttributeStorage::get_subject_attributes`]
    fn get_subject_attributes(&self, subject: &Subject) -> BTreeMap<Key, Value> {
        if let Some(attributes) = self.subjects.get(subject.identifier()) {
            attributes.clone()
        } else {
            BTreeMap::default()
        }
    }

    /// Implementation for [`AbacAttributeStorage::set_subject`]
    fn set_subject<A>(&mut self, subject: Subject, attributes: A)
    where
        A: IntoIterator<Item = Attribute>,
    {
        self.subjects.insert(
            subject.identifier().clone(),
            attributes.into_iter().collect(),
        );
    }

    /// Implementation for [`AbacPolicyStorage::del_policy`]
    fn del_policy(&mut self, resource: &Resource) {
        self.policies.remove(resource);
    }

    /// Implementation for [`AbacPolicyStorage::get_policy`]
    fn get_policy(&self, resource: &Resource, action: &Action) -> Option<Conditional> {
        self.policies
            .get(resource)
            .and_then(|p| p.get(action))
            .cloned()
    }

    /// Implementation for [`AbacPolicyStorage::set_policy`]
    fn set_policy(&mut self, resource: Resource, action: Action, policy: &Conditional) {
        self.policies
            .entry(resource)
            .or_insert_with(BTreeMap::new)
            .insert(action, policy.clone());
    }

    /// Implementation for [`AbacAuthorization::is_authorized`]
    fn is_authorized(&self, subject: &Subject, resource: &Resource, action: &Action) -> bool {
        if let Some(attributes) = self.subjects.get(subject.identifier()) {
            if let Some(policy) = self.get_policy(resource, action) {
                return policy.apply(attributes);
            }
        }
        false
    }
}

#[async_trait]
impl AbacAttributeStorage for Memory {
    async fn del_subject(&self, subject: &Subject) -> Result<()> {
        match self.inner.write() {
            Ok(mut mem) => {
                mem.del_subject(subject);
                Ok(())
            }
            Err(_) => Err(AbacError::Write.into()),
        }
    }

    async fn get_subject_attributes(&self, subject: &Subject) -> Result<Attributes> {
        match self.inner.read() {
            Ok(mem) => Ok(mem.get_subject_attributes(subject)),
            Err(_) => Err(AbacError::Write.into()),
        }
    }

    async fn set_subject<A>(&self, subject: Subject, attrs: A) -> Result<()>
    where
        A: IntoIterator<Item = Attribute> + Send + 'static,
    {
        match self.inner.write() {
            Ok(mut mem) => {
                mem.set_subject(subject, attrs);
                Ok(())
            }
            Err(_) => Err(AbacError::Write.into()),
        }
    }
}

#[async_trait]
impl AbacPolicyStorage for Memory {
    async fn del_policy(&self, resource: &Resource) -> Result<()> {
        match self.inner.write() {
            Ok(mut mem) => {
                mem.del_policy(resource);
                Ok(())
            }
            Err(_) => Err(AbacError::Write.into()),
        }
    }

    /// Return the [`Conditional`] policy entry for a given ABAC
    /// [`Resource`] and [`Action`] .
    async fn get_policy(
        &self,
        resource: &Resource,
        action: &Action,
    ) -> Result<Option<Conditional>> {
        match self.inner.read() {
            Ok(mem) => Ok(mem.get_policy(resource, action)),
            Err(_) => Err(AbacError::Read.into()),
        }
    }

    async fn set_policy(
        &self,
        resource: Resource,
        action: Action,
        policy: &Conditional,
    ) -> Result<()> {
        match self.inner.write() {
            Ok(mut mem) => {
                mem.set_policy(resource, action, policy);
                Ok(())
            }
            Err(_) => Err(AbacError::Write.into()),
        }
    }
}

#[async_trait]
impl AbacAuthorization for Memory {
    async fn is_authorized(
        &self,
        subject: &Subject,
        resource: &Resource,
        action: &Action,
    ) -> Result<bool> {
        match self.inner.read() {
            Ok(mem) => Ok(mem.is_authorized(subject, resource, action)),
            Err(_) => Err(AbacError::Read.into()),
        }
    }
}

// TODO lose this
impl Memory {
    /// Populates Memory with some policy data for testing
    pub async fn populate_with_policy_test_data(&mut self) -> Result<()> {
        use crate::abac::{self, Method};

        // Set up some conditionals on attributes
        let project_green = abac::eq("project", abac::string("green"));
        let project_blue = abac::eq("project", abac::string("blue"));
        let role_reader = abac::eq("role", abac::string("reader"));
        let role_writer = abac::eq("role", abac::string("writer"));

        // Define some policies
        self.set_policy(
            Resource::from("/project/green/1234"),
            Action::from("read"),
            &project_green.and(&role_reader.or(&role_writer)),
        )
        .await?;

        self.set_policy(
            Resource::from("/project/green/1234"),
            Action::from("write"),
            &project_green.and(&role_writer),
        )
        .await?;

        self.set_policy(
            Resource::from("/project/blue/5678"),
            Action::from("write"),
            &project_blue.and(&role_writer),
        )
        .await?;

        let mut resource = Resource::from("/echoer");
        resource.extend([("space".into(), abac::string("some_customer_space"))]);
        self.set_policy(
            resource,
            Action::from(Method::Post),
            &project_green.and(&role_reader.or(&role_writer)),
        )
        .await?;

        Ok(())
    }

    /// Populates Memory with some subject data for testing
    pub async fn populate_with_subject_test_data(&mut self) -> Result<()> {
        use crate::abac;

        // Set up some subjects with attributes
        self.set_subject(
            Subject::from(0x0000_0000_0000_0001),
            [
                ("role".into(), abac::string("reader")),
                ("project".into(), abac::string("green")),
            ],
        )
        .await?;

        self.set_subject(
            Subject::from(0x0000_0000_0000_0002),
            [
                ("role".into(), abac::string("writer")),
                ("project".into(), abac::string("green")),
            ],
        )
        .await?;

        self.set_subject(
            Subject::from(0x0000_0000_0000_0003),
            [
                ("role".into(), abac::string("writer")),
                ("project".into(), abac::string("blue")),
            ],
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::abac::mem::Memory;
    use crate::abac::{eq, gt, int, string, Action, Resource, Subject};

    #[test]
    fn example1() {
        let is_adult = gt("age", int(17));
        let is_john = eq("name", string("John"));
        let condition = is_adult.or(&is_john);

        let read = Action::from("r");
        let resource = Resource::from("/foo/bar/baz");

        let mem = Memory::new();
        mem.inner
            .write()
            .unwrap()
            .set_policy(resource.clone(), read.clone(), &condition);
        mem.inner.write().unwrap().set_subject(
            Subject::from(1),
            [("name".into(), string("John")), ("age".into(), int(25))],
        );
        mem.inner.write().unwrap().set_subject(
            Subject::from(2),
            [
                ("name".into(), string("Jack")),
                ("age".into(), int(12)),
                ("city".into(), string("London")),
            ],
        );
        mem.inner.write().unwrap().set_subject(
            Subject::from(3),
            [("name".into(), string("Bill")), ("age".into(), int(32))],
        );

        assert!(mem
            .inner
            .read()
            .unwrap()
            .is_authorized(&Subject::from(1), &resource, &read)); // John
        assert!(mem
            .inner
            .read()
            .unwrap()
            .is_authorized(&Subject::from(3), &resource, &read)); // adult
        assert!(!mem
            .inner
            .read()
            .unwrap()
            .is_authorized(&Subject::from(2), &resource, &read)); // not John and no adult
    }
}
