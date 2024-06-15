#![cfg(not(tarpaulin_include))]
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
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

#[allow(clippy::wildcard_imports)]
use fake::{faker::name::raw::*, locales::*, Dummy};
use petgraph::graphmap::GraphMap;
use rand::prelude::*;

use super::BoxResult;
use crate::{r5, vendor::camino::Utf8PathBuf};

fn name_sep<R>(rng: &mut R) -> String
where
    R: RngCore,
{
    if rng.gen::<bool>() {
        let mut dst = [0u16; 2];
        let code = rng.gen_range(0x10000u32 .. 0x0010_ffff);
        let char = unsafe { char::from_u32_unchecked(code) };
        char.encode_utf16(&mut dst);
        alloc::format!("{:#06x}{:#06x}", dst[0], dst[1]).replace("0x", "\\u")
    } else {
        let code = {
            let codes = [0x0020 .. 0x007e, 0x00a0 .. 0xd7ff, 0xe000 .. 0xffff];
            let idx = rng.gen_range(0 .. 3);
            rng.gen_range(codes[idx].clone())
        };
        if rng.gen::<bool>() {
            alloc::format!("{}", unsafe { char::from_u32_unchecked(code) })
                .replace('{', "")
                .replace('}', "")
        } else {
            alloc::format!("{code:#06x}").replace("0x", "\\u")
        }
    }
}

fn fake_name<R>(rng: &mut R, more_escapes: bool) -> String
where
    R: RngCore,
{
    let name = match rng.gen_range(0_i32 .. 7_i32) {
        0_i32 => String::dummy_with_rng(&Name(AR_SA), rng),
        1_i32 => String::dummy_with_rng(&Name(EN), rng),
        2_i32 => String::dummy_with_rng(&Name(FR_FR), rng),
        3_i32 => String::dummy_with_rng(&Name(JA_JP), rng),
        4_i32 => String::dummy_with_rng(&Name(PT_BR), rng),
        5_i32 => String::dummy_with_rng(&Name(ZH_CN), rng),
        6_i32 => String::dummy_with_rng(&Name(ZH_TW), rng),
        _ => unreachable!(),
    };
    let mut res = String::new();
    for (i, frag) in name.split_whitespace().enumerate() {
        if i % 2 == 1 && i > 0 {
            if more_escapes {
                res.push_str(&name_sep(rng));
            } else {
                res.push('-');
            }
        }
        res.push_str(&frag.to_lowercase());
    }
    res
}

pub struct DepFileIterator<'r, R> {
    rng: &'r mut R,
    config: GraphGeneratorConfig,
}
impl<'r, R> DepFileIterator<'r, R> {
    pub fn new(rng: &'r mut R, config: GraphGeneratorConfig) -> Self {
        Self { rng, config }
    }
}

impl<'r, R> Iterator for DepFileIterator<'r, R>
where
    R: RngCore,
{
    type Item = BoxResult<r5::DepFile<'static>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(GraphGenerator::gen_dep_file(self.rng, self.config.clone()))
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default)]
pub struct GraphGeneratorConfig {
    node_count: u8,
    more_escapes: bool,
}
impl GraphGeneratorConfig {
    pub fn build<R>(self, rng: &mut R) -> GraphGenerator<R>
    where
        R: RngCore,
    {
        GraphGenerator::new(rng, self)
    }

    #[must_use]
    pub const fn node_count(mut self, node_count: u8) -> Self {
        self.node_count = node_count;
        self
    }

    #[must_use]
    pub const fn more_escapes(mut self, more_escapes: bool) -> Self {
        self.more_escapes = more_escapes;
        self
    }
}
#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct GraphGeneratorState {
    info_mem: BTreeMap<u8, r5::DepInfo<'static>>,
    graph: GraphMap<u8, String, petgraph::Directed>,
    known_producers: BTreeSet<u8>,
    known_consumers: BTreeSet<u8>,
}

#[allow(clippy::module_name_repetitions)]
pub struct GraphGenerator<'r, R> {
    rng: &'r mut R,
    config: GraphGeneratorConfig,
    state: GraphGeneratorState,
}
impl<'r, R> GraphGenerator<'r, R>
where
    R: RngCore,
{
    pub fn gen_dep_files(rng: &'r mut R, config: GraphGeneratorConfig) -> DepFileIterator<'r, R> {
        DepFileIterator::new(rng, config)
    }

    pub fn gen_dep_file(rng: &'r mut R, config: GraphGeneratorConfig) -> BoxResult<r5::DepFile<'static>> {
        let mut bytes = alloc::vec![0u8; 8192];
        rng.fill_bytes(&mut bytes);
        let mut u = arbitrary::Unstructured::new(&bytes);
        let gen = Self::new(rng, config);
        let (info_mem, _graph) = gen.run(&mut u)?;
        let select = u.ratio(1u8, 4u8)?;
        let rules = info_mem
            .into_values()
            .filter(|desc| !(desc.provides.is_empty() && desc.requires.is_empty()) || select)
            .collect();
        Ok(r5::DepFile {
            version: 1,
            revision: None,
            rules,
        })
    }

    fn new(rng: &'r mut R, config: GraphGeneratorConfig) -> Self {
        let mut info_mem = BTreeMap::default();
        for id in 0 ..= config.node_count {
            let primary_output = {
                let name = fake_name(rng, config.more_escapes);
                let path = std::format!("{name}.o");
                let path = Utf8PathBuf::from(path);
                Some(Cow::Owned(path))
            };
            let info = r5::DepInfo {
                work_directory: Option::default(),
                primary_output,
                outputs: Vec::default(),
                provides: Vec::default(),
                requires: Vec::default(),
            };
            info_mem.insert(id, info);
        }
        let state = GraphGeneratorState {
            info_mem,
            graph: GraphMap::default(),
            known_producers: BTreeSet::default(),
            known_consumers: BTreeSet::default(),
        };
        Self { rng, config, state }
    }

    fn gen_dst(&self, u: &mut arbitrary::Unstructured) -> BoxResult<u8> {
        let dst =
            if !self.state.known_consumers.is_empty() && self.state.known_consumers.len() > 4 && u.ratio(1u8, 2u8)? {
                *self
                    .state
                    .known_consumers
                    .iter()
                    .nth(u.int_in_range(0 ..= self.state.known_consumers.len() - 1)?)
                    .ok_or("indexing failed0")?
            } else {
                u.int_in_range(1u8 ..= self.config.node_count)?
            };
        Ok(dst)
    }

    fn gen_src(&self, u: &mut arbitrary::Unstructured, dst: u8) -> BoxResult<u8> {
        let src =
            if !self.state.known_producers.is_empty() && self.state.known_producers.len() > 4 && u.ratio(3u8, 4u8)? {
                *self
                    .state
                    .known_producers
                    .iter()
                    .nth(u.int_in_range(0 ..= self.state.known_producers.len() - 1)?)
                    .ok_or("indexing failed1")?
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
        // let module_name = self.names_edges.next().ok_or("name generation failed")?;
        let module_name = fake_name(self.rng, self.config.more_escapes);
        let source_path = if u.arbitrary()? {
            None
        } else {
            let primary_output = &self
                .state
                .info_mem
                .get(&src)
                .ok_or("lookup failed")?
                .primary_output
                .as_deref()
                .and_then(|path| AsRef::<str>::as_ref(path).strip_suffix(".o"))
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
        if let Some(desc) = self.state.info_mem.get(&src) {
            if !desc.provides.is_empty() && u.ratio(3u8, 4u8)? {
                let provided = desc
                    .provides
                    .get(u.int_in_range(0 ..= desc.provides.len() - 1)?)
                    .ok_or("indexing failed2")?
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
        self.state
            .graph
            .add_edge(src, dst, provided_desc.desc.view().logical_name.to_owned());
        self.state.known_producers.insert(src);
        self.state.known_consumers.insert(dst);
        let requires = &mut self.state.info_mem.get_mut(&dst).ok_or("lookup failed")?.requires;
        if !requires
            .iter()
            .any(|required| required.desc.view().logical_name == provided_desc.desc.view().logical_name)
        {
            requires.push(r5::RequiredModuleDesc {
                desc: provided_desc.desc.clone(),
                lookup_method: u.arbitrary()?,
            });
        }
        let provides = &mut self.state.info_mem.get_mut(&src).ok_or("lookup failed")?.provides;
        if !provides
            .iter()
            .any(|provided| provided.desc.view().logical_name == provided_desc.desc.view().logical_name)
        {
            provides.push(provided_desc);
        }
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
        while i < self.config.node_count {
            let dst = self.gen_dst(u)?;
            let src = self.gen_src(u, dst)?;
            let provided_desc = self.gen_provided_desc(u, src)?;
            self.add_edge(u, src, dst, provided_desc)?;
            i += 1;
        }

        Ok((self.state.info_mem, self.state.graph))
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "serialize")]
    mod serialize {

        use rand::prelude::*;

        use super::super::*;

        #[test]
        fn test() -> BoxResult<()> {
            let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(crate::r5::datagen::CHACHA8RNG_SEED);
            let mut bytes = alloc::vec![0u8; 8192];
            rng.fill_bytes(&mut bytes);
            let mut u = arbitrary::Unstructured::new(&bytes);

            let config = GraphGeneratorConfig::default().node_count(rng.gen_range(0u8 ..= 16u8));

            let generator = GraphGenerator::new(rng, config);
            let (info_mem, graph) = generator.run(&mut u)?;

            for key in info_mem.keys().copied() {
                #[allow(unused)]
                for (src, dst, weight) in graph.edges(key) {
                    let name_src = info_mem.get(&src).unwrap().primary_output.as_deref().unwrap();
                    let name_dst = info_mem.get(&dst).unwrap().primary_output.as_deref().unwrap();
                    // std::println!("{name_src}::{src} -[ {weight} ]-> {name_dst}::{dst}");
                }
            }

            let select = rand::distributions::Bernoulli::from_ratio(1, 4)?.sample(rng);
            let rules = info_mem
                .into_values()
                .filter(|desc| !(desc.provides.is_empty() && desc.requires.is_empty()) || select)
                .collect();
            let dep_file = r5::DepFile {
                version: 1,
                revision: None,
                rules,
            };

            #[allow(unused)]
            let str = serde_json::to_string_pretty(&dep_file).unwrap();
            // std::println!("{str}");

            Ok(())
        }

        #[cfg(feature = "serialize")]
        #[test]
        fn test_many() {
            let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(crate::r5::datagen::CHACHA8RNG_SEED);
            let config = GraphGeneratorConfig::default().node_count(rng.gen_range(0u8 ..= 16u8));
            let dep_files = GraphGenerator::gen_dep_files(rng, config)
                .flat_map(|result| result.and_then(|dep_file| r5::datagen::json::pretty_print_unindented(&dep_file)));
            #[allow(unused)]
            for file in dep_files.take(1) {
                // std::println!("{file}\n");
            }
        }
    }
}
