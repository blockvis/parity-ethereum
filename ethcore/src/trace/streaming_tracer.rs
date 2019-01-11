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

use failure::Error;
use futures::future::Future;
use tokio::net::TcpStream;
use lapin::types::FieldTable;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicProperties, BasicPublishOptions, ExchangeDeclareOptions};

pub type LapinClient = lapin::client::Client<TcpStream>;
pub type LapinChannel = lapin::channel::Channel<TcpStream>;

/// MQ Streaming tracer. Forwards everything to the MQ.
#[derive(Default)]
pub struct MqStreamingTracer {
	client: Option<LapinClient>,
	channel: Option<LapinChannel>
}

impl MqStreamingTracer {
	/// Constructs a new MqStreamingTracer.
	pub fn create() -> Self {
		let mut tracer = MqStreamingTracer {
			client: None,
			channel: None
		};
		tracer.create_channel();
		return tracer;
	}

	fn create_channel(&mut self) {
		let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "127.0.0.1:5672".to_string()).parse().unwrap();

		let _result = TcpStream::connect(&addr)
		.map_err(Error::from)
		.and_then(|stream| {
				lapin::client::Client::connect(
					stream,
					ConnectionOptions{
						frame_max: 65535,
						..Default::default()
					}
				).map_err(Error::from)
		})
		.and_then(|(client, _heartbeat)| {
			client.create_channel().map_err(Error::from)
		})
		.and_then(|channel| {
			//self.client = Some(client.clone());
			self.channel = Some(channel.clone());
			info!("created channel with id: {}", channel.id);
			channel.exchange_declare("StreamingTracer", "fanout", ExchangeDeclareOptions::default(), FieldTable::new())
			.and_then(move |_| {
				info!("channel {} declared exchange {}", channel.id, "StreamingTracer");
				channel.basic_publish("StreamingTracer", "", b"PING".to_vec(), BasicPublishOptions::default(), BasicProperties::default())
			}).map_err(Error::from)
		}).wait();
	}
}

impl Tracer for MqStreamingTracer {
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

/// Noop tracer. Forwards everything to the log.
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