// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Streaming tracer.

use ethereum_types::{U256, Address};
use vm::{Error as VmError, ActionParams};
use trace::trace::{RewardType};
use trace::{Tracer, FlatTrace};

#[derive(Default)]
pub struct MqStreamingTracer {
}

impl Tracer for MqStreamingTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams, depth: usize, is_builtin: bool) {
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
	}

	fn done_trace_failed(&mut self, error: &VmError) {
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
	}

	fn drain(self) -> Vec<FlatTrace> {
		return vec![];
	}
}

#[derive(Default)]
pub struct NoopStreamingTracer {
}

impl Tracer for NoopStreamingTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams, depth: usize, is_builtin: bool) {
		info!(target: "tracer", "prepare_trace_call: {:?} - {:?} - {:?}", params, depth, is_builtin);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		info!(target: "tracer", "prepare_trace_create: {:?}", params);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		info!(target: "tracer", "done_trace_call: {:?} - {:?}", gas_used, output);
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		info!(target: "tracer", "done_trace_create: {:?} - {:?} - {:?}", gas_used, code, address);
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		info!(target: "tracer", "done_trace_failed: {:?}", error);
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		info!(target: "tracer", "trace_suicide: {:?} - {:?} - {:?}", address, balance, refund_address);
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		info!(target: "tracer", "trace_reward: {:?} - {:?} - {:?}", author, value, reward_type);
	}

	fn drain(self) -> Vec<FlatTrace> {
		return vec![];
	}
}