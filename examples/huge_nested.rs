use iced::{Element, Sandbox, Settings};
use iced_native::Renderer;
use iced_taffy::{grid, Grid};
use taffy::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::iter;

mod common {
    pub mod rect;
    pub mod colors;
}
use common::rect::rect;
use common::colors::*;

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

fn random_grid_track<R: Rng>(rng: &mut R) -> TrackSizingFunction {
    let switch: f32 = rng.gen_range(0.0..=1.0);
    if switch < 0.1 {
        auto()
    } else if switch < 0.2 {
        min_content()
    } else if switch < 0.3 {
        max_content()
    } else if switch < 0.5 {
        fr(1.0)
    } else if switch < 0.6 {
        minmax(points(0.0), fr(1.0))
    } else if switch < 0.8 {
        points(40.0)
    } else {
        percent(0.3)
    }
}

fn random_nxn_grid<'a, R: Rng, M, Rend: Renderer>(rng: &mut R, track_count: usize) -> Grid<'a, M, Rend> {
    grid()
        .with_columns(iter::from_fn(|| Some(random_grid_track(rng))).take(track_count).collect())
        .with_rows(iter::from_fn(|| Some(random_grid_track(rng))).take(track_count).collect())
}

/// A helper function to recursively construct a deep tree
pub fn build_deep_grid_tree<'a, R: Rng, M: 'a, Rend: Renderer + 'a>(
    parent: &mut Grid<'a, M, Rend>,
    rng: &mut R,
    node_count: &mut usize,
    levels: usize,
    track_count: usize,
) {
    // The extra one is for a position:absolute child
    let child_count = track_count * track_count;

    if levels == 1 {
        // Build leaf nodes
        for _ in 0..child_count {
            *node_count += 1;
            parent.add_child(rect(20.0, random_color(rng)))
        }
    } else {
        // Add another layer to the tree
        // Each child gets an equal amount of the remaining nodes
        for _ in 0..child_count {
            let mut grid = random_nxn_grid(rng, track_count);
            *node_count += 1;
            build_deep_grid_tree(&mut grid, rng, node_count, levels - 1, track_count);
            parent.add_child(grid);
        }
    }
}

/// A tree with a higher depth for a more realistic scenario
fn build_taffy_deep_grid_hierarchy<'a, M: 'a, Rend: Renderer + 'a>(levels: usize, track_count: usize) -> Grid<'a, M, Rend> {
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let mut grid = random_nxn_grid(&mut rng, track_count);
    let mut node_count = 0;
    build_deep_grid_tree(&mut grid, &mut rng, &mut node_count, levels, track_count);

    println!("node_count: {}", node_count);

    grid
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
        build_taffy_deep_grid_hierarchy(4, 3).into()
    }
}
