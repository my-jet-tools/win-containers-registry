pub struct UploadContainerBlobParams {
    pub container_name: String,
    pub tag: String,
    pub hash: String,
    pub content: Vec<u8>,
    pub uploaded_by: String,
}

pub struct UploadedContainerTagModel {
    pub container_name: String,
    pub tag: String,
    pub hash: String,
    pub size: u64,
    /// Hash the tag was pointing at before this upload.
    pub replaced_hash: Option<String>,
    /// True when the replaced hash became unreferenced and its zip was deleted.
    pub orphan_deleted: bool,
}

pub struct DownloadedContainerTagModel {
    pub file_name: String,
    pub content: Vec<u8>,
}

pub struct ContainerTagModel {
    pub container_name: String,
    pub tag: String,
    pub hash: String,
    pub size: u64,
    pub uploaded_at: String,
    pub uploaded_by: String,
}

pub struct ContainerListItemModel {
    pub container: String,
    pub tags_amount: usize,
}

pub struct DeletedContainerTagModel {
    pub hash: String,
    pub orphan_deleted: bool,
}
