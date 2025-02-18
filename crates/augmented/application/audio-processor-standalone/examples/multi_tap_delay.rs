use std::time::Duration;

use audio_processor_traits::{AudioBuffer, AudioProcessor};
use circular_data_structures::CircularVec;

struct SimpleDelayProcessor {
    current_write_position: usize,
    current_read_positions: Vec<(usize, usize)>,
    delay_buffers: Vec<CircularVec<f32>>,
}

impl SimpleDelayProcessor {
    fn new() -> Self {
        let num_reads = 5;
        let delay_time = 1000.0;
        let tap_increment_right = 0.0 / num_reads as f32;
        let tap_increment_left = 500.0 / num_reads as f32;

        let write_position =
            (Duration::from_millis(delay_time as u64).as_secs_f32() * 44100.0) as usize;
        let read_positions: Vec<(usize, usize)> = (0..num_reads)
            .map(|i| {
                let tap_time_left =
                    tap_increment_left + tap_increment_left * (i as f32 / num_reads as f32);
                let tap_time_right =
                    tap_increment_right + tap_increment_right * (i as f32 / num_reads as f32);
                (
                    (Duration::from_secs_f32(tap_time_left / 1000.0).as_secs_f32() * 44100.0)
                        as usize,
                    (Duration::from_secs_f32(tap_time_right / 1000.0).as_secs_f32() * 44100.0)
                        as usize,
                )
            })
            .collect();

        for pos in &read_positions {
            if pos.0 >= write_position {
                panic!("Delay will cause feedback loop");
            }
            if pos.1 >= write_position {
                panic!("Delay will cause feedback loop");
            }
        }

        Self {
            current_write_position: write_position,
            current_read_positions: read_positions,
            delay_buffers: vec![
                CircularVec::with_size(
                    (Duration::from_secs(10).as_secs_f32() * 44100.0) as usize,
                    0.0,
                ),
                CircularVec::with_size(
                    (Duration::from_secs(10).as_secs_f32() * 44100.0) as usize,
                    0.0,
                ),
            ],
        }
    }
}

impl AudioProcessor for SimpleDelayProcessor {
    type SampleType = f32;

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        // Mono input stage
        for sample_index in 0..data.num_samples() {
            data.set(0, sample_index, *data.get(1, sample_index));
        }

        let num_read_positions = self.delay_buffers.len() as f32;
        // Delay read/write
        for sample_index in 0..data.num_samples() {
            for channel_index in 0..data.num_channels() {
                let input = *data.get(channel_index, sample_index);

                // Read delay output
                let delay_output: f32 = self
                    .current_read_positions
                    .iter()
                    .enumerate()
                    .map(|(index, read_position)| {
                        let read_index = if channel_index == 0 {
                            read_position.0
                        } else {
                            read_position.1
                        };

                        let volume = (1.0 - index as f32) / num_read_positions;

                        volume * self.delay_buffers[channel_index][read_index]
                    })
                    .sum();
                let delay_output = 0.4 * delay_output / num_read_positions;

                // Write input
                let feedback = delay_output * 0.3;
                self.delay_buffers[channel_index][self.current_write_position] = input + feedback;

                // Output stage
                data.set(channel_index, sample_index, input + delay_output);
            }

            for pos in &mut self.current_read_positions {
                *pos = (pos.0 + 1, pos.1 + 1);
            }
            self.current_write_position += 1;
        }
    }
}

fn main() {
    let processor = SimpleDelayProcessor::new();
    audio_processor_standalone::audio_processor_main(processor);
}
