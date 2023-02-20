//! A CSS Grid widget based on Taffy

use iced_native::event::{self, Event};
use iced_native::layout::Limits;
use iced_native::renderer::Renderer;
use iced_native::widget::{Operation, Tree};
use iced_native::{layout, mouse, overlay, renderer};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Widget};

use ::taffy::LayoutAlgorithm;
mod taffy {
    pub use ::taffy::layout::{Cache as CacheEntry, Layout, RunMode, SizeAndBaselines, SizingMode};
    pub use ::taffy::*;
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

// struct GridBuilder<'builder, 'a, Msg, R: Renderer> {
//     grid: &'builder mut Grid<'a, Msg, R>,
// }

// impl<'builder, 'a, Msg, R: Renderer> GridBuilder<'builder, 'a, Msg, R> {

//     pub fn child(&mut self, child: Element<'a, Msg, R>) {
//         self.grid.children.push(child)
//     }
// }

struct GridLayoutTree<'node, 'a, 'b, Msg, R: Renderer> {
    grid: &'node mut Grid<'a, Msg, R>,
    renderer: &'b R,
    layout: taffy::Layout,
    cache: [Option<taffy::CacheEntry>; 5],
    child_layouts: Vec<taffy::Layout>,
    // child_caches: Vec<[Option<taffy::CacheEntry>; 5]>,
}

impl<'node, 'a, 'b, Msg, R: Renderer> taffy::LayoutTree for GridLayoutTree<'node, 'a, 'b, Msg, R> {
    type ChildId = usize;
    type ChildIter<'iter> = std::ops::Range<usize> where Self: 'iter;

    fn style(&self) -> &taffy::Style {
        &self.grid.style
    }

    fn layout_mut(&mut self) -> &mut taffy::Layout {
        &mut self.layout
    }

    fn cache_mut(&mut self, index: usize) -> &mut Option<taffy::CacheEntry> {
        &mut self.cache[index]
    }

    fn children(&self) -> Self::ChildIter<'_> {
        0..(self.child_count())
    }

    fn child_count(&self) -> usize {
        self.grid.children.len()
    }

    fn child(&self, index: usize) -> Self::ChildId {
        index
    }

    fn child_style(&self, child_node_id: Self::ChildId) -> &taffy::Style {
        &self.grid.child_styles[child_node_id]
    }

    fn child_layout_mut(&mut self, child_node_id: Self::ChildId) -> &mut taffy::Layout {
        &mut self.child_layouts[child_node_id]
    }

    fn measure_child_size(
        &mut self,
        child_node_id: Self::ChildId,
        known_dimensions: taffy::Size<Option<f32>>,
        parent_size: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        sizing_mode: taffy::SizingMode,
    ) -> taffy::Size<f32> {
        self.perform_child_layout(
            child_node_id,
            known_dimensions,
            parent_size,
            available_space,
            sizing_mode,
        )
        .size
    }

    fn perform_child_layout(
        &mut self,
        child_node_id: Self::ChildId,
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

        // Compute child layout
        let iced_layout = self.grid.children[child_node_id]
            .as_widget_mut()
            .layout(&self.renderer, &limits);
        let bounds = iced_layout.bounds();

        // Return size
        taffy::SizeAndBaselines {
            size: taffy::Size {
                width: bounds.width,
                height: bounds.height,
            },
            first_baselines: taffy::Point::NONE,
        }
    }

    fn perform_child_hidden_layout(&mut self, child_node_id: Self::ChildId, order: u32) {
        self.child_layouts[child_node_id] = taffy::Layout::with_order(order);
    }
}

pub struct Grid<'a, Msg, R: Renderer> {
    width: Length,
    height: Length,
    style: taffy::Style,
    children: Vec<Element<'a, Msg, R>>,
    child_styles: Vec<taffy::Style>,
}

impl<'a, Msg, R: Renderer> Grid<'a, Msg, R> {
    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            style: taffy::Style::DEFAULT,
            children: vec![],
            child_styles: vec![],
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
        self.child_styles.push(style);
        self.children.push(element.into());

        self
    }

    pub fn with_child(mut self, element: impl Into<Element<'a, Msg, R>>) -> Self {
        self.child_styles.push(taffy::Style::DEFAULT);
        self.children.push(element.into());

        self
    }

    pub fn add_child(&mut self, element: impl Into<Element<'a, Msg, R>>) {
        self.child_styles.push(taffy::Style::DEFAULT);
        self.child_caches.push(taffy::Cache::new());
        self.children.push(element.into());
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
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(|child| child.as_widget())
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

    fn layout(&mut self, renderer: &R, limits: &layout::Limits) -> layout::Node {
        // let limits = limits
        //     .max_width(self.max_width)
        //     .width(self.width)
        //     .height(self.height);

        let child_count = self.children.len();

        let mut node_ref = GridLayoutTree {
            grid: self,
            renderer,
            cache: [None; 5],
            layout: taffy::NULL_LAYOUT,
            child_layouts: vec![taffy::NULL_LAYOUT; child_count],
            // child_caches: vec![[None; 5]; child_count],
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
            known_dimensions,
            parent_size,
            available_space,
            sizing_mode,
        );

        let child_layouts = node_ref.child_layouts;
        let child_nodes = self
            .children
            .iter_mut()
            .zip(child_layouts)
            .map(|(child, mut taffy_layout)| {
                taffy_layout.round();

                let limits = Limits::NONE
                    .width(Length::Fixed(taffy_layout.size.width))
                    .height(Length::Fixed(taffy_layout.size.height));

                let child_layouts = child
                    .as_widget_mut()
                    .layout(renderer, &limits)
                    .children()
                    .to_owned();

                let mut iced_layout = layout::Node::with_children(
                    Size {
                        width: taffy_layout.size.width,
                        height: taffy_layout.size.height,
                    },
                    child_layouts,
                );
                iced_layout.move_to(Point {
                    x: taffy_layout.location.x,
                    y: taffy_layout.location.y,
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
                child.as_widget_mut().on_event(
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
                child.as_widget().mouse_interaction(
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
            child.as_widget_mut().draw(
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
                child.as_widget_mut().overlay(state, layout, renderer)
            })
    }
}

impl<'a, Msg: 'a, R: Renderer + 'a> From<Grid<'a, Msg, R>> for Element<'a, Msg, R> {
    fn from(grid: Grid<'a, Msg, R>) -> Self {
        Self::new(grid)
    }
}
