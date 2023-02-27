use iced::{
    widget::{column, row},
    Alignment, Element, Length, Sandbox, Settings,
};
use iced_native::Renderer;
use iced_taffy::{grid, Grid};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::iter;
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

/// A helper function to recursively construct a deep tree
pub fn build_deep_row_column_tree<'a, R: Rng, M: 'a, Rend: Renderer + 'a>(
    rng: &mut R,
    node_count: &mut usize,
    levels: usize,
    nodes_per_level: usize,
) -> Vec<Element<'a, M, Rend>> {
    *node_count += nodes_per_level;

    if levels == 1 {
        // Build leaf nodes
        return (0..nodes_per_level)
            .map(|_| rect(20.0, random_color(rng)).into())
            .collect();
    } else {
        // Add another layer to the tree
        // Each child gets an equal amount of the remaining nodes
        return (0..nodes_per_level)
            .map(|_| {
                let children =
                    build_deep_row_column_tree(rng, node_count, levels - 1, nodes_per_level);
                if levels % 2 == 0 {
                    row(children)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .align_items(Alignment::Fill)
                        .into()
                } else {
                    column(children)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .align_items(Alignment::Fill)
                        .into()
                }
            })
            .collect();
    }
}

/// A tree with a higher depth for a more realistic scenario
fn build_deep_row_column_hierarchy<'a, M: 'a, Rend: Renderer + 'a>(
    levels: usize,
    nodes_per_level: usize,
) -> Element<'a, M, Rend> {
    let mut node_count = 0;
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let children = build_deep_row_column_tree(&mut rng, &mut node_count, levels, nodes_per_level);
    println!("node_count: {}", node_count);

    if (levels + 1) % 2 == 0 {
        row(children)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_items(Alignment::Fill)
            .into()
    } else {
        column(children)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_items(Alignment::Fill)
            .into()
    }
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
        LayoutTimer::new(build_deep_row_column_hierarchy(12, 2)).into()
    }
}
