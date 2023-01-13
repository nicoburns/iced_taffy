//! This example showcases a simple native custom widget that draws a rect.

// For now, to implement a custom native widget you will need to add
// `iced_native` and `iced_wgpu` to your dependencies.
//
// Then, you simply need to define your widget type and implement the
// `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
//
// Of course, you can choose to make the implementation renderer-agnostic,
// if you wish to, by creating your own `Renderer` trait, which could be
// implemented by `iced_wgpu` and other renderers.
use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::widget::{self, Widget};
use iced_native::{Color, Element, Length, Point, Rectangle, Size};

pub struct Rect {
    size: f32,
    color: Color,
}

impl Rect {
    pub fn new(size: f32, color: Color) -> Self {
        Self { size, color }
    }
}

pub fn rect(size: f32, color: Color) -> Rect {
    Rect::new(size, color)
}

impl<Message, Renderer> Widget<Message, Renderer> for Rect
where
    Renderer: renderer::Renderer,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.size, self.size))
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_radius: 0.0.into(),//self.radius.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            self.color,
        );
    }
}

impl<'a, Message, Renderer> From<Rect> for Element<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(rect: Rect) -> Self {
        Self::new(rect)
    }
}
