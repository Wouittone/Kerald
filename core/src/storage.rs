use crate::{PolledPayloadBatch, TimestampCursor, TopicDefinition, TopicName};
use arrow_array::{Array, ArrayRef, RecordBatch, RecordBatchIterator, TimestampNanosecondArray};
use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};
use futures::TryStreamExt;
use lance::{
    Dataset,
    dataset::{DatasetBuilder, ReadParams, WriteMode, WriteParams},
};
use lance_io::object_store::{ObjectStoreParams, uri_to_url};
use lance_table::io::commit::RenameCommitHandler;
use object_store::DynObjectStore;
use object_store_opendal::OpendalStore;
use opendal::{Operator, services::Fs};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

pub const KERALD_CURSOR_FIELD: &str = "__kerald_cursor_ns";

const STORAGE_INIT_FAILED: &str = "storage could not be initialized";
const STORAGE_OPERATION_FAILED: &str = "storage operation failed";
const INVALID_STORAGE_ROOT: &str = "storage root must be a valid filesystem path";
const INVALID_CURSOR_COLUMN: &str = "stored payload cursor column is invalid";
const PAYLOAD_SCHEMA_MISMATCH: &str = "payload schema must match topic schema";

/// Broker storage settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StorageConfig {
    root: PathBuf,
}

impl StorageConfig {
    pub fn local(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::local("kerald-data")
    }
}

/// OpenDAL-owned storage boundary used by broker persistence.
#[derive(Clone)]
pub struct OpenDalStorage {
    root: PathBuf,
    operator: Operator,
}

impl std::fmt::Debug for OpenDalStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenDalStorage").field("root", &self.root).finish_non_exhaustive()
    }
}

impl OpenDalStorage {
    pub async fn local(config: &StorageConfig) -> Result<Self, StorageError> {
        let root = absolute_path(config.root())
            .await
            .map_err(|_| StorageError::Init(INVALID_STORAGE_ROOT))?;
        let root_str = root.to_str().ok_or(StorageError::Init(INVALID_STORAGE_ROOT))?;

        let builder = Fs::default().root(root_str);
        let operator = Operator::new(builder)
            .map_err(|_| StorageError::Init(STORAGE_INIT_FAILED))?
            .finish();

        operator
            .create_dir("topics/")
            .await
            .map_err(|_| StorageError::Init(STORAGE_INIT_FAILED))?;

        Ok(Self { root, operator })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub async fn append_payload(&self, topic: &TopicDefinition, cursor: TimestampCursor, payload: RecordBatch) -> Result<(), StorageError> {
        validate_topic_schema(topic.schema())?;
        validate_payload_schema(topic.schema(), payload.schema().as_ref())?;

        let dataset_path = self.dataset_path(topic.name());
        let dataset_exists = self
            .operator
            .exists(&dataset_path)
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;
        let stored_batch = stored_payload_batch(topic, cursor, payload)?;
        let reader = RecordBatchIterator::new([Ok(stored_batch)], stored_schema(topic.schema()));

        if dataset_exists {
            let mut dataset = self.open_dataset(topic.name()).await?;
            validate_dataset_schema(&dataset, topic.schema())?;
            dataset
                .append(reader, Some(self.write_params(topic.name(), WriteMode::Append)?))
                .await
                .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;
        } else {
            Dataset::write(
                reader,
                &self.dataset_uri(topic.name()),
                Some(self.write_params(topic.name(), WriteMode::Create)?),
            )
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;
        }

        Ok(())
    }

    pub async fn poll_payloads(&self, topic: &TopicDefinition, after: TimestampCursor) -> Result<Vec<PolledPayloadBatch>, StorageError> {
        validate_topic_schema(topic.schema())?;

        let dataset_path = self.dataset_path(topic.name());
        if !self
            .operator
            .exists(&dataset_path)
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?
        {
            return Ok(Vec::new());
        }

        let dataset = self.open_dataset(topic.name()).await?;
        validate_dataset_schema(&dataset, topic.schema())?;
        let mut scanner = dataset.scan();
        scanner
            .filter(format!("{KERALD_CURSOR_FIELD} > {}", after.as_nanos()))
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;
        let batches: Vec<RecordBatch> = scanner
            .try_into_stream()
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?
            .try_collect()
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;

        let mut payload_batches = Vec::new();
        for batch in batches.into_iter().filter(|batch| batch.num_rows() > 0) {
            payload_batches.extend(polled_payload_batches(topic.schema(), batch)?);
        }
        payload_batches.sort_by_key(|batch| batch.cursor());

        Ok(payload_batches)
    }

    async fn open_dataset(&self, topic_name: &TopicName) -> Result<Dataset, StorageError> {
        DatasetBuilder::from_uri(self.dataset_uri(topic_name))
            .with_read_params(self.read_params(topic_name)?)
            .load()
            .await
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))
    }

    fn dataset_path(&self, topic_name: &TopicName) -> String {
        format!("topics/{topic_name}.lance")
    }

    fn dataset_uri(&self, topic_name: &TopicName) -> String {
        format!("opendal:///{}", self.dataset_path(topic_name))
    }

    fn object_store_params(&self, topic_name: &TopicName) -> Result<ObjectStoreParams, StorageError> {
        let store: Arc<DynObjectStore> = Arc::new(OpendalStore::new(self.operator.clone()));
        let url = uri_to_url(&self.dataset_uri(topic_name)).map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;

        #[allow(deprecated)]
        Ok(ObjectStoreParams {
            object_store: Some((store, url)),
            list_is_lexically_ordered: Some(false),
            ..ObjectStoreParams::default()
        })
    }

    fn read_params(&self, topic_name: &TopicName) -> Result<ReadParams, StorageError> {
        Ok(ReadParams {
            store_options: Some(self.object_store_params(topic_name)?),
            commit_handler: Some(Arc::new(RenameCommitHandler)),
            ..ReadParams::default()
        })
    }

    fn write_params(&self, topic_name: &TopicName, mode: WriteMode) -> Result<WriteParams, StorageError> {
        Ok(WriteParams {
            mode,
            store_params: Some(self.object_store_params(topic_name)?),
            commit_handler: Some(Arc::new(RenameCommitHandler)),
            ..WriteParams::default()
        })
    }
}

/// Storage boundary errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StorageError {
    #[error("storage initialization failed: {0}")]
    Init(&'static str),
    #[error("storage operation failed: {0}")]
    Operation(&'static str),
    #[error("storage schema mismatch: {0}")]
    SchemaMismatch(&'static str),
    #[error("storage reserved field name is not allowed: {0}")]
    ReservedFieldName(&'static str),
}

async fn absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    tokio::fs::create_dir_all(&path).await?;

    Ok(path)
}

fn validate_topic_schema(topic_schema: &SchemaRef) -> Result<(), StorageError> {
    if topic_schema.fields().iter().any(|field| field.name() == KERALD_CURSOR_FIELD) {
        return Err(StorageError::ReservedFieldName(KERALD_CURSOR_FIELD));
    }

    Ok(())
}

fn validate_payload_schema(topic_schema: &SchemaRef, payload_schema: &Schema) -> Result<(), StorageError> {
    if topic_schema.as_ref() != payload_schema {
        return Err(StorageError::SchemaMismatch(PAYLOAD_SCHEMA_MISMATCH));
    }

    Ok(())
}

fn validate_dataset_schema(dataset: &Dataset, topic_schema: &SchemaRef) -> Result<(), StorageError> {
    let actual_schema: Schema = dataset.schema().into();
    let expected_schema = stored_schema(topic_schema);
    if &actual_schema != expected_schema.as_ref() {
        return Err(StorageError::SchemaMismatch(PAYLOAD_SCHEMA_MISMATCH));
    }

    Ok(())
}

fn stored_schema(topic_schema: &SchemaRef) -> SchemaRef {
    let cursor_field = Field::new(KERALD_CURSOR_FIELD, DataType::Timestamp(TimeUnit::Nanosecond, None), false);
    let mut fields = Vec::with_capacity(topic_schema.fields().len() + 1);
    fields.push(cursor_field);
    fields.extend(topic_schema.fields().iter().map(|field| field.as_ref().clone()));

    Arc::new(Schema::new(fields))
}

fn stored_payload_batch(topic: &TopicDefinition, cursor: TimestampCursor, payload: RecordBatch) -> Result<RecordBatch, StorageError> {
    let cursor_column: ArrayRef = Arc::new(TimestampNanosecondArray::from(vec![cursor.as_nanos(); payload.num_rows()]));
    let mut columns = Vec::with_capacity(payload.num_columns() + 1);
    columns.push(cursor_column);
    columns.extend(payload.columns().iter().cloned());

    RecordBatch::try_new(stored_schema(topic.schema()), columns).map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))
}

fn polled_payload_batches(topic_schema: &SchemaRef, batch: RecordBatch) -> Result<Vec<PolledPayloadBatch>, StorageError> {
    let cursor_column = batch
        .column(0)
        .as_any()
        .downcast_ref::<TimestampNanosecondArray>()
        .ok_or(StorageError::Operation(INVALID_CURSOR_COLUMN))?;

    let mut payload_batches = Vec::new();
    let mut start = 0;
    while start < batch.num_rows() {
        if cursor_column.is_null(start) {
            return Err(StorageError::Operation(INVALID_CURSOR_COLUMN));
        }

        let cursor = TimestampCursor::try_new(cursor_column.value(start)).map_err(|_| StorageError::Operation(INVALID_CURSOR_COLUMN))?;
        let mut end = start + 1;
        while end < batch.num_rows() && !cursor_column.is_null(end) && cursor_column.value(end) == cursor.as_nanos() {
            end += 1;
        }

        let slice = batch.slice(start, end - start);
        let payload = RecordBatch::try_new(topic_schema.clone(), slice.columns()[1..].to_vec())
            .map_err(|_| StorageError::Operation(STORAGE_OPERATION_FAILED))?;
        payload_batches.push(PolledPayloadBatch::new(cursor, payload));
        start = end;
    }

    Ok(payload_batches)
}
