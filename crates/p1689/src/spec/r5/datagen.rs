// TODO:
// 1. Generate more realistic paths (generate dir hierarchy)
// 2. Make paths correspond properly with `lookup-method`
// 3. Consider generating actual C++ source files
//   - This should be easy since the modules can be empty, aside from the imports/exports.

#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::iter_nth_zero)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::println_empty_string)]
#![allow(clippy::range_minus_one)]
#![allow(clippy::type_complexity)]

use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
};

use camino::Utf8PathBuf;
use indexmap::IndexSet;
use petgraph::graphmap::GraphMap;

use crate::r5;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxResult<T> = Result<T, BoxError>;

pub struct GraphGenerator {
    node_count: u8,
    names_edges: Box<dyn Iterator<Item = String>>,
    info_mem: BTreeMap<u8, r5::DepInfo<'static>>,
    graph: GraphMap<u8, String, petgraph::Directed>,
    known_producers: BTreeSet<u8>,
    known_consumers: BTreeSet<u8>,
}
impl GraphGenerator {
    pub fn new(node_count: u8) -> BoxResult<Self> {
        let mut names_nodes = names::Generator::default().filter_map(|name| name.split('-').nth(1).map(String::from));
        let names_edges = names::Generator::default().filter_map(|name| name.split('-').nth(0).map(String::from));
        let mut info_mem = BTreeMap::default();
        for id in 0 ..= node_count {
            let primary_output = names_nodes.next().map(|name| {
                let path = std::format!("{name}.o");
                let path = Utf8PathBuf::from(path);
                Cow::Owned(path)
            });
            let info = r5::DepInfo {
                work_directory: Option::default(),
                primary_output,
                outputs: IndexSet::default(),
                provides: IndexSet::default(),
                requires: IndexSet::default(),
            };
            info_mem.insert(id, info);
        }
        Ok(Self {
            node_count,
            names_edges: Box::new(names_edges),
            info_mem,
            graph: GraphMap::default(),
            known_producers: BTreeSet::default(),
            known_consumers: BTreeSet::default(),
        })
    }

    fn gen_dst(&self, u: &mut arbitrary::Unstructured) -> BoxResult<u8> {
        let dst = if !self.known_consumers.is_empty() && self.known_consumers.len() > 4 && u.ratio(1u8, 2u8)? {
            *self
                .known_consumers
                .iter()
                .nth(u.int_in_range(0 ..= self.known_producers.len() - 1)?)
                .ok_or("indexing failed")?
        } else {
            u.int_in_range(1u8 ..= self.node_count)?
        };
        Ok(dst)
    }

    fn gen_src(&self, u: &mut arbitrary::Unstructured, dst: u8) -> BoxResult<u8> {
        let src = if !self.known_producers.is_empty() && self.known_producers.len() > 4 && u.ratio(3u8, 4u8)? {
            *self
                .known_producers
                .iter()
                .nth(u.int_in_range(0 ..= self.known_producers.len() - 1)?)
                .ok_or("indexing failed")?
        } else {
            u.int_in_range(0u8 ..= dst - 1)?
        };
        Ok(src)
    }

    fn gen_provided_desc_fresh(
        &mut self,
        u: &mut arbitrary::Unstructured,
        src: u8,
    ) -> BoxResult<r5::ProvidedModuleDesc<'static>> {
        let module_name = self.names_edges.next().ok_or("name generation failed")?;
        let source_path = if u.arbitrary()? {
            None
        } else {
            let primary_output = &self
                .info_mem
                .get(&src)
                .ok_or("lookup failed")?
                .primary_output
                .as_deref()
                .and_then(|path| path.as_str().strip_suffix(".o"))
                .unwrap_or("");
            let name = std::format!("{primary_output}.cpp");
            let path = Utf8PathBuf::from(name);
            Some(Cow::Owned(path))
        };
        let unique_on_source_path = u.arbitrary()?;
        let compiled_module_path = Option::default();
        let logical_name = Cow::Owned(module_name);
        let desc = match source_path {
            Some(source_path) if unique_on_source_path => r5::ModuleDesc::BySourcePath {
                source_path,
                compiled_module_path,
                logical_name,
                unique_on_source_path: monostate::MustBeBool::<true>,
            },
            _ => r5::ModuleDesc::ByLogicalName {
                source_path,
                compiled_module_path,
                logical_name,
                unique_on_source_path: if u.arbitrary()? {
                    None
                } else {
                    Some(monostate::MustBeBool::<false>)
                },
            },
        };
        let is_interface = u.arbitrary()?;
        let provided = r5::ProvidedModuleDesc { desc, is_interface };
        Ok(provided)
    }

    fn gen_provided_desc(
        &mut self,
        u: &mut arbitrary::Unstructured,
        src: u8,
    ) -> BoxResult<r5::ProvidedModuleDesc<'static>> {
        if let Some(desc) = self.info_mem.get(&src) {
            if !desc.provides.is_empty() && u.ratio(3u8, 4u8)? {
                let provided = desc
                    .provides
                    .iter()
                    .nth(u.int_in_range(0 ..= desc.provides.len() - 1)?)
                    .ok_or("indexing failed")?
                    .clone();
                return Ok(provided);
            }
        }
        self.gen_provided_desc_fresh(u, src)
    }

    fn add_edge(
        &mut self,
        u: &mut arbitrary::Unstructured,
        src: u8,
        dst: u8,
        provided_desc: r5::ProvidedModuleDesc<'static>,
    ) -> BoxResult<()> {
        self.graph
            .add_edge(src, dst, provided_desc.desc.view().logical_name.to_owned());
        self.known_producers.insert(src);
        self.known_consumers.insert(dst);
        self.info_mem
            .get_mut(&dst)
            .ok_or("lookup failed")?
            .requires
            .insert(r5::RequiredModuleDesc {
                desc: provided_desc.desc.clone(),
                lookup_method: u.arbitrary()?,
            });
        self.info_mem
            .get_mut(&src)
            .ok_or("lookup failed")?
            .provides
            .insert(provided_desc);
        Ok(())
    }

    pub fn run(
        mut self,
        u: &mut arbitrary::Unstructured,
    ) -> BoxResult<(
        BTreeMap<u8, r5::DepInfo<'static>>,
        GraphMap<u8, String, petgraph::Directed>,
    )> {
        let mut i = 0;
        while i < self.node_count {
            let dst = self.gen_dst(u)?;
            let src = self.gen_src(u, dst)?;
            let provided_desc = self.gen_provided_desc(u, src)?;
            self.add_edge(u, src, dst, provided_desc)?;
            i += 1;
        }

        Ok((self.info_mem, self.graph))
    }
}

#[cfg(test)]
mod test {
    use super::BoxResult;

    #[cfg(feature = "serialize")]
    #[test]
    fn test() -> BoxResult<()> {
        use rand::RngCore;

        use super::*;

        let mut rng = rand::thread_rng();
        let mut bytes = alloc::vec![0u8; 8192];
        rng.fill_bytes(&mut bytes);
        let mut u = arbitrary::Unstructured::new(&bytes);

        let node_count = u.int_in_range(0u8 ..= 16u8)?;

        let generator = GraphGenerator::new(node_count)?;
        let (info_mem, graph) = generator.run(&mut u)?;

        for key in info_mem.keys().copied() {
            for (src, dst, weight) in graph.edges(key) {
                let name_src = info_mem.get(&src).unwrap().primary_output.as_deref().unwrap();
                let name_dst = info_mem.get(&dst).unwrap().primary_output.as_deref().unwrap();
                std::println!("{name_src}::{src} -[ {weight} ]-> {name_dst}::{dst}");
            }
        }

        let select = u.ratio(1u8, 4u8).unwrap();
        let rules = info_mem
            .into_values()
            .filter(|desc| !(desc.provides.is_empty() && desc.requires.is_empty()) || select)
            .collect();
        let dep_file = r5::DepFile {
            version: 1,
            revision: None,
            rules,
        };

        let str = serde_json::to_string_pretty(&dep_file).unwrap();
        std::println!("{str}");

        Ok(())
    }
}
