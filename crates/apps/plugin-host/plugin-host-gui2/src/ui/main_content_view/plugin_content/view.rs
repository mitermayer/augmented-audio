use audio_processor_iced_design_system::spacing::Spacing;
use iced::{Align, Button, Column, Command, Container, Element, Length, Row, Text};

pub struct View {
    input_file_path_button_state: iced::button::State,
    audio_plugin_path_button_state: iced::button::State,
    plugin_open_button: iced::button::State,
    plugin_float_button: iced::button::State,
    reload_plugin_button: iced::button::State,
    plugin_is_open: bool,
    input_file: Option<String>,
    audio_plugin_path: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    OpenInputFilePathPicker,
    OpenAudioPluginFilePathPicker,
    ReloadPlugin,
    OpenPluginWindow,
    ClosePluginWindow,
    FloatPluginWindow,
    SetInputFile(String),
    SetAudioPlugin(String),
}

impl View {
    pub fn new(input_file: Option<String>, audio_plugin_path: Option<String>) -> Self {
        View {
            input_file_path_button_state: iced::button::State::new(),
            audio_plugin_path_button_state: iced::button::State::new(),
            plugin_open_button: iced::button::State::new(),
            plugin_float_button: iced::button::State::new(),
            reload_plugin_button: iced::button::State::new(),
            plugin_is_open: false,
            input_file,
            audio_plugin_path,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenInputFilePathPicker => {
                let result = tinyfiledialogs::open_file_dialog("Input file", "", None);
                log::info!("Got response {:?}", result);
                if let Some(path) = result {
                    return Command::perform(async move { path }, |path| {
                        Message::SetInputFile(path)
                    });
                }
            }
            Message::OpenAudioPluginFilePathPicker => {
                let result = tinyfiledialogs::open_file_dialog("Audio plugin", "", None);
                log::info!("Got response {:?}", result);
                if let Some(path) = result {
                    return Command::perform(async move { path }, |path| {
                        Message::SetAudioPlugin(path)
                    });
                }
            }
            Message::SetInputFile(path) => {
                self.input_file = Some(path);
            }
            Message::SetAudioPlugin(path) => {
                self.audio_plugin_path = Some(path);
            }
            Message::OpenPluginWindow => self.plugin_is_open = true,
            Message::ClosePluginWindow => self.plugin_is_open = false,
            _ => {}
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut children = vec![
            Self::file_picker_with_label(
                "Input file",
                &mut self.input_file_path_button_state,
                &self.input_file,
                "Select input file",
                Message::OpenInputFilePathPicker,
            ),
            Self::file_picker_with_label(
                "Audio plugin",
                &mut self.audio_plugin_path_button_state,
                &self.audio_plugin_path,
                "Select audio plugin",
                Message::OpenAudioPluginFilePathPicker,
            ),
        ];

        let mut buttons_row = vec![];
        #[cfg(target_os = "macos")]
        {
            if !self.plugin_is_open {
                buttons_row.push(
                    Button::new(&mut self.plugin_open_button, Text::new("Open editor"))
                        .style(audio_processor_iced_design_system::style::Button)
                        .on_press(Message::OpenPluginWindow)
                        .into(),
                );
            } else {
                buttons_row.push(
                    Button::new(&mut self.plugin_open_button, Text::new("Close editor"))
                        .style(audio_processor_iced_design_system::style::Button)
                        .on_press(Message::ClosePluginWindow)
                        .into(),
                );
            }
            buttons_row.push(
                Button::new(&mut self.plugin_float_button, Text::new("Float editor"))
                    .style(audio_processor_iced_design_system::style::Button)
                    .on_press(Message::FloatPluginWindow)
                    .into(),
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            children.push(
                Container::new(Text::new("Opening the editor is not supported in this OS"))
                    .center_x()
                    .width(Length::Fill)
                    .into(),
            );
        }

        buttons_row.push(
            Button::new(&mut self.reload_plugin_button, Text::new("Reload plugin"))
                .on_press(Message::ReloadPlugin)
                .style(audio_processor_iced_design_system::style::Button)
                .into(),
        );
        children.push(
            Container::new(Row::with_children(buttons_row).spacing(Spacing::base_spacing()))
                .center_x()
                .width(Length::Fill)
                .into(),
        );

        Column::with_children(children)
            .spacing(Spacing::base_spacing())
            .padding(Spacing::base_spacing())
            .into()
    }

    fn file_picker_with_label<'a>(
        label: impl Into<String>,
        state: &'a mut iced::button::State,
        option: &'a Option<String>,
        button_text: impl Into<String>,
        message: Message,
    ) -> Element<'a, Message> {
        Row::with_children(vec![
            Container::new(Text::new(label))
                .width(Length::FillPortion(2))
                .align_x(Align::End)
                .center_y()
                .padding([0, Spacing::base_spacing()])
                .into(),
            Container::new(
                Row::with_children(vec![Button::new(
                    state,
                    Text::new(match option {
                        Some(file) => file.into(),
                        None => button_text.into(),
                    }),
                )
                .on_press(message)
                .style(audio_processor_iced_design_system::style::Button)
                .into()])
                .align_items(Align::Center)
                .spacing(Spacing::base_spacing()),
            )
            .center_y()
            .width(Length::FillPortion(8))
            .into(),
        ])
        .align_items(Align::Center)
        .into()
    }
}

#[cfg(feature = "story")]
pub mod story {
    use audio_processor_iced_storybook::StoryView;

    use super::*;

    pub fn default() -> Story {
        Story::default()
    }

    pub struct Story {
        view: View,
    }

    impl Default for Story {
        fn default() -> Self {
            Self {
                view: View::new(None, None),
            }
        }
    }

    impl StoryView<Message> for Story {
        fn update(&mut self, message: Message) -> Command<Message> {
            self.view.update(message)
        }

        fn view(&mut self) -> Element<Message> {
            self.view.view()
        }
    }
}
