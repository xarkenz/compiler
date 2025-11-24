use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::sema::{NamespaceHandle, SimplePath, TypeHandle};

mod context;
pub use context::*;

fn package_file_error(cause: impl ToString) -> Box<crate::Error> {
    Box::new(crate::Error::new(
        None,
        crate::ErrorKind::PackageFile {
            cause: cause.to_string(),
        },
    ))
}

#[derive(Clone, Debug)]
pub struct PackageDependency {
    name: Box<str>,
    path: Box<Path>,
}

impl PackageDependency {
    pub fn new(name: Box<str>, path: Box<Path>) -> Self {
        Self {
            name,
            path,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn parse_toml(parent_dir: &Path, name: Box<str>, table: &toml::Table) -> crate::Result<Self> {
        let path = table.get("path")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| package_file_error(format!("missing or invalid field: dependency.{name}.path")))?;
        let path = parent_dir.join(path)
            .canonicalize()
            .map_err(package_file_error)?;

        Ok(Self::new(
            name,
            path.into_boxed_path(),
        ))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PackageKind {
    Library,
    Executable,
}

impl PackageKind {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "lib" => Some(Self::Library),
            "exe" => Some(Self::Executable),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum PackageVisitMarker {
    NotVisited,
    InProgress,
    Visited,
}

#[derive(Debug)]
pub struct PackageInfo {
    path: Box<Path>,
    name: Box<str>,
    kind: PackageKind,
    main_path: Box<Path>,
    dependencies: Box<[PackageDependency]>,
    visit_marker: Cell<PackageVisitMarker>,
}

impl PackageInfo {
    pub fn new(path: Box<Path>, name: Box<str>, kind: PackageKind, main_path: Box<Path>, dependencies: Box<[PackageDependency]>) -> Self {
        Self {
            path,
            name,
            kind,
            main_path,
            dependencies,
            visit_marker: Cell::new(PackageVisitMarker::NotVisited),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> PackageKind {
        self.kind
    }

    pub fn main_path(&self) -> &Path {
        &self.main_path
    }

    pub fn dependencies(&self) -> &[PackageDependency] {
        &self.dependencies
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.path.join(format!("out{}{}.ll", std::path::MAIN_SEPARATOR, self.name))
    }

    fn parse_package_toml(parent_dir: impl AsRef<Path>) -> crate::Result<Self> {
        let parent_dir = parent_dir.as_ref();
        if !parent_dir.is_dir() {
            return Err(package_file_error(format!("invalid package directory: {}", parent_dir.display())));
        }

        let package_info_path = parent_dir.join("package.toml");
        let table_raw = std::fs::read_to_string(&package_info_path)
            .map_err(package_file_error)?;
        let table = table_raw.parse::<toml::Table>()
            .map_err(package_file_error)?;

        let package_table = table.get("package")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| package_file_error("missing [package] section"))?;
        let name = package_table.get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| package_file_error("missing or invalid field: package.name"))?;
        let kind = package_table.get("kind")
            .and_then(toml::Value::as_str)
            .and_then(PackageKind::from_key)
            .ok_or_else(|| package_file_error("missing or invalid field: package.kind"))?;
        let main_path = package_table.get("main_path")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| package_file_error("missing or invalid field: package.main_path"))?;
        let main_path = parent_dir.join(main_path)
            .canonicalize()
            .map_err(package_file_error)?;

        let mut dependencies = Vec::new();
        if let Some(toml::Value::Table(dependency_table)) = table.get("dependency") {
            for (dependency_name, dependency_info) in dependency_table {
                let dependency_info = dependency_info
                    .as_table()
                    .ok_or_else(|| package_file_error(format!("dependency listing '{dependency_name}' is invalid")))?;
                dependencies.push(PackageDependency::parse_toml(
                    parent_dir,
                    dependency_name.clone().into_boxed_str(),
                    dependency_info,
                )?);
            }
        }

        Ok(Self::new(
            parent_dir.into(),
            name.into(),
            kind,
            main_path.into_boxed_path(),
            dependencies.into_boxed_slice(),
        ))
    }
}

pub struct PackageManager {
    package_registry: HashMap<Box<str>, Rc<PackageInfo>>,
    compile_queue: VecDeque<Box<str>>,
}

impl PackageManager {
    pub fn generate(current_path: impl AsRef<Path>) -> crate::Result<Self> {
        let current_path = current_path.as_ref()
            .canonicalize()
            .map_err(package_file_error)?;

        // Gather all required packages using a breadth-first search.
        let mut package_registry: HashMap<Box<str>, Rc<PackageInfo>> = HashMap::new();
        let mut frontier: VecDeque<Box<Path>> = VecDeque::new();
        frontier.push_back(current_path.into_boxed_path());

        while let Some(parent_dir) = frontier.pop_front() {
            let package_info = PackageInfo::parse_package_toml(&parent_dir)?;
            if let Some(existing_package_info) = package_registry.get(package_info.name()) {
                if package_info.path() != existing_package_info.path() {
                    return Err(package_file_error(format!("dependencies include multiple packages named '{}'", package_info.name())));
                }
                continue;
            }

            for dependency in package_info.dependencies() {
                frontier.push_back(dependency.path().into());
            }

            package_registry.insert(package_info.name().into(), Rc::new(package_info));
        }

        // Order the packages in the compile queue using a topological sort so that a package is
        // only compiled after all of its dependencies are compiled.
        // Based on https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search.
        let mut compile_queue = VecDeque::new();
        while let Some(next_package) = package_registry
            .values()
            .find(|package| package.visit_marker.get() == PackageVisitMarker::NotVisited)
        {
            visit_package(next_package, &package_registry, &mut compile_queue)?;
        }

        fn visit_package(
            package: &PackageInfo,
            package_registry: &HashMap<Box<str>, Rc<PackageInfo>>,
            compile_stack: &mut VecDeque<Box<str>>,
        ) -> crate::Result<()> {
            match package.visit_marker.get() {
                PackageVisitMarker::Visited => {
                    Ok(())
                }
                PackageVisitMarker::InProgress => {
                    Err(package_file_error(format!("cyclic dependency detected for package '{}'", package.name())))
                }
                PackageVisitMarker::NotVisited => {
                    package.visit_marker.set(PackageVisitMarker::InProgress);

                    for dependency in package.dependencies() {
                        visit_package(&package_registry[dependency.name()], package_registry, compile_stack)?;
                    }

                    package.visit_marker.set(PackageVisitMarker::Visited);
                    compile_stack.push_back(package.name().into());

                    Ok(())
                }
            }
        }

        Ok(Self {
            package_registry,
            compile_queue,
        })
    }

    pub fn get_package(&self, name: &str) -> Option<Rc<PackageInfo>> {
        self.package_registry.get(name).cloned()
    }

    pub fn get_next_to_compile(&mut self) -> Option<Rc<PackageInfo>> {
        self.compile_queue.pop_front()
            .map(|name| self.package_registry[&name].clone())
    }
}
