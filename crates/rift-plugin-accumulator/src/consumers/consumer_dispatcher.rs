use rift_plugin_core::prelude::{BlockTime, ChannelsInfo, ConsumerCell};

use crate::prelude::AudioConsumer;

pub enum ChannelMode {
    Averaged,
    Channel(usize),
}

struct ConsumerWithMode {
    consumer: ConsumerCell<dyn AudioConsumer>,
    mode: ChannelMode,
}

pub struct ConsumerDispatcher {
    intermediate: Vec<f32>,
    consumers: Vec<ConsumerWithMode>,
    has_average_consumer: bool,
}

impl ConsumerDispatcher {
    pub fn new() -> Self {
        Self {
            intermediate: Vec::new(),
            consumers: Vec::new(),
            has_average_consumer: false,
        }
    }

    pub fn add_consumer_averaged(&mut self, consumer: ConsumerCell<dyn AudioConsumer>) {
        let consumer_with_mode = ConsumerWithMode {
            consumer,
            mode: ChannelMode::Averaged,
        };

        self.has_average_consumer = true;
        self.consumers.push(consumer_with_mode);
    }

    pub fn add_consumer_at_channel(
        &mut self,
        consumer: ConsumerCell<dyn AudioConsumer>,
        channel: usize,
    ) {
        let consumer_with_mode = ConsumerWithMode {
            consumer,
            mode: ChannelMode::Channel(channel),
        };
        self.consumers.push(consumer_with_mode);
    }

    pub fn consumer_count(&self) -> usize {
        self.consumers.len()
    }

    pub fn dispatch(&mut self, block: &[f32], channels: ChannelsInfo, time: BlockTime) {
        let total_channel = channels.total_channels as f32;

        // At channel 0, we ensure our intermediate buffer is large enough
        if self.has_average_consumer {
            if channels.current == 0 {
                self.intermediate.clear();
                self.intermediate.resize(block.len(), 0.);
            }

            for (s, &v) in self.intermediate.iter_mut().zip(block.iter()) {
                *s += v / total_channel;
            }
        }

        for ConsumerWithMode { consumer, mode } in self.consumers.iter() {
            let Ok(mut consumer) = consumer.try_borrow_mut() else {
                continue;
            };

            match mode {
                ChannelMode::Averaged if channels.is_last_channel() => {
                    consumer.consume(self.intermediate.as_slice(), channels, time);
                }
                ChannelMode::Channel(c) if channels.current == *c => {
                    consumer.consume(block, channels, time);
                }
                _ => {}
            }
        }
    }
}
