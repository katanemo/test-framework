// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::hostcalls::{serial_utils::serialize_map, set_status};
use crate::types::*;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn set_expect_status(checks: bool) {
    if checks {
        set_status(ExpectStatus::Expected)
    } else {
        set_status(ExpectStatus::Failed);
    }
}

// Global structure for handling low-level expectation structure (staged)
pub struct ExpectHandle {
    pub staged: Expect,
}

impl ExpectHandle {
    pub fn new() -> ExpectHandle {
        ExpectHandle {
            staged: Expect::new(false),
        }
    }

    pub fn update_stage(&mut self, allow_unexpected: bool) {
        self.staged = Expect::new(allow_unexpected);
    }

    pub fn assert_stage(&self) {
        if self.staged.expect_count > 0 {
            panic!(
                "Error: failed to consume all expectations - total remaining: {}",
                self.staged.expect_count
            );
        } else if self.staged.expect_count < 0 {
            panic!(
                "Error: expectations failed to account for all host calls by {} \n\
            if this is intended, please use --allow-unexpected (-a) mode",
                -1 * self.staged.expect_count
            );
        }
    }

    pub fn print_staged(&self) {
        println!("{:?}", self.staged);
    }
}

// Structure for setting low-level expectations over specific host functions
#[derive(Debug)]
pub struct Expect {
    allow_unexpected: bool,
    pub expect_count: i32,
    log_message: Vec<(Option<i32>, Option<String>)>,
    tick_period_millis: Vec<Option<Duration>>,
    current_time_nanos: Vec<Option<SystemTime>>,
    get_buffer_bytes: Vec<(Option<i32>, Option<Bytes>)>,
    set_buffer_bytes: Vec<(Option<i32>, Option<Bytes>)>,
    get_header_map_pairs: Vec<(Option<i32>, Option<Bytes>)>,
    set_header_map_pairs: Vec<(Option<i32>, Option<Bytes>)>,
    get_header_map_value: Vec<(Option<i32>, Option<String>, Option<String>)>,
    replace_header_map_value: Vec<(Option<i32>, Option<String>, Option<String>)>,
    remove_header_map_value: Vec<(Option<i32>, Option<String>)>,
    add_header_map_value: Vec<(Option<i32>, Option<String>, Option<String>)>,
    send_local_response: Vec<(Option<i32>, Option<String>, Option<Bytes>, Option<i32>)>,
    http_call: Vec<(
        Option<String>,
        Option<Bytes>,
        Option<String>,
        Option<Bytes>,
        Option<Duration>,
        Option<u32>,
    )>,
    metrics_create: Vec<(i32, String)>,
    metrics_increment: Vec<(i32, i64)>,
    metrics_record: Vec<(i32, u64)>,
    metrics_get: Vec<(i32, u64)>,
}

impl Expect {
    pub fn new(allow_unexpected: bool) -> Expect {
        Expect {
            allow_unexpected: allow_unexpected,
            expect_count: 0,
            log_message: vec![],
            tick_period_millis: vec![],
            current_time_nanos: vec![],
            get_buffer_bytes: vec![],
            set_buffer_bytes: vec![],
            get_header_map_pairs: vec![],
            set_header_map_pairs: vec![],
            get_header_map_value: vec![],
            replace_header_map_value: vec![],
            remove_header_map_value: vec![],
            add_header_map_value: vec![],
            send_local_response: vec![],
            http_call: vec![],
            metrics_create: vec![],
            metrics_increment: vec![],
            metrics_record: vec![],
            metrics_get: vec![],
        }
    }

    #[named]
    pub fn set_expect_log(&mut self, log_level: Option<i32>, log_string: Option<&str>) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.log_message
            .push((log_level, log_string.map(|s| s.to_string())));
    }

    #[named]
    pub fn get_expect_log(&mut self, log_level: i32, log_string: &str) {
        match self.log_message.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let log_tuple = self.log_message.remove(0);
                let mut expect_status = log_level == log_tuple.0.unwrap_or(log_level);
                expect_status =
                    expect_status && log_string == log_tuple.1.unwrap_or(log_string.to_string());
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_set_tick_period_millis(&mut self, tick_period_millis: Option<u64>) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.tick_period_millis
            .push(tick_period_millis.map(|period| Duration::from_millis(period)));
    }

    #[named]
    pub fn get_expect_set_tick_period_millis(&mut self, tick_period_millis: u128) {
        match self.tick_period_millis.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expect_status = tick_period_millis
                    == self
                        .tick_period_millis
                        .remove(0)
                        .map(|period| period.as_millis())
                        .unwrap_or(tick_period_millis);
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_get_current_time_nanos(&mut self, current_time_nanos: Option<u64>) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.current_time_nanos.push(
            current_time_nanos.map(|time_nanos| UNIX_EPOCH + Duration::from_nanos(time_nanos)),
        );
    }

    #[named]
    pub fn get_expect_get_current_time_nanos(&mut self) -> Option<u128> {
        match self.current_time_nanos.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
                None
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                set_status(ExpectStatus::Expected);
                self.current_time_nanos
                    .remove(0)
                    .map(|time_nanos| time_nanos.duration_since(UNIX_EPOCH).unwrap().as_nanos())
            }
        }
    }

    #[named]
    pub fn set_expect_get_buffer_bytes(
        &mut self,
        buffer_type: Option<i32>,
        buffer_data: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.get_buffer_bytes.push((
            buffer_type,
            buffer_data.map(|data| data.as_bytes().to_vec()),
        ));
    }

    #[named]
    pub fn get_expect_get_buffer_bytes(&mut self, buffer_type: i32) -> Option<Bytes> {
        match self.get_buffer_bytes.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
                None
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expect_status =
                    buffer_type == self.get_buffer_bytes[0].0.unwrap_or(buffer_type);
                set_expect_status(expect_status);
                self.get_buffer_bytes.remove(0).1
            }
        }
    }

    #[named]
    pub fn set_expect_set_buffer_bytes(
        &mut self,
        buffer_type: Option<i32>,
        buffer_data: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.set_buffer_bytes.push((
            buffer_type,
            buffer_data.map(|data| data.as_bytes().to_vec()),
        ));
    }

    #[named]
    pub fn get_expect_set_buffer_bytes(&mut self, buffer_type: i32, buffer_data: &[u8]) {
        match self.set_buffer_bytes.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expect_buffer = self.set_buffer_bytes.remove(0);
                let mut expect_status = buffer_type == expect_buffer.0.unwrap_or(buffer_type);
                expect_status = expect_status
                    && &buffer_data == &&expect_buffer.1.unwrap_or(buffer_data.to_vec())[..];
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_get_header_map_pairs(
        &mut self,
        map_type: Option<i32>,
        header_map_pairs: Option<Vec<(&str, &str)>>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.get_header_map_pairs
            .push((map_type, header_map_pairs.map(|map| serialize_map(map))));
    }

    #[named]
    pub fn get_expect_get_header_map_pairs(&mut self, map_type: i32) -> Option<Bytes> {
        match self.get_header_map_pairs.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
                None
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expect_status = map_type == self.get_header_map_pairs[0].0.unwrap_or(map_type);
                set_expect_status(expect_status);
                self.get_header_map_pairs.remove(0).1
            }
        }
    }

    #[named]
    pub fn set_expect_set_header_map_pairs(
        &mut self,
        map_type: Option<i32>,
        header_map_pairs: Option<Vec<(&str, &str)>>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.set_header_map_pairs
            .push((map_type, header_map_pairs.map(|map| serialize_map(map))));
    }

    #[named]
    pub fn get_expect_set_header_map_pairs(&mut self, map_type: i32, header_map_pairs: &[u8]) {
        match self.set_header_map_pairs.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let mut expect_status =
                    map_type == self.set_header_map_pairs[0].0.unwrap_or(map_type);
                expect_status = expect_status
                    && &header_map_pairs
                        == &&self
                            .set_header_map_pairs
                            .remove(0)
                            .1
                            .unwrap_or(header_map_pairs.to_vec())[..];
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_get_header_map_value(
        &mut self,
        map_type: Option<i32>,
        header_map_key: Option<&str>,
        header_map_value: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.get_header_map_value.push((
            map_type,
            header_map_key.map(|key| key.to_string()),
            header_map_value.map(|value| value.to_string()),
        ));
    }

    #[named]
    pub fn get_expect_get_header_map_value(
        &mut self,
        map_type: i32,
        header_map_key: &str,
    ) -> Option<String> {
        match self.get_header_map_value.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
                None
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let header_map_tuple = self.get_header_map_value.remove(0);
                let mut expect_status = map_type == header_map_tuple.0.unwrap_or(map_type);
                expect_status = expect_status
                    && header_map_key == &header_map_tuple.1.unwrap_or(header_map_key.to_string());
                set_expect_status(expect_status);
                header_map_tuple.2
            }
        }
    }

    #[named]
    pub fn set_expect_replace_header_map_value(
        &mut self,
        map_type: Option<i32>,
        header_map_key: Option<&str>,
        header_map_value: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.replace_header_map_value.push((
            map_type,
            header_map_key.map(|key| key.to_string()),
            header_map_value.map(|value| value.to_string()),
        ));
    }

    #[named]
    pub fn get_expect_replace_header_map_value(
        &mut self,
        map_type: i32,
        header_map_key: &str,
        header_map_value: &str,
    ) {
        match self.replace_header_map_value.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let header_map_tuple = self.replace_header_map_value.remove(0);
                let mut expect_status = map_type == header_map_tuple.0.unwrap_or(map_type);
                expect_status = expect_status
                    && header_map_key == &header_map_tuple.1.unwrap_or(header_map_key.to_string());
                expect_status = expect_status
                    && header_map_value
                        == &header_map_tuple.2.unwrap_or(header_map_value.to_string());
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_remove_header_map_value(
        &mut self,
        map_type: Option<i32>,
        header_map_key: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.remove_header_map_value
            .push((map_type, header_map_key.map(|key| key.to_string())));
    }

    #[named]
    pub fn get_expect_remove_header_map_value(&mut self, map_type: i32, header_map_key: &str) {
        match self.remove_header_map_value.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let header_map_tuple = self.remove_header_map_value.remove(0);
                let mut expect_status = map_type == header_map_tuple.0.unwrap_or(map_type);
                expect_status = expect_status
                    && header_map_key == &header_map_tuple.1.unwrap_or(header_map_key.to_string());
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_add_header_map_value(
        &mut self,
        map_type: Option<i32>,
        header_map_key: Option<&str>,
        header_map_value: Option<&str>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.add_header_map_value.push((
            map_type,
            header_map_key.map(|key| key.to_string()),
            header_map_value.map(|value| value.to_string()),
        ));
    }

    #[named]
    pub fn get_expect_add_header_map_value(
        &mut self,
        map_type: i32,
        header_map_key: &str,
        header_map_value: &str,
    ) {
        match self.add_header_map_value.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let header_map_tuple = self.add_header_map_value.remove(0);
                let mut expect_status = map_type == header_map_tuple.0.unwrap_or(map_type);
                expect_status = expect_status
                    && header_map_key == &header_map_tuple.1.unwrap_or(header_map_key.to_string());
                expect_status = expect_status
                    && header_map_value
                        == &header_map_tuple.2.unwrap_or(header_map_value.to_string());
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_send_local_response(
        &mut self,
        status_code: Option<i32>,
        body: Option<&str>,
        headers: Option<Vec<(&str, &str)>>,
        grpc_status: Option<i32>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.send_local_response.push((
            status_code,
            body.map(|data| data.to_string()),
            headers.map(|data| serialize_map(data)),
            grpc_status,
        ))
    }

    #[named]
    pub fn get_expect_send_local_response(
        &mut self,
        status_code: i32,
        body: Option<&str>,
        headers: &[u8],
        grpc_status: i32,
    ) {
        match self.send_local_response.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let local_response_tuple = self.send_local_response.remove(0);
                let mut expect_status =
                    status_code == local_response_tuple.0.unwrap_or(status_code);
                expect_status = expect_status
                    && body.unwrap_or("default")
                        == &local_response_tuple
                            .1
                            .unwrap_or(body.unwrap_or("default").to_string());
                expect_status = expect_status
                    && &headers == &&local_response_tuple.2.unwrap_or(headers.to_vec())[..];
                expect_status =
                    expect_status && grpc_status == local_response_tuple.3.unwrap_or(grpc_status);
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_http_call(
        &mut self,
        upstream: Option<&str>,
        headers: Option<Vec<(&str, &str)>>,
        body: Option<&str>,
        trailers: Option<Vec<(&str, &str)>>,
        timeout: Option<u64>,
        token_id: Option<u32>,
    ) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.http_call.push((
            upstream.map(|data| data.to_string()),
            headers.map(|data| serialize_map(data)),
            body.map(|data| data.to_string()),
            trailers.map(|data| serialize_map(data)),
            timeout.map(|data| Duration::from_millis(data)),
            token_id,
        ));
    }

    #[named]
    pub fn get_expect_http_call(
        &mut self,
        upstream: &str,
        headers: &[u8],
        body: Option<&str>,
        trailers: &[u8],
        timeout: i32,
    ) -> Option<u32> {
        match self.http_call.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
                None
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let http_call_tuple = self.http_call.remove(0);
                let mut expect_status =
                    upstream == &http_call_tuple.0.unwrap_or(upstream.to_string());
                expect_status = expect_status
                    && &headers == &&http_call_tuple.1.unwrap_or(headers.to_vec())[..];
                expect_status = expect_status
                    && body.unwrap_or("default")
                        == &http_call_tuple
                            .2
                            .unwrap_or(body.unwrap_or("default").to_string());
                expect_status = expect_status
                    && &trailers == &&http_call_tuple.3.unwrap_or(trailers.to_vec())[..];
                expect_status = expect_status
                    && timeout
                        == http_call_tuple
                            .4
                            .map(|data| data.as_millis() as i32)
                            .unwrap_or(timeout);
                set_expect_status(expect_status);
                http_call_tuple.5
            }
        }
    }

    #[named]
    pub fn set_expect_metric_create(&mut self, metric_type: i32, name: &str) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.metrics_create.push((metric_type, name.to_string()));
    }

    #[named]
    pub fn get_expect_metric_create(&mut self, metric_type: i32, name: &str) {
        match self.metrics_create.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expected_metric_type = self.metrics_create.remove(0);
                let expect_status = expected_metric_type == (metric_type, name.to_string());
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_metric_increment(&mut self, metric_id: i32, offset: i64) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.metrics_increment.push((metric_id, offset));
    }

    #[named]
    pub fn get_expect_metric_increment(&mut self, metric_id: i32, offset: i64) {
        match self.metrics_increment.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expected_metric_increment_tuple = self.metrics_increment.remove(0);
                let expect_status = expected_metric_increment_tuple == (metric_id, offset);
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_metric_record(&mut self, metric_id: i32, value: u64) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.metrics_record.push((metric_id, value));
    }

    #[named]
    pub fn get_expect_metric_record(&mut self, metric_id: i32, value: u64) {
        match self.metrics_record.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expected_metric_record_tuple = self.metrics_record.remove(0);
                let expect_status = expected_metric_record_tuple == (metric_id, value);
                set_expect_status(expect_status);
            }
        }
    }

    #[named]
    pub fn set_expect_metric_get(&mut self, metric_id: i32, value: u64) {
        self.expect_count += 1;
        println!("Expected count increased in {}", function_name!());
        self.metrics_get.push((metric_id, value));
    }

    #[named]
    pub fn get_expect_metric_get(&mut self, metric_id: i32, value: u64) {
        match self.metrics_get.len() {
            0 => {
                if !self.allow_unexpected {
                    self.expect_count -= 1;
                    println!(
                        "Decreasing expected with no records in {}",
                        function_name!()
                    );
                }
                set_status(ExpectStatus::Unexpected);
            }
            _ => {
                self.expect_count -= 1;
                println!("Decreasing expected count in {}", function_name!());
                let expected_get_metric_tuple = self.metrics_get.remove(0);
                let expect_status = expected_get_metric_tuple == (metric_id, value);
                set_expect_status(expect_status);
            }
        }
    }
}
