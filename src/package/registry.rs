use std::collections::{HashSet, HashMap};

use anyhow::{Result, bail};

use super::{Package, Id, manifest::{Dependency, Version}};

/// A package registry. This is an arena of packages.
/// Allows for querying packages by name, uuid, etc., and dependency queries.
#[derive(Debug, Default)]
pub struct Registry {
    packages: HashMap<Id, Package>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            packages: Default::default(),
        }
    }

    /// Add a package to the registry.
    /// Returns an error if the package is already in the registry.
    /// Returns the package's ID so it can be used to refer to the package.
    pub fn register(&mut self, package: Package) -> Result<Id> {
        let id = package.id()?;
        if self.packages.contains_key(&id) {
            bail!("package {} already in registry", package);
        }
        self.packages.insert(id, package);
        Ok(id)
    }

    /// Remove a package from the registry.
    /// Returns an error if the package is not in the registry.
    pub fn take(&mut self, id: Id) -> Result<Package> {
        match self.packages.remove(&id) {
            Some(package) => Ok(package),
            None => bail!("package {} not in registry", id),
        }
    }

    /// Get a package by ID.
    pub fn get(&self, id: Id) -> Option<&Package> {
        self.packages.get(&id)
    }

    pub fn get_or_error(&self, id: Id) -> Result<&Package> {
        match self.get(id) {
            Some(package) => Ok(package),
            None => bail!("package {id} not found in registry"),
        }
    }

    pub fn edit<F, T>(&mut self, id: Id, f: F) -> Result<T>
    where
        F: FnOnce(&mut Package) -> Result<T>
    {
        let mut package = self.take(id)?;
        let ret = f(&mut package)?;
        self.register(package)?;
        Ok(ret)
    }
}

// Queries. Note they talk in IDs, not a &Package, to satisfy the borrow checker.
impl Registry {
    /// Iterates over the direct dependency packages of a package.
    pub fn get_direct_dependencies(&self, id: Id) -> Result<HashSet<Dependency>> { 
        let package = self.get_or_error(id)?;
        let manifest = package.manifest()?;
        Ok(manifest.iter_direct_dependencies()
            .map(|dep| dep.clone()) // Clone so we can drop manifest
            .collect())
    }

    /// Iterates over all dependencies of a package, including both direct and transitive dependencies.
    pub fn get_dependencies(&self, id: Id) -> Result<HashSet<Dependency>> {
        // Breadth first search
        let mut dependencies = HashSet::new();
        let mut stack: Vec<Dependency> = self.get_direct_dependencies(id)?
            .into_iter()
            .collect();
        while let Some(popped_dep) = stack.pop() {
            if dependencies.contains(&popped_dep) {
                continue;
            }
            if let Dependency::Package { id: dep_id, .. } = &popped_dep {
                if *dep_id == id {
                    bail!("found circular dependency");
                }
                for dependency in self.get_direct_dependencies(*dep_id)? {
                    stack.push(dependency);
                }
            }
            dependencies.insert(popped_dep);
        }
        Ok(dependencies)
    }

    /// Returns true if a package has a dependency - transitive or direct - on another package.
    pub fn has_dependency(&self, id: Id, dependency_id: Id) -> Result<bool> {
        let dependencies = self.get_dependencies(id)?;
        Ok(dependencies.iter().any(|dep| {
            if let Dependency::Package { id: dep_id, .. } = dep {
                *dep_id == dependency_id
            } else {
                false
            }
        }))
    }

    /// Adds a direct dependency to a package.
    /// Both the package and the dependency must be registered.
    pub fn add_direct_dependency(&mut self, id: Id, dependency_id: Id) -> Result<()> {
        let package = self.get_or_error(id)?;
        let dependency_package = self.get_or_error(dependency_id)?;
        let dependency_manifest = dependency_package.manifest()?;
        let dependency_metadata = dependency_manifest.metadata();
        let dependency = dependency_metadata.into();
        package.edit_manifest(|manifest| {
            manifest.declare_direct_dependency(dependency)
        })
    }

    /// Returns the set of all dependencies across all packages in the registry.
    pub fn all_dependencies(&self) -> Result<HashSet<Dependency>> {
        let mut dependencies = HashSet::new();
        for (id, _) in self.packages.iter() {
            for dependency in self.get_dependencies(*id)? {
                dependencies.insert(dependency);
            }
        }
        Ok(dependencies)
    }

    pub fn package_version_map(&self) -> Result<HashMap<Id, Version>> {
        let mut map = HashMap::new();
        for (id, package) in self.packages.iter() {
            let id = *id;
            let manifest = package.manifest()?;
            let metadata = manifest.metadata();
            if metadata.id() != id {
                bail!("package id mismatch: {id}");
            }
            let version = metadata.version();
            if let Some(other_version) = map.insert(id, version.clone()) {
                if other_version != *version {
                    bail!("package {id} has multiple versions: {version} and {other_version}");
                }
            }
        }
        Ok(map)
    }

    /// Returns an error if packages exist with incompatible versions.
    /// For example, if package A depends on package B ^1.0.0, but package B is registered as 2.0.0, its an error.
    pub fn check_version_compatibility(&self) -> Result<()> {
        let map = self.package_version_map()?;
        for dependency in self.all_dependencies()? {
            if let Dependency::Package { id, version } = dependency {
                match map.get(&id) {
                    None => bail!("dependency exists for {id} {version}, but it is not in registry"),
                    Some(actual_version) => {
                        if !version.matches(actual_version) {
                            bail!(
                                "a package depends on {} {} which is incompatible with its actual version {}",
                                self.get(id).unwrap(), // unwrap: if its in map, its in registry
                                version,
                                actual_version,
                            );
                        }
                    }
                }
                
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use temp_dir::TempDir;
    use anyhow::Result;

    use super::{Registry, Package, Dependency, Version};

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

        let base_as_dep: Dependency = registry.get_or_error(base)?.try_into()?;
        let b_as_dep: Dependency = registry.get_or_error(b)?.try_into()?;
        let c_as_dep: Dependency = registry.get_or_error(c)?.try_into()?;

        // Assert A depends on B, C, and base only.
        let deps = registry.get_dependencies(a)?;
        let expected: HashSet<Dependency> = vec![b_as_dep.clone(), c_as_dep.clone(), base_as_dep.clone()].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert B depends on C and base only.
        let deps = registry.get_dependencies(b)?;
        let expected: HashSet<Dependency> = vec![c_as_dep.clone(), base_as_dep.clone()].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert C depends on base only.
        let deps = registry.get_dependencies(c)?;
        let expected: HashSet<Dependency> = vec![base_as_dep.clone()].into_iter().collect();
        assert_eq!(deps, expected);

        // Assert base has no dependencies.
        let base_deps = registry.get_dependencies(base)?;
        assert!(base_deps.is_empty());

        // If we update base's major version, it should become incompatible
        registry.check_version_compatibility()?;
        registry.edit(base, |package| {
            package.edit_manifest(|manifest| {
                manifest.metadata_mut().set_version(Version::new(2, 0, 0));
                Ok(())
            })
        })?;
        assert!(registry.check_version_compatibility().is_err());

        Ok(())
    }
}
