pub(crate) const MACHINE_CORE_TABLE_NAME: &'static str = "MACHINE_CORE_TABLE_NAME";
pub(crate) const DEFAULT_MACHINE_CORE_TABLE: &'static str = "machine_core";

pub(crate) const VM_CONFIG_TABLE_NAME: &'static str = "VM_CONFIG_TABLE_NAME";
pub(crate) const DEFAULT_VM_CONFIG_TABLE: &'static str = "vmconfig";

pub(crate) const VM_MEM_SNAPSHOT_TABLE_NAME: &'static str = "SNAPSHOT_TABLE_NAME";
pub(crate) const DEFAULT_VM_MEM_SNAPSHOT_TABLE: &'static str = "snapshots";

pub(crate) const VOLUME_TABLE_NAME: &'static str = "VOLUME_TABLE_NAME";
pub(crate) const DEFAULT_VOLUME_TABLE: &'static str = "volume";

pub(crate) const CREATE_VMVIEWCONFIGS_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        vmid                UUID PRIMARY KEY,
        config              JSON
    );
"#;
pub(crate) const DROP_VMVIEWCONFIGS_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;
pub(crate) const INSERT_VMVIEWCONFIGS_BY_VMID: &'static str = r#"
    INSERT INTO $1 (vmid, config) VALUES ($2, $3);
"#;
pub(crate) const DELETE_VMVIEWCONFIGS_BY_VMID: &'static str = r#"
    DELETE * FROM $1 WHERE vmid = $2;
"#;

pub(crate) const CREATE_MACHINE_CORE_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        vmid                UUID,
        core                JSON,
        status              INT,
    );
"#;
pub(crate) const DROP_MAHCINE_CORE_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;
pub(crate) const GET_MACHINE_CORE_BY_VMID: &'static str = r#"
    SELECT * FROM $1 WHERE vmid = $2;
"#;
pub(crate) const INSERT_MACHINE_CORE_BY_VMID: &'static str = r#"
    INSERT INTO $1 (vmid, core, status) VALUES ($2, $3, $4);
"#;
pub(crate) const DELETE_MACHINE_CORE_BY_VMID: &'static str = r#"
    DELETE * FROM $1 WHERE vmid = $2;
"#;
pub(crate) const UPDATE_MACHINE_CORE_BY_VMID: &'static str = r#"
    UPDATE $1 SET core = $2, status = $3 WHERE vmid = $4
"#;

pub(crate) const CREATE_VM_MEM_SNAPSHOT_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        vmid                UUID,
        snapshot_id         UUID,
        mem_file_path       VARCHAR(256),
        snapshot_path       VARCHAR(256)
    );
"#;
pub(crate) const DROP_VM_MEM_SNAPSHOT_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;
pub(crate) const INSERT_VM_MEM_SNAPSHOT_BY_ID: &'static str = r#"
    INSERT INTO $1 (vmid, snapshot_id, mem_file_path, snapshot_path)
    VALUES ($2, $3, $4, $5);
"#;
pub(crate) const DELETE_VM_MEM_SNAPSHOT_BY_ID: &'static str = r#"
    DELETE * FROM $1 WHERE vmid = $2 AND snapshot_id = $3;
"#;

pub(crate) const CREATE_VOLUME_TABLE_SQL: &'static str = r#"
    CREATE TABLE if not exists $1 (
        vmid                UUID,
        volume_id           UUID
    );
"#;
pub(crate) const DROP_VOLUME_TABLE_SQL: &'static str = r#"
    DROP TABLE if exists $1;
"#;
pub(crate) const INSERT_VOLUME_BY_ID: &'static str = r#"
    INSERT INTO $1 (vmid, volume_id)
    VALUES ($2, $3);
"#;
pub(crate) const DELETE_VOLUME_BY_ID: &'static str = r#"
    DELETE * FROM $1 WHERE vmid = $2 AND volume_id = $3;
"#;
pub(crate) const GET_VOLUME_ALL: &'static str = r#"
    SELECT * FROM $1 WHERE vmid = $2;
"#;
