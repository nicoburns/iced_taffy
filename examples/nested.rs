use iced::widget::{button, text};
use iced::{Element, Length, Sandbox, Settings};
use iced_taffy::grid;
use taffy::prelude::*;

mod common {
    pub mod colors;
    pub mod layout_timer;
    pub mod rect;
}
use common::colors::*;
use common::layout_timer::LayoutTimer;
use common::rect::rect;

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    click_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example { click_count: 0 }
    }

    fn title(&self) -> String {
        String::from("Custom widget - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.click_count += 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        const REALLY_LONG_PARAGRAPH : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let content = grid()
            .with_columns(vec![flex(1.), flex(2.), flex(1.)])
            .with_rows(vec![auto(), auto(), flex(1.)])
            .style(|style| {
                style.size.width = percent(1.);
                style.size.height = percent(1.);
                style.gap = points(20.);
            })
            .with_child(rect(20.0, BLACK))
            .with_child({
                grid()
                    .with_styled_child(
                        text(format!("Button clicked {} times", self.click_count)).size(32),
                        |style| {
                            style.align_self = Some(AlignSelf::Center);
                            style.justify_self = Some(AlignSelf::Center);
                        },
                    )
                    .with_styled_child(
                        text(REALLY_LONG_PARAGRAPH).width(Length::Fixed(100.)),
                        |style| {
                            style.margin = points(40.);
                        },
                    )
            })
            .with_child(rect(20.0, RED))
            .with_child(rect(20.0, COLOR1))
            .with_child(rect(20.0, COLOR2))
            .with_child(rect(20.0, COLOR3))
            .with_child(rect(20.0, COLOR4))
            .with_child({
                grid()
                    .with_columns(vec![flex(1.), flex(2.), flex(1.)])
                    .with_rows(vec![flex(1.), percent(0.5), flex(1.)])
                    .with_child(rect(20.0, COLOR7))
                    .with_child(rect(20.0, COLOR8))
                    .with_child(rect(20.0, COLOR9))
                    .with_child(rect(20.0, COLOR10))
                    .with_styled_child(button("Increment").on_press(Message::Increment), |style| {
                        style.align_self = Some(AlignSelf::Center);
                        style.justify_self = Some(AlignSelf::Center);
                    })
                    .with_child(rect(20.0, COLOR11))
                    .with_child(rect(20.0, COLOR13))
                    .with_child(rect(20.0, COLOR14))
                    .with_child(rect(20.0, COLOR15))
                    .with_styled_child(rect(20.0, COLOR16), |style| {
                        style.position = Position::Absolute;
                        style.grid_row = line(1);
                        style.grid_column = line(1);
                        style.inset.left = points(10.);
                        style.inset.top = points(10.);
                    })
            })
            .with_child(rect(20.0, COLOR6));

        LayoutTimer::new(content).into()
    }
}
