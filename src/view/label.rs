use device::CURRENT_DEVICE;
use font::{Fonts, font_from_style, NORMAL_STYLE};
use view::{View, Event, Hub, Bus, Align};
use framebuffer::{Framebuffer, UpdateMode};
use geom::Rectangle;
use color::TEXT_NORMAL;
use app::Context;

pub struct Label {
    rect: Rectangle,
    children: Vec<Box<View>>,
    text: String,
    align: Align,
}

impl Label {
    pub fn new(rect: Rectangle, text: String, align: Align) -> Label {
        Label {
            rect,
            children: vec![],
            text,
            align,
        }
    }

    pub fn update(&mut self, text: String, hub: &Hub) {
        self.text = text;
        hub.send(Event::Render(self.rect, UpdateMode::Gui)).unwrap();
    }
}

impl View for Label {
    fn handle_event(&mut self, _evt: &Event, _hub: &Hub, _bus: &mut Bus, _context: &mut Context) -> bool {
        false
    }

    fn render(&self, fb: &mut Framebuffer, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        fb.draw_rectangle(&self.rect, TEXT_NORMAL[0]);

        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;
        let max_width = self.rect.width() as i32 - padding;

        let plan = font.plan(&self.text, Some(max_width as u32), None);

        let dx = self.align.offset(plan.width as i32, self.rect.width() as i32);
        let dy = (self.rect.height() as i32 - x_height) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);

        font.render(fb, TEXT_NORMAL[1], &plan, &pt);
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<View>> {
        &mut self.children
    }
}
