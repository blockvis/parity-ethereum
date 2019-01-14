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

use bytes::Bytes;
use ethereum_types::{U256, Address};
use vm::{Error as VmError, ActionParams, CallType};
use trace::trace::{Action, Create, RewardType};
use trace::{Tracer, FlatTrace};

use failure::Error;
use futures::future::Future;
use tokio::net::TcpStream;
use lapin::types::FieldTable;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicProperties, BasicPublishOptions, ExchangeDeclareOptions};

use std::convert::From;
use std::collections::BTreeMap;
use rustc_serialize::json::{self, ToJson, Json};

/// MQ Streaming payload. Incapsulates ActionParams & json-serializable.
pub struct MqStreamingPayload {
	code_address: Address,
	from: Address,
	to: Address,
	origin: Address,
	gas: U256,
	gas_price: U256,
	gas_used: U256,
	value: U256,
	input_data: Option<Bytes>,
	output_data: Option<Bytes>,
	call_type: CallType,
	reward_type: RewardType
}

impl Default for MqStreamingPayload {
	/// Returns default MqStreamingPayload initialized with zeros
	fn default() -> MqStreamingPayload {
		MqStreamingPayload {
			code_address: Address::new(),
			from: Address::new(),
			to: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			gas_used: U256::zero(),
			value: U256::zero(),
			input_data: None,
			output_data: None,
			call_type: CallType::None,
			reward_type: RewardType::EmptyStep
		}
	}
}

impl From<ActionParams> for MqStreamingPayload {
	fn from(a: ActionParams) -> Self {
		MqStreamingPayload {
			code_address: a.code_address.clone(),
			from: a.sender.clone(),
			to: a.address.clone(),
			origin: a.origin.clone(),
			gas: a.gas.clone(),
			gas_price: a.gas_price.clone(),
			gas_used: U256::zero(),
			value: a.value.value().clone(),
			input_data: a.data.clone(),
			output_data: None,
			call_type: a.call_type.clone(),
			reward_type: RewardType::EmptyStep
		}
	}
}

impl ToString for MqStreamingPayload {
	fn to_string(&self) -> String {
		let serialized: String = json::encode(&self.to_json()).unwrap();
		return serialized;
	}
}

impl ToJson for MqStreamingPayload {
    fn to_json(&self) -> Json {
		let mut map = BTreeMap::new();
		map.insert("code_address".to_string(), Json::String(format!("{}", self.code_address)));
		map.insert("from".to_string(), Json::String(format!("{}", self.from)));
		map.insert("to".to_string(), Json::String(format!("{}", self.to)));
		map.insert("origin".to_string(), Json::String(format!("{}", self.origin)));
		map.insert("gas".to_string(), Json::String(format!("{}", self.gas)));
		map.insert("gas_price".to_string(), Json::String(format!("{}", self.gas_price)));
		map.insert("gas_used".to_string(), Json::String(format!("{}", self.gas_used)));
		map.insert("value".to_string(), Json::String(format!("{}", self.value)));
		map.insert("input_data".to_string(), Json::String(format!("{}", "input_data")));
		map.insert("output_data".to_string(), Json::String(format!("{}", "output_data")));
		map.insert("call_type".to_string(), Json::String(format!("{:?}", self.call_type)));
		map.insert("reward_type".to_string(), Json::String(format!("{:?}", self.reward_type)));
		return Json::Object(map);
	}
}

pub type LapinClient = lapin::client::Client<TcpStream>;
pub type LapinChannel = lapin::channel::Channel<TcpStream>;

/// MQ Streaming tracer. Forwards everything to the MQ.
#[derive(Default)]
pub struct MqStreamingTracer {
	last_action_params: Option<ActionParams>,
	client: Option<LapinClient>,
	channel: Option<LapinChannel>
}

impl MqStreamingTracer {
	/// Constructs a new MqStreamingTracer.
	pub fn create() -> Self {
		let mut tracer = MqStreamingTracer {
			last_action_params: None,
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
			channel.exchange_declare("MqStreamingTracer", "fanout", ExchangeDeclareOptions::default(), FieldTable::new())
			.and_then(move |_| {
				info!("channel {} declared exchange {}", channel.id, "MqStreamingTracer");
				channel.basic_publish("MqStreamingTracer", "", b"PING".to_vec(), BasicPublishOptions::default(), BasicProperties::default())
			}).map_err(Error::from)
		}).wait();
	}

	fn post_to_channel(&mut self, payload: &MqStreamingPayload) {
		if let Some(ref channel) = self.channel {
			let data_str = payload.to_string();
			let data_bytes = data_str.as_bytes();
			channel.basic_publish("MqStreamingTracer", "", data_bytes.to_vec(), BasicPublishOptions::default(), BasicProperties::default())
				.wait()
				.expect("Error: basic_publish FAILED.");

		}
		else {
			error!("Error: post_to_channel FAILED.");
		}
	}
}

impl Tracer for MqStreamingTracer {
	type Output = FlatTrace;

	fn prepare_trace_call(&mut self, params: &ActionParams, depth: usize, is_builtin: bool) {
		self.last_action_params = Some(params.clone());

		info!(target: "tracer", "prepare_trace_call: {:?} - {:?} - {:?}", params, depth, is_builtin);
	}

	fn prepare_trace_create(&mut self, params: &ActionParams) {
		self.last_action_params = Some(params.clone());

		info!(target: "tracer", "prepare_trace_create: {:?}", params);
	}

	fn done_trace_call(&mut self, gas_used: U256, output: &[u8]) {
		if let Some(last_action_params) = self.last_action_params.clone() {
			let mut payload = MqStreamingPayload::from(last_action_params);

			// TODO: review params.
			payload.gas_used = gas_used.clone();
			payload.output_data = Some(output.clone().to_vec());

			self.post_to_channel(&payload);
		}
		else {
			error!("Error: done_trace_call FAILED.");
		}

		self.last_action_params = None;

		info!(target: "tracer", "done_trace_call: {:?} - {:?}", gas_used, output);
	}

	fn done_trace_create(&mut self, gas_used: U256, code: &[u8], address: Address) {
		if let Some(last_action_params) = self.last_action_params.clone() {
			let mut payload = MqStreamingPayload::from(last_action_params);

			// TODO: review params.
			payload.gas_used = gas_used.clone();
			payload.code_address = address.clone();

			self.post_to_channel(&payload);
		}
		else {
			error!("Error: done_trace_create FAILED.");
		}

		self.last_action_params = None;

		info!(target: "tracer", "done_trace_create: {:?} - {:?} - {:?}", gas_used, code, address);
	}

	fn done_trace_failed(&mut self, error: &VmError) {
		let mut is_create = false;
		if let Some(last_action_params) = self.last_action_params.clone() {
			let last_action_params_cloned = last_action_params.clone();
			let action = Action::Create(Create::from(last_action_params_cloned));

			is_create = match action {
				Action::Create(_) => true,
				_ => false,
			};

			// TODO: review error & create.
			let payload = MqStreamingPayload::from(last_action_params);

			self.post_to_channel(&payload);
		}
		else {
			error!("Error: done_trace_failed FAILED.");
		}

		self.last_action_params = None;

		info!(target: "tracer", "done_trace_failed: {:?}", error);
	}

	fn trace_suicide(&mut self, address: Address, balance: U256, refund_address: Address) {
		let mut payload = MqStreamingPayload::default();
		
		// TODO: review params.
		payload.code_address = address.clone();
		payload.to = refund_address.clone();
		payload.value = balance.clone();

		self.post_to_channel(&payload);

		info!(target: "tracer", "trace_suicide: {:?} - {:?} - {:?}", address, balance, refund_address);
	}

	fn trace_reward(&mut self, author: Address, value: U256, reward_type: RewardType) {
		let mut payload = MqStreamingPayload::default();

		// TODO: review params.
		payload.to = author.clone();
		payload.value = value.clone();
		payload.reward_type = reward_type.clone();

		self.post_to_channel(&payload);

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