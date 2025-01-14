use std::time::Duration;

use basedrop::Shared;
use cpal::Stream;
use iced::canvas::{Cursor, Frame, Geometry, Program, Stroke};
use iced::{
    Application, Canvas, Clipboard, Column, Command, Element, Length, Point, Rectangle, Settings,
    Subscription,
};

use atomic_queue::Queue;
use audio_garbage_collector::GarbageCollector;
use audio_processor_standalone::{audio_processor_start, StandaloneHandles};
use circular_data_structures::CircularVec;
use example_iced_audio_viz::buffer_analyser::BufferAnalyserProcessor;

fn main() -> iced::Result {
    log::info!("Initializing app");
    AudioVisualization::run(Settings::default())
}

struct AudioProcessingHandles {
    #[allow(dead_code)]
    garbage_collector: GarbageCollector,
    #[allow(dead_code)]
    standalone_handles: StandaloneHandles,
}

struct AudioVisualization {
    #[allow(dead_code)]
    audio_processing_handles: AudioProcessingHandles,
    queue_handle: Shared<Queue<f32>>,
    buffer: CircularVec<f32>,
    position: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl Application for AudioVisualization {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let garbage_collector = GarbageCollector::default();
        let processor = BufferAnalyserProcessor::new(garbage_collector.handle());
        let queue_handle = processor.queue();
        let standalone_handles = audio_processor_start(processor);
        let buffer = CircularVec::with_size(5 * 4410, 0.0);

        (
            AudioVisualization {
                audio_processing_handles: AudioProcessingHandles {
                    garbage_collector,
                    standalone_handles,
                },
                queue_handle,
                buffer,
                position: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ICED Audio Viz")
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        while let Some(sample) = self.queue_handle.pop() {
            self.buffer[self.position] = sample;
            self.position += 1;
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        Column::with_children(vec![canvas]).into()
    }
}

impl Program<Message> for AudioVisualization {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());
        let mut path = iced::canvas::path::Builder::new();

        let data = self.buffer.inner();
        let mut prev = data[0];
        let mut index = 0;

        while index < data.len() {
            let item = data[index];
            let f_index = index as f32;
            let x_coord = (f_index / data.len() as f32) * frame.width();
            let x2_coord = ((f_index + 1.0) / data.len() as f32) * frame.width();
            let y_coord = (prev as f32) * frame.height() / 2.0 + frame.height() / 2.0;
            let y2_coord = (item as f32) * frame.height() / 2.0 + frame.height() / 2.0;

            path.move_to(Point::new(x_coord, y_coord));
            path.line_to(Point::new(x2_coord, y2_coord));

            prev = item;
            index += 10;
        }

        frame.stroke(&path.build(), Stroke::default());

        vec![frame.into_geometry()]
    }
}
