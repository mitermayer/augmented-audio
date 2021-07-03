use basedrop::Owned;

use crate::audio_io::midi::{MidiMessageQueue, MidiMessageWrapper};

/// Pops MIDI events from the the MIDI queue & collects them on a pre-allocated fixed capacity
/// vector.
pub struct MidiMessageHandler {
    buffer: Vec<Owned<MidiMessageWrapper>>,
    capacity: usize,
}

impl MidiMessageHandler {
    pub fn new(capacity: usize) -> Self {
        MidiMessageHandler {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn buffer(&self) -> &Vec<Owned<MidiMessageWrapper>> {
        &self.buffer
    }

    pub fn collect_midi_messages(&mut self, midi_message_queue: &MidiMessageQueue) -> usize {
        let mut midi_message_count = 0;
        for _i in 0..self.capacity {
            if let Some(midi_message) = midi_message_queue.pop() {
                self.buffer.push(midi_message);
                midi_message_count += 1;
            } else {
                return midi_message_count;
            }
        }
        midi_message_count
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}