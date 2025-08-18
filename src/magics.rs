use std::fmt;

use bevy::asset::AssetLoader;
use bevy::asset::LoadContext;
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use engine::prelude::MagicTable;
use serde::Deserialize;

#[allow(unused)]
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct MagicTableAsset(pub MagicTable);

#[derive(Default)]
pub struct MagicTableLoader;

#[derive(Debug)]
pub enum MagicTableLoaderError {
    Io(std::io::Error),
    RonSpannedError(ron::error::SpannedError),
}

impl fmt::Display for MagicTableLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicTableLoaderError::Io(err) => write!(f, "Could not load asset: {err}"),
            MagicTableLoaderError::RonSpannedError(err) => write!(f, "Could not parse RON: {err}"),
        }
    }
}

impl std::error::Error for MagicTableLoaderError {}

impl From<std::io::Error> for MagicTableLoaderError {
    fn from(error: std::io::Error) -> Self {
        MagicTableLoaderError::Io(error)
    }
}

impl From<ron::error::SpannedError> for MagicTableLoaderError {
    fn from(error: ron::error::SpannedError) -> Self {
        MagicTableLoaderError::RonSpannedError(error)
    }
}

impl AssetLoader for MagicTableLoader {
    type Asset = MagicTableAsset;
    type Settings = ();
    type Error = MagicTableLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading MagicTable...");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let magic_table = ron::de::from_bytes::<MagicTable>(&bytes)?;

        Ok(MagicTableAsset(magic_table))
    }

    fn extensions(&self) -> &[&str] {
        &[".ron"]
    }
}
