use std::{cell::RefCell, rc::Rc};

use rift_plugin_core::prelude::*;

use super::*;

struct ConsumerMock {
    n_calls: usize,
    data: Vec<f32>,
    time: Option<BlockTime>,
}

impl ConsumerMock {
    fn new() -> ConsumerCell<Self> {
        Rc::new(RefCell::new(ConsumerMock {
            n_calls: 0,
            data: Vec::new(),
            time: None,
        }))
    }
}

impl AudioConsumer for ConsumerMock {
    fn consume(&mut self, block: &[f32], _: ChannelsInfo, time: BlockTime) {
        self.data.extend_from_slice(block);
        self.n_calls += 1;
        self.time = Some(time);
    }
}

fn init_audio_accumulator() -> AudioAccumulator {
    // 1 channel and 4 max blocks
    AudioAccumulator::new::<10>(1, 4)
}

#[test]
fn test_add_slice() {
    let acc = init_audio_accumulator();
    let channel: Vec<f32> = vec![0., 1., 2., 3.];
    acc.push_slices(&mut [channel.as_slice()].into_iter(), None);

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);

    assert_eq!(acc.channels(), 1);
    assert_eq!(consumer.borrow().n_calls, 1);
    assert_eq!(consumer.borrow().data.len(), channel.len());
    assert_eq!(consumer.borrow().data, channel);
    assert!(consumer.borrow().time.is_some()); // Even if we pass none, this must be some, just no data inside
    assert_eq!(consumer.borrow().time.map(|t| t.seconds()), Some(None))
}

#[test]
fn test_add_slice_with_time_info() {
    let acc = init_audio_accumulator();
    let channel: Vec<f32> = vec![0., 1., 2., 3.];
    let infos = BlockInfo::new(0., 0., 44100., 60.);
    acc.push_slices(&mut [channel.as_slice()].into_iter(), Some(infos));

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);

    assert_eq!(consumer.borrow().time.map(|t| t.seconds()), Some(Some(0.)));
    assert_eq!(acc.num_writes(), 1);
}

#[test]
fn test_clear() {
    let acc = init_audio_accumulator();
    let channel: Vec<f32> = vec![0., 1., 2., 3.];
    acc.push_slices(&mut [channel.as_slice()].into_iter(), None);
    acc.clear();

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);
    assert_eq!(consumer.borrow().n_calls, 0);
}

#[test]
fn test_push_slice_more_than_block_size() {
    let acc = init_audio_accumulator();
    let channel: Vec<f32> = (0..40).map(|i| i as f32).collect();
    acc.push_slices(&mut [channel.as_slice()].into_iter(), None);

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);

    assert_eq!(acc.channels(), 1);
    assert_eq!(consumer.borrow().n_calls, 4);
    assert_eq!(consumer.borrow().data, channel);
}

#[test]
fn test_push_slice_exceed_queue() {
    let acc = init_audio_accumulator();
    let channel: Vec<f32> = (0..50).map(|i| i as f32).collect();
    acc.push_slices(&mut [channel.as_slice()].into_iter(), None);

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);

    assert_eq!(consumer.borrow().n_calls, 4);
    assert!(consumer.borrow().data.len() < channel.len());
    assert_eq!(consumer.borrow().data, channel[..40]);
}

#[test]
fn test_no_channels() {
    let acc = AudioAccumulator::new::<10>(0, 4);
    let channel: Vec<f32> = (0..50).map(|i| i as f32).collect();
    acc.push_slices(&mut [channel.as_slice()].into_iter(), None);

    let consumer = ConsumerMock::new();
    acc.drain(&[consumer.clone()]);
}
