//! Decorate content and apply alignment.
use iced_native::event::{self, Event};
use iced_native::renderer::Renderer;
use iced_native::widget::{Operation, Tree};
use iced_native::{layout, mouse, overlay, renderer};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Widget};

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct LayoutTimer<'a, Message, R: Renderer> {
    content: Element<'a, Message, R>,
}

impl<'a, Message, R> LayoutTimer<'a, Message, R>
where
    Message: Clone,
    R: Renderer,
{
    /// Creates an empty [`LayoutTimer`].
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, R>>,
    {
        LayoutTimer {
            content: content.into(),
        }
    }
}

impl<'a, Message: Clone, R: Renderer> Widget<Message, R> for LayoutTimer<'a, Message, R> {
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree)
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(&mut self, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let start = std::time::Instant::now();
        let layout = self.content.as_widget_mut().layout(renderer, limits);
        println!("Layout took: {:.3}ms", start.elapsed().as_micros() as f64 / 1000.0);
        layout
    }

    fn measure(&mut self, renderer: &R, limits: &layout::Limits) -> Size {
        self.content.as_widget_mut().measure(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                tree,
                layout,
                renderer,
                operation,
            );
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
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            tree,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &mut self,
        tree: &Tree,
        renderer: &mut R,
        theme: &R::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().draw(
            tree,
            renderer,
            theme,
            renderer_style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &R,
    ) -> Option<overlay::Element<'b, Message, R>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout,
            renderer,
        )
    }
}

impl<'a, Message: Clone, R: 'a + Renderer> From<LayoutTimer<'a, Message, R>>
    for Element<'a, Message, R>
where
    Message: 'a,
{
    fn from(column: LayoutTimer<'a, Message, R>) -> Element<'a, Message, R> {
        Element::new(column)
    }
}
