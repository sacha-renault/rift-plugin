use std::cell::RefCell;

use vizia::prelude::{BoundingBox, Canvas, DrawContext};
use vizia::vg::{AlphaType, Color, ColorType, ISize, Image, ImageInfo};

pub struct CachedTexture {
    texture: RefCell<Option<Image>>,
}

impl CachedTexture {
    pub fn new() -> Self {
        Self {
            texture: RefCell::new(None),
        }
    }

    pub fn invalidate(&self) {
        *self.texture.borrow_mut() = None;
    }

    pub fn has_cache(&self) -> bool {
        self.texture.borrow().is_some()
    }

    pub fn draw<F>(&self, cx: &mut DrawContext, canvas: &Canvas, draw_fn: F)
    where
        F: Fn(&mut DrawContext, &Canvas),
    {
        if let Some(image) = self.texture.borrow().as_ref() {
            draw_image(cx, canvas, image);
        } else if let Some(image) = create_texture(cx, canvas, draw_fn) {
            draw_image(cx, canvas, &image);
            *self.texture.borrow_mut() = Some(image);
        }
    }
}

fn create_texture<F>(cx: &mut DrawContext, canvas: &Canvas, draw_fn: F) -> Option<Image>
where
    F: Fn(&mut DrawContext, &Canvas),
{
    let BoundingBox { w, h, .. } = cx.bounds();

    // Create the new texture
    let size = ISize::new(w.ceil() as i32, h.ceil() as i32);
    let info = ImageInfo::new(size, ColorType::RGBA8888, AlphaType::Premul, None);
    let mut surface = canvas.new_surface(&info, None)?;
    let render_canvas = surface.canvas();

    // Draw in the render canvas
    render_canvas.clear(Color::TRANSPARENT);
    draw_fn(cx, render_canvas);
    let image = surface.image_snapshot();
    Some(image)
}

fn draw_image(cx: &mut DrawContext, canvas: &Canvas, image: &Image) {
    let BoundingBox { x, y, .. } = cx.bounds();
    canvas.draw_image(image, (x, y), None);
}
