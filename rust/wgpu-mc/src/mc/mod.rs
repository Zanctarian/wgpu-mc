//! Rust implementations of minecraft concepts that are important to us.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use guillotiere::euclid::default;
use indexmap::map::IndexMap;
use minecraft_assets::schemas;
use parking_lot::RwLock;

use crate::mc::chunk::ChunkManager;
use crate::mc::entity::Entity;
use crate::mc::resource::ResourceProvider;
use crate::render::atlas::{Atlas, TextureManager};
use crate::render::pipeline::BLOCK_ATLAS;
use crate::texture::BindableTexture;
use crate::{WgpuState, WmRenderer};

use self::block::ModelMesh;
use self::resource::ResourcePath;

pub mod block;
pub mod chunk;
pub mod entity;
pub mod resource;

/// Take in a block name (not a [ResourcePath]!) and optionally a variant state key, e.g. "facing=north" and format it some way
/// for example, `minecraft:anvil[facing=north]` or `Block{minecraft:anvil}[facing=north]`
pub type BlockVariantFormatter = dyn Fn(&str, Option<&str>) -> String;

pub struct BlockManager {
    /// This maps block state keys to either a [VariantMesh] or a [Multipart] struct. How the keys are formatted
    /// is defined by the user of wgpu-mc. For example `Block{minecraft:anvil}[facing=west]` or `minecraft:anvil#facing=west`
    pub blocks: IndexMap<String, Block>,
}

#[derive(Debug)]
pub enum Block {
    Multipart(Multipart),
    Variants(IndexMap<String, Vec<Arc<ModelMesh>>>),
}

impl Block {
    pub fn get_model(&self, key: u16, _seed: u8) -> Arc<ModelMesh> {
        match &self {
            Block::Multipart(multipart) => multipart
                .keys
                .read()
                .get_index(key as usize)
                .expect(&format!("{self:#?}\n{key}"))
                .1
                .clone(),
            //TODO, random variant selection through weight and seed
            Block::Variants(variants) => variants.get_index(key as usize).unwrap().1[0].clone(),
        }
    }

    pub fn get_model_by_key<'a>(
        &self,
        key: impl IntoIterator<Item = (&'a str, &'a schemas::blockstates::multipart::StateValue)>
            + Clone,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
        //TODO use this
        _seed: u8,
    ) -> Option<(Arc<ModelMesh>, u16)> {
        let key_string = key
            .clone()
            .into_iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    key,
                    match value {
                        schemas::blockstates::multipart::StateValue::Bool(bool) =>
                            if *bool {
                                "true"
                            } else {
                                "false"
                            },
                        schemas::blockstates::multipart::StateValue::String(string) => string,
                    }
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        match &self {
            Block::Multipart(multipart) => {
                {
                    if let Some(full) = multipart.keys.read().get_full(&key_string) {
                        return Some((full.2.clone(), full.0 as u16));
                    }
                }

                let mesh = multipart.generate_mesh(key, resource_provider, block_atlas);

                let mut multipart_write = multipart.keys.write();
                multipart_write.insert(key_string, mesh.clone());

                Some((mesh, multipart_write.len() as u16 - 1))
            }
            Block::Variants(variants) => {
                let full = variants.get_full(&key_string)?;
                Some((full.2[0].clone(), full.0 as u16))
            }
        }
    }
}

#[derive(Debug)]
pub struct Multipart {
    pub cases: Vec<schemas::blockstates::multipart::Case>,
    pub keys: RwLock<IndexMap<String, Arc<ModelMesh>>>,
}

impl Multipart {
    pub fn generate_mesh<'a>(
        &self,
        key: impl IntoIterator<Item = (&'a str, &'a schemas::blockstates::multipart::StateValue)>
            + Clone,
        resource_provider: &dyn ResourceProvider,
        block_atlas: &Atlas,
    ) -> Arc<ModelMesh> {
        let apply_variants = self.cases.iter().filter_map(|case| {
            if case.applies(key.clone()) {
                Some(case.apply.models())
            } else {
                None
            }
        });

        let mesh = ModelMesh::bake(
            apply_variants.into_iter().flatten(),
            resource_provider,
            block_atlas,
        )
        .unwrap();

        Arc::new(mesh)
    }
}

pub enum MultipartOrMesh {
    Multipart(Arc<Multipart>),
    Mesh(Arc<ModelMesh>),
}

/// Multipart models are generated dynamically as they can be too complex
pub struct BlockInstance {
    pub render_settings: block::RenderSettings,
    pub block: MultipartOrMesh,
}

#[derive(Default, Clone)]
pub struct SkyData {
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub angle: f32,
    pub brightness: f32,
    pub star_shimmer: f32,
    pub moon_phase: i32,
    pub textures: HashMap<String, Arc<BindableTexture>>,
}

#[derive(Default, Clone)]
pub struct RenderEffectsData {
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_shape: f32,
    pub fog_color: Vec<f32>,
    pub color_modulator: Vec<f32>,
    pub dimension_fog_color: Vec<f32>,
}

/// Minecraft-specific state and data structures go in here
pub struct MinecraftState {
    pub sky_data: ArcSwap<SkyData>,
    pub stars_index_buffer: RwLock<Option<wgpu::Buffer>>,
    pub stars_vertex_buffer: RwLock<Option<wgpu::Buffer>>,
    pub stars_length: RwLock<u32>,
    pub render_effects: ArcSwap<RenderEffectsData>,

    pub block_manager: RwLock<BlockManager>,

    pub chunks: ChunkManager,
    pub entity_models: RwLock<HashMap<String, Arc<Entity>>>,

    pub resource_provider: Arc<dyn ResourceProvider>,

    pub texture_manager: TextureManager,

    pub animated_block_buffer: ArcSwap<Option<wgpu::Buffer>>,
    pub animated_block_bind_group: ArcSwap<Option<wgpu::BindGroup>>,
}

impl MinecraftState {
    #[must_use]
    pub fn new(wgpu_state: &WgpuState, resource_provider: Arc<dyn ResourceProvider>) -> Self {
        MinecraftState {
            sky_data: ArcSwap::new(Arc::new(SkyData::default())),
            stars_index_buffer: RwLock::new(None),
            stars_vertex_buffer: RwLock::new(None),
            stars_length: RwLock::new(0),
            render_effects: ArcSwap::new(Arc::new(RenderEffectsData::default())),

            chunks: ChunkManager::new(wgpu_state),
            entity_models: RwLock::new(HashMap::new()),

            texture_manager: TextureManager::new(),

            block_manager: RwLock::new(BlockManager {
                blocks: IndexMap::new(),
            }),

            resource_provider,

            animated_block_buffer: ArcSwap::new(Arc::new(None)),
            animated_block_bind_group: ArcSwap::new(Arc::new(None)),
        }
    }

    /// Bake blocks from their blockstates
    ///
    /// # Example
    ///
    ///```ignore
    /// # use wgpu_mc::mc::MinecraftState;
    /// # use wgpu_mc::mc::resource::ResourcePath;
    /// # use wgpu_mc::WmRenderer;
    ///
    /// # let minecraft_state: MinecraftState;
    /// # let wm: WmRenderer;
    ///
    /// minecraft_state.bake_blocks(
    ///     &wm,
    ///     [("minecraft:anvil", &ResourcePath("minecraft:blockstates/anvil.json".into()))]
    /// );
    /// ```
    pub fn bake_blocks<'a>(
        &self,
        wm: &WmRenderer,
        block_states: impl IntoIterator<Item = (impl AsRef<str>, &'a ResourcePath)>,
    ) {
        puffin::profile_function!();

        let mut block_manager = self.block_manager.write();
        let block_atlas = self
            .texture_manager
            .atlases
            .load()
            .get(BLOCK_ATLAS)
            .unwrap()
            .load();

        //Figure out which block models there are
        block_states
            .into_iter()
            .for_each(|(block_name, block_state)| {
                let blockstates: schemas::BlockStates =
                    serde_json::from_str(&self.resource_provider.get_string(block_state).unwrap())
                        .unwrap();

                let block = match &blockstates {
                    schemas::BlockStates::Variants { variants } => {
                        let meshes: IndexMap<String, Vec<Arc<ModelMesh>>> = variants
                            .iter()
                            .map(|(variant_id, variant)| {
                                (
                                    variant_id.clone(),
                                    variant
                                        .models()
                                        .iter()
                                        .map(|variation| {
                                            Arc::new(
                                                ModelMesh::bake(
                                                    std::slice::from_ref(variation),
                                                    &*self.resource_provider,
                                                    &block_atlas,
                                                )
                                                .unwrap(),
                                            )
                                        })
                                        .collect::<Vec<Arc<ModelMesh>>>(),
                                )
                            })
                            .collect();

                        Block::Variants(meshes)
                    }
                    schemas::BlockStates::Multipart { cases } => Block::Multipart(Multipart {
                        cases: cases.clone(),
                        keys: RwLock::new(IndexMap::new()),
                    }),
                };

                block_manager
                    .blocks
                    .insert(String::from(block_name.as_ref()), block);
            });

        block_atlas.upload(wm);
    }
}
