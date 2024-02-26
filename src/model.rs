use chrono::{DateTime, Local};
use rustcracker::{
    components::machine::{Config, MachineCore},
    model::{full_vm_configuration::FullVmConfiguration, instance_info::InstanceInfo},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateRequest {
    pub config: MachineCreateConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateResponse {
    pub vmid: Uuid,
    pub created_at: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmQueryStatusRequest {
    pub vmid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmQueryStatusResponse {
    pub vmid: Uuid,
    pub info: VmViewInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Operation {
    Start,
    Pause,
    Resume,
    Stop,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmOperateRequest {
    pub vmid: Uuid,
    pub operation: Operation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmOperateResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteRequest {
    pub vmid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateSnapshotRequest {
    pub vmid: Uuid,
    pub snapshot_path: String,
    pub memory_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmLoadSnapshotRequest {
    pub vmid: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmModifyMetadataRequest {
    pub vmid: Uuid,
    pub metadata: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmModifyMetadataResponse {
    pub vmid: Uuid,
    pub time: chrono::DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmSnapshotDetailRequest {
    pub vmid: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmRestoreAllRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmRestoreAllResponse {
    pub infos: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateVMMSRequest {
    pub vmid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmCreateVMMSResponse {
    pub vmid: Uuid,
    pub vm_mem_snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteVMMSRequest {
    pub vmid: Uuid,
    pub vm_mem_snapshot_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmDeleteVMMSResponse {
    pub vmid: Uuid,
    pub vm_mem_snapshot_id: Uuid,
}

// Schemas

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MachineCreateConfig {
    pub memory_size_in_mib: i32,
    pub vcpu_count: i32,
    pub kernel_name: String,
    pub kernel_version: String,
    pub enable_hyperthreading: Option<bool>,
    pub initial_metadata: Option<String>,
    pub volume_size_in_mib: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmViewConfig {
    user_id: Option<Uuid>,
    vmid: Option<Uuid>,
    config: Option<sqlx::types::Json<FullVmConfiguration>>,
    execute_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VmViewInfo {
    pub vmid: Uuid,
    pub vm_info: InstanceInfo,
    pub full_config: FullVmConfiguration,
    pub boot_config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotInfo {
    pub snapshot_id: Uuid,
    pub snapshot_path: String,
    pub memory_path: String,
    pub created_at: DateTime<Local>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct PgMachineCoreElement {
    pub vmid: Uuid,
    pub core: sqlx::types::Json<MachineCore>,
    pub status: MachineState,
}

pub type MachineState = i8;
pub const CREATED: MachineState = 1;
pub const RUNNING: MachineState = 2;
pub const PAUSED: MachineState = 3;
pub const STOPPED: MachineState = 4;
pub const DELETED: MachineState = 5;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct PgVolumeElement {
    pub vmid: Uuid,
    pub volume_id: Uuid,
}
