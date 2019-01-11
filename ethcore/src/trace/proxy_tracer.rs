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
use trace::trace::{VMTrace, RewardType};
use trace::{Tracer, VMTracer, FlatTrace};
use trace::streaming_tracer::{NoopStreamingTracer};
use trace::executive_tracer::{ExecutiveTracer, ExecutiveVMTracer};

/// Proxy tracer. Forwards everything to the inner impl.
#[derive(Default)]
pub struct ProxyTracer {
	inner_tracer: Box<ExecutiveTracer>,
	streaming_tracer: Box<NoopStreamingTracer>,
}

impl ProxyTracer {
	/// Constructs a new ProxyTracer.
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
		self.streaming_tracer.prepare_trace_call(params, depth, is_builtin);
		self.inner_tracer.prepare_trace_call(params, depth, is_builtin);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		self.streaming_tracer.prepare_trace_create(params);
		self.inner_tracer.prepare_trace_create(params);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		self.streaming_tracer.done_trace_call(gas_used, output);
		self.inner_tracer.done_trace_call(gas_used, output);
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		self.streaming_tracer.done_trace_create(gas_used, code, address);
		self.inner_tracer.done_trace_create(gas_used, code, address);
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		self.streaming_tracer.done_trace_failed(error);
		self.inner_tracer.done_trace_failed(error);
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		self.streaming_tracer.trace_suicide(address, balance, refund_address);
		self.inner_tracer.trace_suicide(address, balance, refund_address);
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		let rt = reward_type.clone();
		self.streaming_tracer.trace_reward(author, value, rt);
		self.inner_tracer.trace_reward(author, value, reward_type);
	}

	fn drain(self) -> Vec<FlatTrace> {
		self.streaming_tracer.drain();
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
}

impl VMTracer for ProxyVMTracer {
	type Output = VMTrace;

	fn drain(self) -> Option<VMTrace> {
		return self.inner_vmtracer.drain()
	}
}
