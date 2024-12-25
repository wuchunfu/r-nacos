use crate::common::model::privilege::{PrivilegeGroup, PrivilegeGroupFlags};
use crate::user::permission::UserRoleHelper;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, prost::Message, Serialize, Deserialize)]
pub struct UserDo {
    #[prost(string, tag = "1")]
    pub username: String,
    #[prost(string, tag = "2")]
    pub password: String,
    #[prost(string, tag = "3")]
    pub nickname: String,
    #[prost(uint32, tag = "4")]
    pub gmt_create: u32,
    #[prost(uint32, tag = "5")]
    pub gmt_modified: u32,
    #[prost(bool, tag = "6")]
    pub enable: bool,
    #[prost(string, repeated, tag = "7")]
    pub roles: ::prost::alloc::vec::Vec<String>,
    #[prost(map = "string, string", tag = "8")]
    pub extend_info:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub password_hash: Option<String>,
    #[prost(uint32, optional, tag = "10")]
    pub namespace_privilege_flags: Option<u32>,
    #[prost(string, repeated, tag = "11")]
    pub namespace_white_list: ::prost::alloc::vec::Vec<String>,
    #[prost(string, repeated, tag = "12")]
    pub namespace_black_list: ::prost::alloc::vec::Vec<String>,
}

impl UserDo {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        prost::Message::encode(self, &mut v).unwrap_or_default();
        v
    }

    pub fn from_bytes(v: &[u8]) -> anyhow::Result<Self> {
        Ok(prost::Message::decode(v)?)
    }

    pub fn build_namespace_privilege(&self) -> PrivilegeGroup<Arc<String>> {
        let namespace_privilege_flags =
            self.namespace_privilege_flags.clone().unwrap_or_default() as u8;
        let namespace_privilege =
            if namespace_privilege_flags & PrivilegeGroupFlags::ENABLE.bits() > 0 {
                let mut namespace_whitelist = HashSet::new();
                for item in &self.namespace_white_list {
                    namespace_whitelist.insert(Arc::new(item.clone()));
                }
                let mut namespace_black_list = HashSet::new();
                for item in &self.namespace_black_list {
                    namespace_black_list.insert(Arc::new(item.clone()));
                }
                PrivilegeGroup::new(
                    self.namespace_privilege_flags.unwrap_or_default() as u8,
                    Some(Arc::new(namespace_whitelist)),
                    Some(Arc::new(namespace_black_list)),
                )
            } else {
                PrivilegeGroup::all()
            };
        namespace_privilege
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub username: Arc<String>,
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub gmt_create: Option<i64>,
    pub gmt_modified: Option<i64>,
    pub enable: Option<bool>,
    pub roles: Option<Vec<Arc<String>>>,
    pub extend_info: Option<HashMap<String, String>>,
    pub namespace_privilege: Option<PrivilegeGroup<Arc<String>>>,
}

impl From<UserDo> for UserDto {
    fn from(value: UserDo) -> Self {
        let mut roles = vec![];
        for role in &value.roles {
            roles.push(UserRoleHelper::get_role(role));
        }
        let namespace_privilege = Some(value.build_namespace_privilege());
        Self {
            username: Arc::new(value.username),
            nickname: Some(value.nickname),
            //password: Some(value.password),
            //不直接返回密码
            password_hash: value.password_hash,
            password: None,
            gmt_create: Some(value.gmt_create as i64 * 1000),
            gmt_modified: Some(value.gmt_modified as i64 * 1000),
            enable: Some(value.enable),
            roles: Some(roles),
            extend_info: Some(value.extend_info),
            namespace_privilege,
        }
    }
}
