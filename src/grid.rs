//! A CSS Grid widget based on Taffy

use iced_native::event::{self, Event};
use iced_native::layout::Limits;
use iced_native::renderer::Renderer;
use iced_native::widget::{Operation, Tree};
use iced_native::{layout, mouse, overlay, renderer};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Widget};

use ::taffy::LayoutAlgorithm;
mod taffy {
    pub use ::taffy::layout::{Layout, RunMode, SizeAndBaselines, SizingMode};
    pub use ::taffy::*;
    pub use taffy::cache::Cache;
    pub use taffy::geometry::*;
    pub use taffy::prelude::*;

    pub const NULL_LAYOUT: Layout = Layout {
        order: 0,
        size: Size::ZERO,
        location: Point::ZERO,
    };
}

fn f32_to_opt(input: f32) -> Option<f32> {
    if input.is_nan() || input.is_infinite() {
        None
    } else {
        Some(input)
    }
}

struct GridLayoutTree<'node, 'a, 'b, Msg, R: Renderer> {
    grid: &'node mut Grid<'a, Msg, R>,
    renderer: &'b R,
    layout: taffy::Layout,
}

const CURRENT_NODE_ID : taffy::NodeId = taffy::NodeId::new(u64::MAX);

/// Iterator that wraps a range of u64, lazily converting them to NodeId's
pub struct GridChildIter(std::ops::Range<usize>);
impl Iterator for GridChildIter {
    type Item = taffy::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|idx| idx.into())
    }
}

impl<'node, 'a, 'b, Msg, R: Renderer> taffy::LayoutTree for GridLayoutTree<'node, 'a, 'b, Msg, R> {
    type ChildIter<'iter> = GridChildIter where Self: 'iter;

    fn style(&self, node: taffy::NodeId) -> &taffy::Style {
        if node == CURRENT_NODE_ID {
            &self.grid.style
        } else {
            let child_index : usize = node.into();
            &self.grid.children[child_index].style
        }
    }

    fn layout_mut(&mut self, node: taffy::NodeId) -> &mut taffy::Layout {
        if node == CURRENT_NODE_ID {
            panic!();
        } else {
            let child_index : usize = node.into();
            &mut self.grid.children[child_index].taffy_layout
        }
    }

    fn children(&self, node: taffy::NodeId) -> Self::ChildIter<'_> {
        GridChildIter(0..(self.child_count(node)))
    }

    fn child_count(&self, node: taffy::NodeId) -> usize {
        self.grid.children.len()
    }

    fn child(&self, node: taffy::NodeId, index: usize) -> taffy::NodeId {
        index.into()
    }

    fn measure_child_size(
        &mut self,
        child_node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        _parent_size: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        _sizing_mode: taffy::SizingMode,
    ) -> taffy::Size<f32> {
        let mut limits = Limits::NONE;

        // Set constraints based on available_space
        if let taffy::AvailableSpace::Definite(height) = available_space.height {
            limits = limits.max_height(height.round());
        }
        if let taffy::AvailableSpace::Definite(width) = available_space.width {
            limits = limits.max_width(width.round());
        }

        // Set constraints based on known dimensions
        if let Some(height) = known_dimensions.height {
            limits = limits.height(Length::Fixed(height.round()))
        }
        if let Some(width) = known_dimensions.width {
            limits = limits.width(Length::Fixed(width.round()))
        }

        let child_index : usize = child_node_id.into();
        let child = &mut self.grid.children[child_index];
        let cached_size = child.cache.get(
            known_dimensions,
            available_space,
            taffy::RunMode::ComputeSize,
        );

        let size = cached_size
            .map(|size_and_baselines| size_and_baselines.size)
            .unwrap_or_else(|| {
                // Compute child layout
                let iced_size = child
                    .element
                    .as_widget_mut()
                    .measure(&self.renderer, &limits);
                let taffy_size = taffy::Size {
                    width: iced_size.width,
                    height: iced_size.height,
                };
                child.cache.store(
                    known_dimensions,
                    available_space,
                    taffy::RunMode::ComputeSize,
                    taffy_size.into(),
                );
                taffy_size
            });

        // Return size
        size
    }

    fn perform_child_layout(
        &mut self,
        child_node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        _parent_size: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        _sizing_mode: taffy::SizingMode,
    ) -> taffy::SizeAndBaselines {
        let mut limits = Limits::NONE;

        // Set constraints based on available_space
        if let taffy::AvailableSpace::Definite(height) = available_space.height {
            limits = limits.max_height(height.round());
        }
        if let taffy::AvailableSpace::Definite(width) = available_space.width {
            limits = limits.max_width(width.round());
        }

        // Set constraints based on known dimensions
        if let Some(height) = known_dimensions.height {
            limits = limits.height(Length::Fixed(height.round()))
        }
        if let Some(width) = known_dimensions.width {
            limits = limits.width(Length::Fixed(width.round()))
        }

        let child_index : usize = child_node_id.into();
        let child = &mut self.grid.children[child_index];
        let cached_layout = child.cache.get(
            known_dimensions,
            available_space,
            taffy::RunMode::PeformLayout,
        );

        let layout = cached_layout.unwrap_or_else(|| {
            // Compute child layout
            let iced_layout = child
                .element
                .as_widget_mut()
                .layout(&self.renderer, &limits);
            let bounds = iced_layout.bounds();
            let taffy_layout = taffy::SizeAndBaselines {
                size: taffy::Size {
                    width: bounds.width,
                    height: bounds.height,
                },
                first_baselines: taffy::Point::NONE,
            };
            child.cache.store(
                known_dimensions,
                available_space,
                taffy::RunMode::PeformLayout,
                taffy_layout,
            );
            child.iced_child_layouts = iced_layout.into_children();
            taffy_layout
        });

        // Return size
        layout
    }

    // fn perform_child_hidden_layout(&mut self, child_node_id: taffy::NodeId, order: u32) {
    //     self.grid.children[child_node_id].taffy_layout = taffy::Layout::with_order(order);
    // }
}

struct GridChild<'a, Msg, R: Renderer> {
    element: Element<'a, Msg, R>,
    style: taffy::Style,
    cache: taffy::Cache,
    taffy_layout: taffy::Layout,
    iced_child_layouts: Vec<iced_native::layout::Node>,
}

impl<'a, Msg, R: Renderer> GridChild<'a, Msg, R> {
    fn new(element: Element<'a, Msg, R>, style: taffy::Style) -> Self {
        Self {
            element,
            style,
            cache: taffy::Cache::new(),
            taffy_layout: taffy::NULL_LAYOUT,
            iced_child_layouts: vec![],
        }
    }
}

pub struct Grid<'a, Msg, R: Renderer> {
    width: Length,
    height: Length,
    style: taffy::Style,
    children: Vec<GridChild<'a, Msg, R>>,
}

impl<'a, Msg, R: Renderer> Grid<'a, Msg, R> {
    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            style: taffy::Style::DEFAULT,
            children: vec![],
        }
    }

    pub fn with_columns(mut self, columns: Vec<taffy::TrackSizingFunction>) -> Self {
        self.style.grid_template_columns = columns;
        self
    }

    pub fn with_rows(mut self, rows: Vec<taffy::TrackSizingFunction>) -> Self {
        self.style.grid_template_rows = rows;
        self
    }

    pub fn with_column_gap(mut self, gap: taffy::LengthPercentage) -> Self {
        self.style.gap.width = gap;
        self
    }

    pub fn with_row_gap(mut self, gap: taffy::LengthPercentage) -> Self {
        self.style.gap.height = gap;
        self
    }

    pub fn min_height(mut self, size: taffy::Dimension) -> Self {
        self.style.min_size.height = size;
        self
    }

    pub fn min_width(mut self, size: taffy::Dimension) -> Self {
        self.style.min_size.width = size;
        self
    }

    pub fn max_height(mut self, size: taffy::Dimension) -> Self {
        self.style.max_size.height = size;
        self
    }

    pub fn max_width(mut self, size: taffy::Dimension) -> Self {
        self.style.max_size.width = size;
        self
    }

    pub fn style(mut self, mut callback: impl FnMut(&mut taffy::Style)) -> Self {
        callback(&mut self.style);
        self
    }

    pub fn with_styled_child(
        mut self,
        element: impl Into<Element<'a, Msg, R>>,
        mut callback: impl FnMut(&mut taffy::Style),
    ) -> Self {
        let mut style = taffy::Style::DEFAULT;
        callback(&mut style);
        self.children.push(GridChild::new(element.into(), style));
        self
    }

    pub fn with_child(mut self, element: impl Into<Element<'a, Msg, R>>) -> Self {
        self.children
            .push(GridChild::new(element.into(), taffy::Style::DEFAULT));
        self
    }

    pub fn add_child(&mut self, element: impl Into<Element<'a, Msg, R>>) {
        self.children
            .push(GridChild::new(element.into(), taffy::Style::DEFAULT));
    }

    /// Sets the width of the [`Grid`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Grid`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

pub fn grid<'a, Msg, R: Renderer>() -> Grid<'a, Msg, R> {
    Grid::new()
}

impl<'a, Msg, R: Renderer> Widget<Msg, R> for Grid<'a, Msg, R> {
    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|child| Tree::new(&child.element))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(|child| child.element.as_widget())
                .collect::<Vec<_>>(),
        );
    }

    fn width(&self) -> Length {
        // match self.style.size.width {
        //     Dimension::Auto => Length::Shrink,
        //     Dimension::Points(units) => Length::Fixed(units.round() as u16),
        //     Dimension::Percent(_) => Length::Fill,
        // }
        self.width
    }

    fn height(&self) -> Length {
        // match self.style.size.height {
        //     Dimension::Auto => Length::Shrink,
        //     Dimension::Points(units) => Length::Fixed(units.round() as u16),
        //     Dimension::Percent(_) => Length::Fill,
        // }
        self.height
    }

    fn measure(&mut self, renderer: &R, limits: &layout::Limits) -> iced_native::Size {
        let mut node_ref = GridLayoutTree {
            grid: self,
            renderer,
            layout: taffy::NULL_LAYOUT,
        };

        let mut known_dimensions = taffy::Size::NONE;
        if limits.min().height < f32::INFINITY && limits.min().height == limits.max().height {
            known_dimensions.height = Some(limits.min().height);
        }
        if limits.min().width < f32::INFINITY && limits.min().width == limits.max().width {
            known_dimensions.width = Some(limits.min().width);
        }
        let parent_size = taffy::Size {
            width: f32_to_opt(limits.max().width),
            height: f32_to_opt(limits.max().height),
        };
        let available_space = parent_size.map(|s| s.into());
        let sizing_mode = taffy::SizingMode::InherentSize;

        let size = taffy::CssGridAlgorithm::measure_size(
            &mut node_ref,
            CURRENT_NODE_ID,
            known_dimensions,
            parent_size,
            available_space,
            sizing_mode,
        );

        Size {
            width: size.width,
            height: size.height,
        }
    }

    fn layout(&mut self, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let mut node_ref = GridLayoutTree {
            grid: self,
            renderer,
            layout: taffy::NULL_LAYOUT,
        };

        let mut known_dimensions = taffy::Size::NONE;
        if limits.min().height < f32::INFINITY && limits.min().height == limits.max().height {
            known_dimensions.height = Some(limits.min().height);
        }
        if limits.min().width < f32::INFINITY && limits.min().width == limits.max().width {
            known_dimensions.width = Some(limits.min().width);
        }
        let parent_size = taffy::Size {
            width: f32_to_opt(limits.max().width),
            height: f32_to_opt(limits.max().height),
        };
        let available_space = parent_size.map(|s| s.into());
        let sizing_mode = taffy::SizingMode::InherentSize;

        let size_and_baselines = taffy::CssGridAlgorithm::perform_layout(
            &mut node_ref,
            CURRENT_NODE_ID,
            known_dimensions,
            parent_size,
            available_space,
            sizing_mode,
        );

        let child_nodes = self
            .children
            .iter_mut()
            .map(|child| {
                // child.taffy_layout.round();
                let mut iced_layout = layout::Node::with_children(
                    Size {
                        width: child.taffy_layout.size.width,
                        height: child.taffy_layout.size.height,
                    },
                    child.iced_child_layouts.clone(),
                );
                iced_layout.move_to(Point {
                    x: child.taffy_layout.location.x,
                    y: child.taffy_layout.location.y,
                });
                iced_layout
            })
            .collect::<Vec<layout::Node>>();

        return layout::Node::with_children(
            Size {
                width: size_and_baselines.size.width,
                height: size_and_baselines.size.height,
            },
            child_nodes,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation<Msg>,
    ) {
        operation.container(None, &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .element
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Msg>,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.element.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.element.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &mut self,
        tree: &Tree,
        renderer: &mut R,
        theme: &R::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.element.as_widget_mut().draw(
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &R,
    ) -> Option<overlay::Element<'b, Msg, R>> {
        // This calls the first overlay. We probably want all overlays?
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .find_map(|((child, state), layout)| {
                child
                    .element
                    .as_widget_mut()
                    .overlay(state, layout, renderer)
            })
    }
}

impl<'a, Msg: 'a, R: Renderer + 'a> From<Grid<'a, Msg, R>> for Element<'a, Msg, R> {
    fn from(grid: Grid<'a, Msg, R>) -> Self {
        Self::new(grid)
    }
}
