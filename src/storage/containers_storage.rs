use my_logger::{LOGGER, LogEventCtx};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::models::ContainersError;

use super::{
    ContainerInfoYamlModel, ContainerListItemModel, ContainerLocks, ContainerTagModel,
    ContainerTagYamlModel, DeletedContainerTagModel, DownloadedContainerTagModel,
    UploadContainerBlobParams, UploadedContainerTagModel,
};

/// Content-addressable container storage on the local file system.
///
/// ```text
/// {containers_path}/{container_name}/container-info.yaml   <- tag -> hash index
/// {containers_path}/{container_name}/{hash}.zip            <- the blobs
/// ```
///
/// Every operation on a single container runs under that container's own lock,
/// so writing the blob and updating `container-info.yaml` is one atomic step.
pub struct ContainersStorage {
    root_path: String,
    locks: ContainerLocks,
}

impl ContainersStorage {
    pub fn new(root_path: String) -> Self {
        Self {
            root_path: root_path.trim_end_matches('/').to_string(),
            locks: ContainerLocks::new(),
        }
    }

    pub async fn upload(&self, params: UploadContainerBlobParams) -> UploadedContainerTagModel {
        let container_dir = self.compile_container_dir(&params.container_name);

        let lock = self.locks.get_lock(&params.container_name).await;
        let _access = lock.lock().await;

        tokio::fs::create_dir_all(container_dir.as_str())
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "containers_storage: can not create dir {}. Err: {:?}",
                    container_dir, err
                )
            });

        let mut container_info = Self::load_container_info(container_dir.as_str())
            .await
            .unwrap_or_else(|| ContainerInfoYamlModel::new(&params.container_name));

        let size = params.content.len() as u64;

        let blob_file_name = Self::compile_blob_file_name(container_dir.as_str(), &params.hash);

        // Same hash means byte-identical content — the blob is already there.
        if !Self::file_exists(blob_file_name.as_str()).await {
            Self::write_file(blob_file_name.as_str(), params.content.as_slice()).await;
        }

        let previous_tag = container_info.tags.insert(
            params.tag.clone(),
            ContainerTagYamlModel {
                hash: params.hash.clone(),
                size,
                uploaded_at: DateTimeAsMicroseconds::now().to_rfc3339(),
                uploaded_by: params.uploaded_by,
            },
        );

        let orphan_hash = match previous_tag {
            Some(previous_tag) if previous_tag.hash != params.hash => Some(previous_tag.hash),
            Some(_) => None,
            None => None,
        };

        // Yaml goes first — if the process dies right here, we are left with an
        // unreferenced blob, never with a tag pointing at a deleted file.
        Self::save_container_info(container_dir.as_str(), &container_info).await;

        let mut replaced_hash = None;
        let mut orphan_deleted = false;

        if let Some(orphan_hash) = orphan_hash {
            replaced_hash = Some(orphan_hash.clone());

            if !container_info.has_reference_to_hash(orphan_hash.as_str()) {
                orphan_deleted =
                    Self::delete_blob(container_dir.as_str(), orphan_hash.as_str()).await;
            }
        }

        UploadedContainerTagModel {
            container_name: params.container_name,
            tag: params.tag,
            hash: params.hash,
            size,
            replaced_hash,
            orphan_deleted,
        }
    }

    pub async fn download(
        &self,
        container_name: &str,
        tag: &str,
    ) -> Result<DownloadedContainerTagModel, ContainersError> {
        let container_dir = self.compile_container_dir(container_name);

        let lock = self.locks.get_lock(container_name).await;
        let _access = lock.lock().await;

        let container_info = Self::load_container_info(container_dir.as_str())
            .await
            .ok_or_else(|| ContainersError::ContainerNotFound(container_name.to_string()))?;

        let tag_info =
            container_info
                .tags
                .get(tag)
                .ok_or_else(|| ContainersError::TagNotFound {
                    container: container_name.to_string(),
                    tag: tag.to_string(),
                })?;

        let blob_file_name = Self::compile_blob_file_name(container_dir.as_str(), &tag_info.hash);

        let content = tokio::fs::read(blob_file_name.as_str())
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "containers_storage: can not read blob {}. Err: {:?}",
                    blob_file_name, err
                )
            });

        Ok(DownloadedContainerTagModel {
            file_name: format!("{}-{}.zip", container_name, tag),
            content,
        })
    }

    pub async fn get_tag(
        &self,
        container_name: &str,
        tag: &str,
    ) -> Result<ContainerTagModel, ContainersError> {
        let container_dir = self.compile_container_dir(container_name);

        let lock = self.locks.get_lock(container_name).await;
        let _access = lock.lock().await;

        let mut container_info = Self::load_container_info(container_dir.as_str())
            .await
            .ok_or_else(|| ContainersError::ContainerNotFound(container_name.to_string()))?;

        let tag_info =
            container_info
                .tags
                .remove(tag)
                .ok_or_else(|| ContainersError::TagNotFound {
                    container: container_name.to_string(),
                    tag: tag.to_string(),
                })?;

        Ok(ContainerTagModel {
            container_name: container_name.to_string(),
            tag: tag.to_string(),
            hash: tag_info.hash,
            size: tag_info.size,
            uploaded_at: tag_info.uploaded_at,
            uploaded_by: tag_info.uploaded_by,
        })
    }

    pub async fn get_tags(
        &self,
        container_name: &str,
    ) -> Result<Vec<ContainerTagModel>, ContainersError> {
        let container_dir = self.compile_container_dir(container_name);

        let lock = self.locks.get_lock(container_name).await;
        let _access = lock.lock().await;

        let container_info = Self::load_container_info(container_dir.as_str())
            .await
            .ok_or_else(|| ContainersError::ContainerNotFound(container_name.to_string()))?;

        let result = container_info
            .tags
            .into_iter()
            .map(|(tag, tag_info)| ContainerTagModel {
                container_name: container_name.to_string(),
                tag,
                hash: tag_info.hash,
                size: tag_info.size,
                uploaded_at: tag_info.uploaded_at,
                uploaded_by: tag_info.uploaded_by,
            })
            .collect();

        Ok(result)
    }

    pub async fn get_containers(&self) -> Vec<ContainerListItemModel> {
        let container_names = self.get_container_names().await;

        let mut result = Vec::with_capacity(container_names.len());

        for container_name in container_names {
            let container_dir = self.compile_container_dir(container_name.as_str());

            let lock = self.locks.get_lock(container_name.as_str()).await;
            let _access = lock.lock().await;

            if let Some(container_info) = Self::load_container_info(container_dir.as_str()).await {
                result.push(ContainerListItemModel {
                    container: container_name,
                    tags_amount: container_info.tags.len(),
                });
            }
        }

        result
    }

    pub async fn delete_tag(
        &self,
        container_name: &str,
        tag: &str,
    ) -> Result<DeletedContainerTagModel, ContainersError> {
        let container_dir = self.compile_container_dir(container_name);

        let lock = self.locks.get_lock(container_name).await;
        let _access = lock.lock().await;

        let mut container_info = Self::load_container_info(container_dir.as_str())
            .await
            .ok_or_else(|| ContainersError::ContainerNotFound(container_name.to_string()))?;

        let removed_tag =
            container_info
                .tags
                .remove(tag)
                .ok_or_else(|| ContainersError::TagNotFound {
                    container: container_name.to_string(),
                    tag: tag.to_string(),
                })?;

        Self::save_container_info(container_dir.as_str(), &container_info).await;

        let mut orphan_deleted = false;

        if !container_info.has_reference_to_hash(removed_tag.hash.as_str()) {
            orphan_deleted =
                Self::delete_blob(container_dir.as_str(), removed_tag.hash.as_str()).await;
        }

        Ok(DeletedContainerTagModel {
            hash: removed_tag.hash,
            orphan_deleted,
        })
    }

    async fn get_container_names(&self) -> Vec<String> {
        let mut dir = match tokio::fs::read_dir(self.root_path.as_str()).await {
            Ok(dir) => dir,
            Err(_) => return vec![],
        };

        let mut result = Vec::new();

        while let Some(entry) = dir.next_entry().await.unwrap_or_else(|err| {
            panic!(
                "containers_storage: can not read dir {}. Err: {:?}",
                self.root_path, err
            )
        }) {
            let is_dir = match entry.file_type().await {
                Ok(file_type) => file_type.is_dir(),
                Err(_) => false,
            };

            if !is_dir {
                continue;
            }

            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }

        result.sort();

        result
    }

    fn compile_container_dir(&self, container_name: &str) -> String {
        format!("{}/{}", self.root_path, container_name)
    }

    fn compile_container_info_file_name(container_dir: &str) -> String {
        format!(
            "{}/{}",
            container_dir,
            crate::consts::CONTAINER_INFO_FILE_NAME
        )
    }

    fn compile_blob_file_name(container_dir: &str, hash: &str) -> String {
        format!("{}/{}.zip", container_dir, hash)
    }

    async fn load_container_info(container_dir: &str) -> Option<ContainerInfoYamlModel> {
        let file_name = Self::compile_container_info_file_name(container_dir);

        let content = tokio::fs::read(file_name.as_str()).await.ok()?;

        let result = serde_yaml::from_slice(content.as_slice()).unwrap_or_else(|err| {
            panic!(
                "containers_storage: can not deserialize {}. Err: {:?}",
                file_name, err
            )
        });

        Some(result)
    }

    async fn save_container_info(container_dir: &str, container_info: &ContainerInfoYamlModel) {
        let content = serde_yaml::to_string(container_info)
            .unwrap_or_else(|err| panic!("containers_storage: can not serialize. Err: {:?}", err));

        let file_name = Self::compile_container_info_file_name(container_dir);

        Self::write_file(file_name.as_str(), content.as_bytes()).await;
    }

    async fn file_exists(file_name: &str) -> bool {
        tokio::fs::metadata(file_name).await.is_ok()
    }

    /// Writes into `{file_name}.tmp` and renames — a crash mid-write can never
    /// leave a half-written zip or yaml behind the final name.
    async fn write_file(file_name: &str, content: &[u8]) {
        let tmp_file_name = format!("{}.tmp", file_name);

        tokio::fs::write(tmp_file_name.as_str(), content)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "containers_storage: can not write {}. Err: {:?}",
                    tmp_file_name, err
                )
            });

        tokio::fs::rename(tmp_file_name.as_str(), file_name)
            .await
            .unwrap_or_else(|err| {
                panic!(
                    "containers_storage: can not rename {} to {}. Err: {:?}",
                    tmp_file_name, file_name, err
                )
            });
    }

    async fn delete_blob(container_dir: &str, hash: &str) -> bool {
        let file_name = Self::compile_blob_file_name(container_dir, hash);

        if let Err(err) = tokio::fs::remove_file(file_name.as_str()).await {
            LOGGER.write_error(
                "containers_storage::delete_blob",
                format!("{:?}", err),
                LogEventCtx::new().add("file_name", file_name),
            );

            return false;
        }

        true
    }
}
