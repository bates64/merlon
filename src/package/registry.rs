use std::collections::HashSet;

use anyhow::{Result, bail};

use super::{Package, Id};

/// A package registry. This is an arena of packages.
/// Allows for querying packages by name, uuid, etc., and dependency queries.
#[derive(Debug, Default)]
pub struct Registry {
    packages: HashSet<Package>, // TODO: consider BTreeSet where most-dependent packages are first
}

impl Registry {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }

    /// Add a package to the registry.
    /// Returns an error if the package is already in the registry.
    /// Returns the package's ID so it can be used to refer to the package.
    pub fn register(&mut self, package: Package) -> Result<Id> {
        if self.packages.contains(&package) {
            anyhow::bail!("package already in registry");
        }
        let id = package.id()?;
        self.packages.insert(package);
        Ok(id)
    }

    /// Remove a package from the registry.
    /// Returns an error if the package is not in the registry.
    pub fn unregister(&mut self, package: &Package) -> Result<()> {
        if !self.packages.contains(package) {
            anyhow::bail!("package not in registry");
        }
        // TODO: check if any packages depend on this one
        self.packages.remove(package);
        Ok(())
    }

    /// Get a package by ID.
    pub fn get(&self, id: Id) -> Option<&Package> {
        // TODO: use hashmap id->package instead of iterating
        self.packages.iter().find(|pkg| pkg.id().map(|pkg_id| pkg_id == id).unwrap_or(false))
    }
}

// Queries. Note they talk in IDs, not a &Package, to satisfy the borrow checker.
impl Registry {
    /// Iterates over the direct dependency packages of a package.
    pub fn get_direct_dependencies(&self, id: Id) -> Result<HashSet<Id>> { 
        let package = match self.get(id) {
            Some(package) => package,
            None => anyhow::bail!("package {id} not found in registry"),
        };
        let manifest = package.manifest()?;
        Ok(manifest.iter_direct_dependencies()
            .map(|dep| dep.id().clone()) // Clone so we can drop manifest
            .collect())
    }

    /// Iterates over all dependencies of a package, including both direct and transitive dependencies.
    pub fn get_dependencies(&self, id: Id) -> Result<HashSet<Id>> {
        // Depth first search
        let mut dependencies = HashSet::new();
        let mut stack: Vec<Id> = self.get_direct_dependencies(id)?
            .into_iter()
            .collect();
        while let Some(popped_id) = stack.pop() {
            if popped_id == id {
                bail!("found circular dependency");
            }
            if dependencies.contains(&popped_id) {
                // TODO: check version
                continue;
            }
            dependencies.insert(popped_id.clone());
            for dependency in self.get_direct_dependencies(popped_id)? {
                stack.push(dependency);
            }
        }
        Ok(dependencies)
    }

    /// Returns true if a package has a dependency - transitive or direct - on another package.
    pub fn has_dependency(&self, id: Id, dependency_id: Id) -> Result<bool> {
        let dependencies = self.get_dependencies(id)?;
        Ok(dependencies.contains(&dependency_id))
    }

    /// Adds a direct dependency to a package.
    pub fn add_direct_dependency(&mut self, id: Id, dependency_id: Id) -> Result<()> {
        let package = match self.get(id) {
            Some(package) => package,
            None => anyhow::bail!("package {id} not found in registry"),
        };
        // TODO: checks
        package.edit_manifest(|manifest| {
            manifest.declare_direct_dependency(dependency_id)
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use temp_dir::TempDir;
    use anyhow::Result;
    use super::{Registry, Package, Id};

    #[test]
    fn dependency_graph() -> Result<()> {
        let dir = TempDir::new()?;
        let mut registry = Registry::new();

        // Create a graph of four packages: Base, A, B, and C.
        let base = Package::new("Base", dir.path().join("base.merlon"))?;
        let a = Package::new("A", dir.path().join("a.merlon"))?;
        let b = Package::new("B", dir.path().join("b.merlon"))?;
        let c = Package::new("C", dir.path().join("c.merlon"))?;
        let base = registry.register(base)?;
        let a = registry.register(a)?;
        let b = registry.register(b)?;
        let c = registry.register(c)?;

        // Print IDs for debugging.
        dbg!(&base, &a, &b, &c);

        // Both A and B directly depend on C.
        registry.add_direct_dependency(a, b)?;
        registry.add_direct_dependency(b, c)?;

        // All depend directly on base.
        for parent in [a, b, c] {
            registry.add_direct_dependency(parent, base)?;
        }

        // Assert A depends on B, C, and base only.
        let deps = registry.get_dependencies(a)?;
        let expected: HashSet<Id> = vec![b, c, base].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert B depends on C and base only.
        let deps = registry.get_dependencies(b)?;
        let expected: HashSet<Id> = vec![c, base].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert C depends on base only.
        let deps = registry.get_dependencies(c)?;
        let expected: HashSet<Id> = vec![base].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert base has no dependencies.
        let base_deps = registry.get_dependencies(base)?;
        assert!(base_deps.is_empty());

        Ok(())
    }
}
