// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm::call_manager::CallManager;
use fvm::gas::Gas;
use fvm::kernel::prelude::*;
use fvm::kernel::Result;
use fvm::kernel::{
    ActorOps, CryptoOps, DebugOps, EventOps, IpldBlockOps, MessageOps, NetworkOps, RandomnessOps,
    SelfOps, SendOps, SyscallHandler, UpgradeOps,
};
use fvm::syscalls::Linker;
use fvm::DefaultKernel;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::randomness::RANDOMNESS_LENGTH;
use fvm_shared::sys::out::network::NetworkContext;
use fvm_shared::sys::out::vm::MessageContext;
use fvm_shared::{econ::TokenAmount, ActorID, MethodNum};

use ambassador::Delegate;
use cid::Cid;

use cbor::{Decoder, Encoder};
use ethers::{
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    // types::Address,
};
use reqwest::blocking::get;
use std::sync::Arc;

// we define a single custom syscall which fetch recommendations from stylus contract
pub trait CustomKernel: Kernel {
    fn my_custom_syscall(
        &self,
        user_index: i64,
        // user_activity_matrix: [i64; 5],
        k: i64,
    ) -> Result<[u8; 1000]>;
}

// our custom kernel extends the filecoin kernel
#[derive(Delegate)]
#[delegate(IpldBlockOps, where = "C: CallManager")]
#[delegate(ActorOps, where = "C: CallManager")]
#[delegate(CryptoOps, where = "C: CallManager")]
#[delegate(DebugOps, where = "C: CallManager")]
#[delegate(EventOps, where = "C: CallManager")]
#[delegate(MessageOps, where = "C: CallManager")]
#[delegate(NetworkOps, where = "C: CallManager")]
#[delegate(RandomnessOps, where = "C: CallManager")]
#[delegate(SelfOps, where = "C: CallManager")]
#[delegate(SendOps<K>, generics = "K", where = "K: CustomKernel")]
#[delegate(UpgradeOps<K>, generics = "K", where = "K: CustomKernel")]
pub struct CustomKernelImpl<C>(pub DefaultKernel<C>);

impl<C> CustomKernel for CustomKernelImpl<C>
where
    C: CallManager,
    CustomKernelImpl<C>: Kernel,
{
    fn my_custom_syscall(
        &self,
        user_index: i64,
        user_activity_matrix: [u8; 1000],
        k: i64,
    ) -> Result<[u8; 1000]> {
        // currently this is not deterministic since sometimes the request is rate limited

        abigen!(
          IRecommendation,
          "[function getRecommendations(int64[][] memory user_activity_matrix, int64 user_index, int64 k) external view returns (int64[][] memory)]"
        );

        let provider = Arc::new(Provider::try_from(
            "https://stylus-testnet.arbitrum.io/rpc",
        )?);

        let recommender = IRecommendation::new(
            "0xa69E3ccFd133A80B92CD93De555243416c19E566".parse()?,
            provider,
        );

        async {
            let recommendation_matrix = recommender
                .get_recommendations(user_activity_matrix.to_vec(), user_index, k)
                .call()
                .await?;

            let mut result: [u8; 1000] = [0; 1000];

            match recommendation_matrix {
                Err(_) => Ok(result),
            };
            let mut e = Encoder::from_memory();
            e.encode(&recommendation_matrix).unwrap();

            let bytes_arr = e.as_bytes().clone();
            let mut idx = 0;
            for i in bytes_arr.iter() {
                if idx == 1000 {
                    break;
                }
                result[idx] = i.clone();
            }
            Ok(result)
        };
    }
}

impl<C> Kernel for CustomKernelImpl<C>
where
    C: CallManager,
{
    type CallManager = C;
    type Limiter = <DefaultKernel<C> as Kernel>::Limiter;

    fn into_inner(self) -> (Self::CallManager, BlockRegistry)
    where
        Self: Sized,
    {
        self.0.into_inner()
    }

    fn new(
        mgr: C,
        blocks: BlockRegistry,
        caller: ActorID,
        actor_id: ActorID,
        method: MethodNum,
        value_received: TokenAmount,
        read_only: bool,
    ) -> Self {
        CustomKernelImpl(DefaultKernel::new(
            mgr,
            blocks,
            caller,
            actor_id,
            method,
            value_received,
            read_only,
        ))
    }

    fn machine(&self) -> &<Self::CallManager as CallManager>::Machine {
        self.0.machine()
    }

    fn limiter_mut(&mut self) -> &mut Self::Limiter {
        self.0.limiter_mut()
    }

    fn gas_available(&self) -> Gas {
        self.0.gas_available()
    }

    fn charge_gas(&self, name: &str, compute: Gas) -> Result<GasTimer> {
        self.0.charge_gas(name, compute)
    }
}

impl<K> SyscallHandler<K> for CustomKernelImpl<K::CallManager>
where
    K: CustomKernel
        + ActorOps
        + SendOps
        + UpgradeOps
        + IpldBlockOps
        + CryptoOps
        + DebugOps
        + EventOps
        + MessageOps
        + NetworkOps
        + RandomnessOps
        + SelfOps,
{
    fn link_syscalls(linker: &mut Linker<K>) -> anyhow::Result<()> {
        DefaultKernel::<K::CallManager>::link_syscalls(linker)?;

        linker.link_syscall("my_custom_kernel", "my_custom_syscall", my_custom_syscall)?;

        Ok(())
    }
}

pub fn my_custom_syscall(
    context: fvm::syscalls::Context<'_, impl CustomKernel>,
    user_index: i64,
    // user_activity_matrix: [i64; 5],
    k: i64,
) -> Result<[u8; 1000]> {
    context.kernel.my_custom_syscall(user_index, k)
}
