use crate::common::operation_error::OperationResult;
use crate::common::Flusher;
use crate::id_tracker::{IdTracker, IdTrackerEnum};
use crate::types::{ExtendedPointId, PointIdType, SeqNumberType};
use bitvec::prelude::BitSlice;
use bitvec::vec::BitVec;
use common::types::PointOffsetType;
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

const DELETED_FILE_NAME: &str = "id_tracker_deleted";
const MAPPINGS_FILE_NAME: &str = "id_tracker_mappings";

#[derive(Serialize, Deserialize)]
pub struct ImmutableIdTracker {
    #[serde(skip)]
    path: PathBuf,

    deleted: Arc<RwLock<BitVec>>,
    mappings: PointMappings,
}

#[derive(Serialize, Deserialize)]
struct PointMappings {
    internal_to_version: Vec<SeqNumberType>,

    internal_to_external: Vec<PointIdType>,
    external_to_internal: BTreeMap<ExtendedPointId, PointOffsetType>,
}

impl ImmutableIdTracker {
    pub fn open(segment_path: &Path) -> OperationResult<Self> {
        let deleted: BitVec = Self::open_file(segment_path.join(DELETED_FILE_NAME).as_path())?;
        let mappings: PointMappings =
            Self::open_file(segment_path.join(MAPPINGS_FILE_NAME).as_path())?;

        Ok(Self {
            path: segment_path.to_path_buf(),
            deleted: Arc::new(RwLock::new(deleted)),
            mappings,
        })
    }

    fn open_file<T: DeserializeOwned>(file: &Path) -> OperationResult<T> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    fn deleted_file_path(&self) -> PathBuf {
        self.path.join(DELETED_FILE_NAME)
    }

    #[allow(dead_code)]
    fn mappings_file_path(&self) -> PathBuf {
        self.path.join(MAPPINGS_FILE_NAME)
    }

    pub(crate) fn save<T: Serialize>(path: &Path, value: &T) -> OperationResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, value)?;
        Ok(())
    }
}

impl IdTracker for ImmutableIdTracker {
    fn internal_version(&self, internal_id: PointOffsetType) -> Option<SeqNumberType> {
        self.mappings
            .internal_to_version
            .get(internal_id as usize)
            .copied()
    }

    fn set_internal_version(
        &mut self,
        _internal_id: PointOffsetType,
        _version: SeqNumberType,
    ) -> OperationResult<()> {
        panic!("Trying to call a mutating function (`set_internal_version`) of an immutable id tracker");
    }

    fn internal_id(&self, external_id: PointIdType) -> Option<PointOffsetType> {
        self.mappings
            .external_to_internal
            .get(&external_id)
            .copied()
    }

    fn external_id(&self, internal_id: PointOffsetType) -> Option<PointIdType> {
        let deleted = self.deleted.read();
        if let Some(deleted) = deleted.get(internal_id as usize) {
            if !deleted {
                return self
                    .mappings
                    .internal_to_external
                    .get(internal_id as usize)
                    .copied();
            }
        }
        None
    }

    fn set_link(
        &mut self,
        _external_id: PointIdType,
        _internal_id: PointOffsetType,
    ) -> OperationResult<()> {
        panic!("Trying to call a mutating function (`set_link`) of an immutable id tracker");
    }

    fn drop(&mut self, _external_id: PointIdType) -> OperationResult<()> {
        panic!("Trying to call a mutating function (`drop`) of an immutable id tracker");
    }

    fn iter_external(&self) -> Box<dyn Iterator<Item = PointIdType> + '_> {
        let iter_num = self
            .mappings
            .external_to_internal
            .keys()
            .filter(|i| i.is_num_id())
            .copied()
            .map(|i| i as PointIdType);

        let iter_uuid = self
            .mappings
            .external_to_internal
            .keys()
            .filter(|i| i.is_uuid())
            .copied()
            .map(|i| i as PointIdType);
        // order is important here, we want to iterate over the u64 ids first
        Box::new(iter_num.chain(iter_uuid))
    }

    fn iter_internal(&self) -> Box<dyn Iterator<Item = PointOffsetType> + '_> {
        Box::new(
            (0..self.mappings.internal_to_external.len() as PointOffsetType)
                .filter(move |i| !self.deleted.read()[*i as usize]),
        )
    }

    fn iter_from(
        &self,
        external_id: Option<PointIdType>,
    ) -> Box<dyn Iterator<Item = (PointIdType, PointOffsetType)> + '_> {
        let full_num_iter = || {
            self.mappings
                .external_to_internal
                .iter()
                .filter(|i| i.0.is_num_id())
                .map(|(k, v)| (*k as PointIdType, *v))
        };
        let offset_num_iter = |offset: u64| {
            self.mappings
                .external_to_internal
                .range(ExtendedPointId::NumId(offset)..)
                .filter(|i| i.0.is_num_id())
                .map(|(k, v)| (*k as PointIdType, *v))
        };
        let full_uuid_iter = || {
            self.mappings
                .external_to_internal
                .iter()
                .filter(|i| i.0.is_uuid())
                .map(|(k, v)| (*k as PointIdType, *v))
        };
        let offset_uuid_iter = |offset: Uuid| {
            self.mappings
                .external_to_internal
                .range(ExtendedPointId::Uuid(offset)..)
                .map(|(k, v)| (*k as PointIdType, *v))
        };

        match external_id {
            None => {
                let iter_num = full_num_iter();
                let iter_uuid = full_uuid_iter();
                // order is important here, we want to iterate over the u64 ids first
                Box::new(iter_num.chain(iter_uuid))
            }
            Some(offset) => match offset {
                PointIdType::NumId(idx) => {
                    // Because u64 keys are less that uuid key, we can just use the full iterator for uuid
                    let iter_num = offset_num_iter(idx);
                    let iter_uuid = full_uuid_iter();
                    // order is important here, we want to iterate over the u64 ids first
                    Box::new(iter_num.chain(iter_uuid))
                }
                PointIdType::Uuid(uuid) => {
                    // if offset is a uuid, we can only iterate over uuids
                    Box::new(offset_uuid_iter(uuid))
                }
            },
        }
    }

    fn iter_ids(&self) -> Box<dyn Iterator<Item = PointOffsetType> + '_> {
        self.iter_internal()
    }

    /// Creates a flusher function, that flushes the deleted points bitvec to disk.
    fn mapping_flusher(&self) -> Flusher {
        let deleted = self.deleted.clone();
        let path = self.deleted_file_path();
        Box::new(move || {
            Self::save(&path, &*deleted.read())?;
            Ok(())
        })
    }

    /// Not implemented for immutable id tracker.
    fn versions_flusher(&self) -> Flusher {
        Box::new(|| Ok(()))
    }

    fn total_point_count(&self) -> usize {
        self.mappings.internal_to_external.len()
    }

    fn available_point_count(&self) -> usize {
        self.mappings.external_to_internal.len()
    }

    fn deleted_point_count(&self) -> usize {
        self.total_point_count() - self.available_point_count()
    }

    fn deleted_point_bitslice(&self) -> &BitSlice {
        Box::new(self.deleted.read().clone()).leak()
    }

    fn is_deleted_point(&self, key: PointOffsetType) -> bool {
        let key = key as usize;
        let deleted = self.deleted.read();
        if key >= deleted.len() {
            return true;
        }
        deleted[key]
    }

    fn make_immutable(self) -> OperationResult<IdTrackerEnum> {
        Ok(IdTrackerEnum::ImmutableIdTracker(self))
    }

    fn cleanup_versions(&mut self) -> OperationResult<()> {
        panic!(
            "Trying to call a mutating function (`cleanup_versions`) of an immutable id tracker"
        );
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_iterator() {}

    #[test]
    fn test_mixed_types_iterator() {}
}
