// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorError;

use crate::InvokeParams;
use crate::{Method, CUSTOMSYSCALL_ACTOR_NAME};

fil_actors_runtime::wasm_trampoline!(Actor);

fvm_sdk::sys::fvm_syscalls! {
    module = "my_custom_kernel";
    pub fn my_custom_syscall(
      user_index: i64,
      user_activity_matrix: Vec<Vec<i64>>,
      k: i64) -> Result<Vec<Vec<i64>>>;
}

pub struct Actor;
impl Actor {
    fn invoke(rt: &impl Runtime, params: InvokeParams) -> Result<Vec<Vec<i64>>, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        unsafe {
            let value = my_custom_syscall(params.user_index, params.user_activity_matrix, params.k)
                .unwrap();
            Ok(value)
        }
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        CUSTOMSYSCALL_ACTOR_NAME
    }

    actor_dispatch! {
        Invoke => invoke,
    }
}
