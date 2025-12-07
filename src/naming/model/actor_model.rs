use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use actix::prelude::*;
use crate::naming::model::Instance;
use crate::raft::filestore::raftsnapshot::SnapshotWriterActor;
use crate::raft::filestore::model::SnapshotRecordDto;

/// 命名服务Raft请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamingRaftReq {
    /// 注册永久实例
    RegisterInstance {
        param: InstanceRegisterParam,
    },
    /// 更新永久实例
    UpdateInstance {
        param: InstanceRegisterParam,
    },
    /// 删除永久实例
    RemoveInstance {
        namespace_id: String,
        group_name: String,
        service_name: String,
        ip: String,
        port: u32,
    },
}

impl Message for NamingRaftReq {
    type Result = anyhow::Result<NamingRaftResult>;
}

/// 命名服务Raft响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamingRaftResult {
    InstanceInfo(Arc<Instance>),
    None,
}

/// 实例注册参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRegisterParam {
    pub ip: String,
    pub port: u32,
    pub weight: f32,
    pub enabled: bool,
    pub healthy: bool,
    pub ephemeral: bool,
    pub metadata: HashMap<String, String>,
    pub namespace_id: String,
    pub group_name: String,
    pub service_name: String,
    pub cluster_name: Option<String>,
    pub app_name: Option<String>,
}

/// 快照构建请求
#[derive(Debug, Clone, Message)]
#[rtype(result = "anyhow::Result<()>")]
pub struct SnapshotBuildRequest {
    pub writer: Addr<SnapshotWriterActor>,
}

/// 快照加载请求
#[derive(Debug, Clone, Message)]
#[rtype(result = "anyhow::Result<LoadResult>")]
pub struct SnapshotLoadRequest {
    pub record: SnapshotRecordDto,
}

/// 加载结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadResult {
    Success,
    Error(String),
}

impl Default for InstanceRegisterParam {
    fn default() -> Self {
        Self {
            ip: Default::default(),
            port: Default::default(),
            weight: 1f32,
            enabled: true,
            healthy: true,
            ephemeral: true,
            metadata: Default::default(),
            namespace_id: Default::default(),
            group_name: Default::default(),
            service_name: Default::default(),
            cluster_name: Some("DEFAULT".to_string()),
            app_name: None,
        }
    }
}