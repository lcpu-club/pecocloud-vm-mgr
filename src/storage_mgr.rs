//! RPC to storage management (naive with http)
use uuid::Uuid;

use crate::{error::VmManageResult, pool::VmPool, storage_models::*};

pub async fn create_volume(pool: &mut VmPool, size: i32, parent: Option<Uuid>) -> VmManageResult<Uuid> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/volume");
    let req = VolumeCreateRequest { size, parent };
    let res = pool
        .storage_client
        .post(url)
        .json(&req)
        .send()
        .await?
        .json::<VolumeCreateResponse>()
        .await?;

    Ok(res.volume)
}

pub async fn delete_volume(pool: &mut VmPool, volume: Uuid) -> VmManageResult<Uuid> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/volume");
    let req = VolumeDeleteRequest { volume };
    let res = pool
        .storage_client
        .delete(url)
        .json(&req)
        .send()
        .await?
        .json::<VolumeDeleteResponse>()
        .await?;

    Ok(res.volume)
}

/// Get the volume path
pub async fn attach_volume(pool: &mut VmPool, volume: Uuid) -> VmManageResult<String> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/volume/attach");
    let req = VolumeAttachRequest { volume };
    let res = pool
        .storage_client
        .post(url)
        .json(&req)
        .send()
        .await?
        .json::<VolumeAttachResponse>()
        .await?;

    Ok(res.device)
}

pub async fn detach_volume(pool: &mut VmPool, volume: Uuid) -> VmManageResult<Uuid> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/volume/detach");
    let req = VolumeDetachRequest { volume };
    let res = pool
        .storage_client
        .post(url)
        .json(&req)
        .send()
        .await?
        .json::<VolumeDetachResponse>()
        .await?;

    Ok(res.volume)
}

pub async fn create_volume_snapshot(pool: &mut VmPool, volume: Uuid) -> VmManageResult<Uuid> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/snapshot");
    let req = SnapshotCreateRequest { volume };
    let res = pool
        .storage_client
        .post(url)
        .json(&req)
        .send()
        .await?
        .json::<SnapshotCreateResponse>()
        .await?;

    Ok(res.volume)
}

pub async fn delete_volume_snapshot(
    pool: &mut VmPool,
    volume: Uuid,
    snapshot: Uuid,
) -> VmManageResult<()> {
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1/snapshot");
    let req = SnapshotDeleteRequest { volume, snapshot };
    let _ = pool.storage_client.post(url).json(&req).send().await?;

    Ok(())
}
