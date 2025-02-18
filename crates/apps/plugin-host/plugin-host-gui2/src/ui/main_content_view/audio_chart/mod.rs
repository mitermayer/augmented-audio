use std::cell::RefCell;
use std::time::SystemTime;

use iced::canvas::{Cursor, Fill, Frame, Geometry, Program};
use iced::{canvas, Canvas, Element, Length, Point, Rectangle, Size};

use audio_garbage_collector::Shared;
use audio_processor_iced_design_system::colors::Colors;
use audio_processor_traits::audio_buffer::{OwnedAudioBuffer, VecAudioBuffer};
use audio_processor_traits::AudioBuffer;
use plugin_host_lib::processors::running_rms_processor::RunningRMSProcessorHandle;

pub type Message = ();

pub struct AudioChart {
    frame: RefCell<Frame>,
    handle: Shared<RunningRMSProcessorHandle>,
    rms_buffer: VecAudioBuffer<f32>,
    last_update: usize,
    cursor: usize,
}

impl AudioChart {
    pub fn new(handle: Shared<RunningRMSProcessorHandle>) -> Self {
        let mut rms_buffer = VecAudioBuffer::new();
        rms_buffer.resize(1, 500, 0.0);
        Self {
            frame: RefCell::new(Frame::new(Size::new(100., 100.))),
            handle,
            rms_buffer,
            last_update: 0,
            cursor: 0,
        }
    }

    pub fn update(&mut self) {
        let now: usize = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        if now - self.last_update >= 16 {
            self.last_update = now;
            let left_volume = self.handle.calculate_rms(0);
            let right_volume = self.handle.calculate_rms(1);
            self.rms_buffer
                .set(0, self.cursor, (left_volume + right_volume) / 2.0);
            self.cursor += 1;
            if self.cursor >= self.rms_buffer.num_samples() {
                self.cursor = 0;
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        AudioChartView::new(&mut self.frame, &self.rms_buffer, self.cursor).view()
    }
}

// TODO - Use a view model here rather than a raw buffer
pub struct AudioChartView<'a, Buffer: AudioBuffer> {
    frame: &'a mut RefCell<Frame>,
    audio_buffer: &'a Buffer,
    position: usize,
}

impl<'a, Buffer: AudioBuffer<SampleType = f32>> AudioChartView<'a, Buffer> {
    pub fn new(frame: &'a mut RefCell<Frame>, audio_buffer: &'a Buffer, position: usize) -> Self {
        Self {
            frame,
            audio_buffer,
            position,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        Canvas::new(self)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}

impl<'a, Buffer: AudioBuffer<SampleType = f32>> Program<Message> for AudioChartView<'a, Buffer> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = self.frame.borrow_mut();
        frame.resize(bounds.size());
        let mut path = canvas::path::Builder::new();

        let num_samples = self.audio_buffer.num_samples();

        path.line_to(Point::new(frame.width(), frame.height()));
        let position = self.position;
        for sample_index in 0..num_samples {
            let read_index = ((num_samples - sample_index) + position) % num_samples;
            let sample = *self.audio_buffer.get(0, read_index); // TODO - This is "mono" just ignoring channels
            if sample.is_nan() {
                log::error!("NaN sample in volume buffer");
                return vec![];
            }

            let item = sample * 10.0;
            let f_index = sample_index as f32;
            let x2_coord = frame.width() - ((f_index + 1.0) / num_samples as f32) * frame.width();
            let y2_coord = frame.height() - (item as f32) * frame.height();
            path.line_to(Point::new(x2_coord, y2_coord));
        }
        path.line_to(Point::new(0.0, frame.height()));
        path.line_to(Point::new(frame.width(), frame.height()));

        frame.fill(
            &path.build(),
            Fill::from(Colors::border_color().darken(-0.3)),
        );

        vec![frame.geometry()]
    }
}
