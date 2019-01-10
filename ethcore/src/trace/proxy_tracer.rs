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

//! Proxy tracer.

use ethereum_types::{U256, Address};
use vm::{Error as VmError, ActionParams};
use trace::trace::{Call, Create, Action, Res, CreateResult, CallResult, VMTrace, VMOperation, VMExecutedOperation, MemoryDiff, StorageDiff, Suicide, Reward, RewardType};
use trace::{Tracer, VMTracer, FlatTrace};
use trace::streaming_tracer::{StreamingTracer, NoopStreamingTracer};
use trace::executive_tracer::{ExecutiveTracer, ExecutiveVMTracer};

/// Proxy tracer. Forwards everything to the inner impl.
#[derive(Default)]
pub struct ProxyTracer {
	inner_tracer: Box<ExecutiveTracer>,
	streaming_tracer: Box<NoopStreamingTracer>,
}

impl ProxyTracer {
	pub fn create(inner: ExecutiveTracer) -> Self {
		return ProxyTracer {
			inner_tracer: Box::new(inner),
			streaming_tracer: Box::new(NoopStreamingTracer::default())
		}
	}
}

impl Tracer for ProxyTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams, depth: usize, is_builtin: bool) {
		self.inner_tracer.prepare_trace_call(params, depth, is_builtin);
		info!(target: "tracer", "prepare_trace_call: {:?} - {:?} - {:?}", params, depth, is_builtin);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		self.inner_tracer.prepare_trace_create(params);
		info!(target: "tracer", "prepare_trace_create: {:?}", params);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		self.inner_tracer.done_trace_call(gas_used, output);
		info!(target: "tracer", "done_trace_call: {:?} - {:?}", gas_used, output);
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		self.inner_tracer.done_trace_create(gas_used, code, address);
		info!(target: "tracer", "done_trace_create: {:?} - {:?} - {:?}", gas_used, code, address);
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		self.inner_tracer.done_trace_failed(error);
		info!(target: "tracer", "done_trace_failed: {:?}", error);
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		self.inner_tracer.trace_suicide(address, balance, refund_address);
		info!(target: "tracer", "trace_suicide: {:?} - {:?} - {:?}", address, balance, refund_address);
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		let rt = reward_type.clone();
		self.inner_tracer.trace_reward(author, value, reward_type);
		info!(target: "tracer", "trace_reward: {:?} - {:?} - {:?}", author, value, rt);
	}

	fn drain(self) -> Vec<FlatTrace> {
		return self.inner_tracer.drain();
	}
}

/// Proxy VM tracer. Forwards everything to the inner impl.
#[derive(Default)]
pub struct ProxyVMTracer {
	inner_vmtracer: Box<ExecutiveVMTracer>
}

impl ProxyVMTracer {
	/// Create a new top-level instance.
	pub fn toplevel() -> Self {
		return ProxyVMTracer {
			inner_vmtracer: Box::new(ExecutiveVMTracer::toplevel())
		}
	}

	fn with_trace_in_depth<F: Fn(&mut VMTrace)>(trace: &mut VMTrace, depth: usize, f: F) {
		if depth == 0 {
			f(trace);
		} else {
			Self::with_trace_in_depth(trace.subs.last_mut().expect("self.depth is incremented with prepare_subtrace; a subtrace is always pushed; self.depth cannot be greater than subtrace stack; qed"), depth - 1, f);
		}
	}
}

impl VMTracer for ProxyVMTracer {
	type Output = VMTrace;

	fn drain(self) -> Option<VMTrace> {
		return self.inner_vmtracer.drain()
	}
}
