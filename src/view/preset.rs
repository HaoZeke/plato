use device::CURRENT_DEVICE;
use geom::{Rectangle, CornerSpec, CycleDir};
use font::{Fonts, font_from_style, NORMAL_STYLE};
use view::{View, Event, Hub, Bus};
use view::BORDER_RADIUS_MEDIUM;
use framebuffer::{Framebuffer, UpdateMode};
use input::{DeviceEvent, FingerStatus};
use gesture::GestureEvent;
use color::{TEXT_NORMAL, TEXT_INVERTED_HARD};
use unit::scale_by_dpi;
use app::Context;

pub struct Preset {
    rect: Rectangle,
    children: Vec<Box<View>>,
    kind: PresetKind,
    active: bool,
}

pub enum PresetKind {
    Normal(String, usize),
    Page(CycleDir),
}

impl Preset {
    pub fn new(rect: Rectangle, kind: PresetKind) -> Preset {
        Preset {
            rect,
            children: vec![],
            kind,
            active: false,
        }
    }
}

impl View for Preset {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, bus: &mut Bus, _context: &mut Context) -> bool {
        match *evt {
            Event::Device(DeviceEvent::Finger { status, ref position, .. }) => {
                match status {
                    FingerStatus::Down if self.rect.includes(position) => {
                        self.active = true;
                        hub.send(Event::Render(self.rect, UpdateMode::Fast)).unwrap();
                        true
                    },
                    FingerStatus::Up if self.active => {
                        self.active = false;
                        hub.send(Event::Render(self.rect, UpdateMode::Gui)).unwrap();
                        true
                    },
                    _ => false,
                }
            },
            Event::Gesture(GestureEvent::Tap(ref center)) if self.rect.includes(center) => {
                match self.kind {
                    PresetKind::Normal(_, index) => bus.push_back(Event::LoadPreset(index)),
                    PresetKind::Page(dir) => bus.push_back(Event::Page(dir)),
                }
                true
            },
            Event::Gesture(GestureEvent::HoldFinger(ref center)) if self.rect.includes(center) => {
                if let PresetKind::Normal(_, index) = self.kind {
                    bus.push_back(Event::TogglePresetMenu(self.rect, index)); 
                }
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut Framebuffer, fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        let (scheme, border_radius) = if self.active {
            (TEXT_INVERTED_HARD, scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32)
        } else {
            (TEXT_NORMAL, 0)
        };

        fb.draw_rounded_rectangle(&self.rect, &CornerSpec::Uniform(border_radius), scheme[0]);

        let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
        let x_height = font.x_heights.0 as i32;
        let padding = font.em() as i32;
        let max_width = self.rect.width() as i32 - padding;

        let name = match self.kind {
            PresetKind::Normal(ref text, _) => text,
            _ => "…",
        };

        let plan = font.plan(name, Some(max_width as u32), None);

        let dx = (self.rect.width() as i32 - plan.width as i32) / 2;
        let dy = (self.rect.height() as i32 - x_height) / 2;
        let pt = pt!(self.rect.min.x + dx, self.rect.max.y - dy);

        font.render(fb, scheme[1], &plan, &pt);
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
