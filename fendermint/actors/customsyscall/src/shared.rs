// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use num_derive::FromPrimitive;

pub const CUSTOMSYSCALL_ACTOR_NAME: &str = "customsyscall";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct InvokeParams {
    pub user_index: i64,
    pub user_activity_matrix: [u8; 1000],
    pub k: i64,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Invoke = frc42_dispatch::method_hash!("Invoke"),
}
