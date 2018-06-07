use std::sync::mpsc;
use device::{CURRENT_DEVICE, BAR_SIZES};
use framebuffer::{Framebuffer, UpdateMode};
use geom::{Rectangle, CornerSpec, BorderSpec};
use font::{Fonts, font_from_style, NORMAL_STYLE};
use view::{View, Event, Hub, Bus, ViewId, EntryId, SliderId, Align};
use view::{THICKNESS_LARGE, BORDER_RADIUS_MEDIUM};
use view::label::Label;
use view::button::Button;
use view::slider::Slider;
use view::icon::Icon;
use view::presets_list::PresetsList;
use view::common::shift;
use frontlight::LightLevels;
use gesture::GestureEvent;
use input::FingerStatus;
use settings::{LightPreset, guess_frontlight};
use color::{BLACK, WHITE};
use unit::scale_by_dpi;
use app::Context;

const LABEL_SAVE: &str = "Save";
const LABEL_GUESS: &str = "Guess";

pub struct FrontlightWindow {
    rect: Rectangle,
    children: Vec<Box<View>>,
}

impl FrontlightWindow {
    pub fn new(context: &mut Context) -> FrontlightWindow {
        let fonts = &mut context.fonts;
        let levels = context.frontlight.levels();
        let presets = &context.settings.frontlight_presets;
        let mut children = Vec::new();
        let dpi = CURRENT_DEVICE.dpi;
        let (width, height) = CURRENT_DEVICE.dims;
        let &(small_height, _) = BAR_SIZES.get(&(height, dpi)).unwrap();
        let thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as i32;
        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;

        let (x_height, padding) = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            (font.x_heights.0 as i32, font.em() as i32)
        };

        let window_width = width as i32 - 2 * padding;

        let mut window_height = small_height as i32 * 3 + 2 * padding;

        if CURRENT_DEVICE.has_natural_light() {
            window_height += small_height as i32;
        }

        if !presets.is_empty() {
            window_height += small_height as i32;
        }

        let dx = (width as i32 - window_width) / 2;
        let dy = (height as i32 - window_height) / 3;

        let rect = rect![dx, dy, dx + window_width, dy + window_height];

        let close_icon = Icon::new("close",
                                   rect![rect.max.x - small_height as i32,
                                         rect.min.y + thickness,
                                         rect.max.x - thickness,
                                         rect.min.y + small_height as i32],
                                   Event::Close(ViewId::Frontlight))
                              .corners(Some(CornerSpec::Uniform(border_radius - thickness)));

        children.push(Box::new(close_icon) as Box<View>);

        let label = Label::new(rect![rect.min.x + small_height as i32,
                                     rect.min.y + thickness,
                                     rect.max.x - small_height as i32,
                                     rect.min.y + small_height as i32],
                               "Frontlight".to_string(),
                               Align::Center);

        children.push(Box::new(label) as Box<View>);

        let mut button_y = rect.min.y + 2 * small_height as i32;

        if CURRENT_DEVICE.has_natural_light() {
            let max_label_width = {
                let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
                ["Intensity", "Warmth"].iter().map(|t| font.plan(t, None, None).width)
                                                           .max().unwrap() as i32
            };

            for (index, slider_id) in [SliderId::LightIntensity, SliderId::LightWarmth].iter().enumerate() {
                let min_y = rect.min.y + (index + 1) as i32 * small_height as i32;
                let label = Label::new(rect![rect.min.x + padding,
                                             min_y,
                                             rect.min.x + 2 * padding + max_label_width,
                                             min_y + small_height as i32],
                                       slider_id.label(),
                                       Align::Right(padding / 2));
                children.push(Box::new(label) as Box<View>);

                let value = if *slider_id == SliderId::LightIntensity {
                    levels.intensity
                } else {
                    levels.warmth
                };

                let slider = Slider::new(rect![rect.min.x + max_label_width + 3 * padding,
                                               min_y,
                                               rect.max.x - padding,
                                               min_y + small_height as i32],
                                         *slider_id,
                                         value,
                                         0.0,
                                         100.0);
                children.push(Box::new(slider) as Box<View>);
            }

            button_y += small_height as i32;
        } else {
                let min_y = rect.min.y + small_height as i32;
                let slider = Slider::new(rect![rect.min.x + padding,
                                               min_y,
                                               rect.max.x - padding,
                                               min_y + small_height as i32],
                                         SliderId::LightIntensity,
                                         levels.intensity,
                                         0.0,
                                         100.0);
                children.push(Box::new(slider) as Box<View>);
        }

        let max_label_width = {
            let font = font_from_style(fonts, &NORMAL_STYLE, dpi);
            [LABEL_SAVE, LABEL_GUESS].iter().map(|t| font.plan(t, None, None).width)
                                                         .max().unwrap() as i32
        };

        let button_height = 4 * x_height;

        let button_save = Button::new(rect![rect.min.x + 3 * padding,
                                            button_y + small_height as i32 - button_height,
                                            rect.min.x + 5 * padding + max_label_width,
                                            button_y + small_height as i32],
                                      Event::Save,
                                      LABEL_SAVE.to_string());
        children.push(Box::new(button_save) as Box<View>);

        let button_guess = Button::new(rect![rect.max.x - 5 * padding - max_label_width,
                                             button_y + small_height as i32 - button_height,
                                             rect.max.x - 3 * padding,
                                             button_y + small_height as i32],
                                       Event::Guess,
                                       LABEL_GUESS.to_string()).disabled(presets.len() < 2);
        children.push(Box::new(button_guess) as Box<View>);

        if !presets.is_empty() {
            let presets_rect = rect![rect.min.x + thickness + 4 * padding,
                                     rect.max.y - small_height as i32 - 2 * padding,
                                     rect.max.x - thickness - 4 * padding,
                                     rect.max.y - thickness - 2 * padding];
            let mut presets_list = PresetsList::new(presets_rect);
            let (tx, _rx) = mpsc::channel();
            presets_list.update(&presets, &tx, fonts);
            children.push(Box::new(presets_list) as Box<View>);
        }

        FrontlightWindow {
            rect,
            children,
        }
    }

    fn toggle_presets(&mut self, enable: bool, hub: &Hub, context: &mut Context) {
        let dpi = CURRENT_DEVICE.dpi;
        let (_, height) = CURRENT_DEVICE.dims;
        let &(small_height, _) = BAR_SIZES.get(&(height, dpi)).unwrap();

        if enable {
            let thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as i32;
            let padding = {
                let font = font_from_style(&mut context.fonts, &NORMAL_STYLE, dpi);
                font.em() as i32
            };
            shift(self, &pt!(0, -(small_height as i32) / 2));
            self.rect.max.y += small_height as i32;
            let (tx, _rx) = mpsc::channel();
            let presets_rect = rect![self.rect.min.x + thickness + 4 * padding,
                                     self.rect.max.y - small_height as i32 - 2 * padding,
                                     self.rect.max.x - thickness - 4 * padding,
                                     self.rect.max.y - thickness - 2 * padding];
            let mut presets_list = PresetsList::new(presets_rect);
            presets_list.update(&context.settings.frontlight_presets, &tx, &mut context.fonts);
            self.children.push(Box::new(presets_list) as Box<View>);
            hub.send(Event::Render(self.rect, UpdateMode::Gui)).unwrap();
        } else {
            self.children.pop();
            hub.send(Event::Expose(self.rect)).unwrap();
            shift(self, &pt!(0, small_height as i32 / 2));
            self.rect.max.y -= small_height as i32;
        }
    }

    fn set_frontlight_levels(&mut self, frontlight_levels: &LightLevels, hub: &Hub, context: &mut Context) {
        let LightLevels { intensity, warmth } = *frontlight_levels;
        context.frontlight.set_intensity(intensity);
        context.frontlight.set_warmth(warmth);
        if CURRENT_DEVICE.has_natural_light() {
            if let Some(slider_intensity) = self.child_mut(3).downcast_mut::<Slider>() {
                slider_intensity.value = intensity;
                hub.send(Event::Render(*slider_intensity.rect(), UpdateMode::Gui)).unwrap();
            }
            if let Some(slider_warmth) = self.child_mut(5).downcast_mut::<Slider>() {
                slider_warmth.value = warmth;
                hub.send(Event::Render(*slider_warmth.rect(), UpdateMode::Gui)).unwrap();
            }
        } else {
            if let Some(slider_intensity) = self.child_mut(2).downcast_mut::<Slider>() {
                slider_intensity.value = intensity;
                hub.send(Event::Render(*slider_intensity.rect(), UpdateMode::Gui)).unwrap();
            }
        }
    }

    fn update_presets(&mut self, hub: &Hub, context: &mut Context) {
        let len = self.len();
        if let Some(presets_list) = self.child_mut(len - 1).downcast_mut::<PresetsList>() {
            presets_list.update(&context.settings.frontlight_presets, hub, &mut context.fonts);
        }
    }
}

impl View for FrontlightWindow {
    fn handle_event(&mut self, evt: &Event, hub: &Hub, _bus: &mut Bus, context: &mut Context) -> bool {
        match *evt {
            Event::Slider(SliderId::LightIntensity, value, FingerStatus::Up) => {
                context.frontlight.set_intensity(value);
                true
            },
            Event::Slider(SliderId::LightWarmth, value, FingerStatus::Up) => {
                context.frontlight.set_warmth(value);
                true
            },
            Event::Gesture(GestureEvent::Tap(ref center)) if !self.rect.includes(center) => {
                hub.send(Event::Close(ViewId::Frontlight)).unwrap();
                true
            },
            Event::Gesture(..) => true,
            Event::Save => {
                let lightsensor_level = if CURRENT_DEVICE.has_lightsensor() {
                    context.lightsensor.level().ok()
                } else {
                    None
                };
                let light_preset = LightPreset {
                    lightsensor_level,
                    frontlight_levels: context.frontlight.levels(),
                    .. Default::default()
                };
                context.settings.frontlight_presets.push(light_preset);
                context.settings.frontlight_presets.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                if context.settings.frontlight_presets.len() == 1 {
                    self.toggle_presets(true, hub, context);
                } else {
                    if context.settings.frontlight_presets.len() == 2 {
                        let index = self.len() - 2;
                        if let Some(button_guess) = self.child_mut(index).downcast_mut::<Button>() {
                            button_guess.disabled = false;
                            hub.send(Event::Render(*button_guess.rect(), UpdateMode::Gui)).unwrap();
                        }
                    }
                    self.update_presets(hub, context);
                }
                true
            },
            Event::Select(EntryId::RemovePreset(index)) => {
                if index < context.settings.frontlight_presets.len() {
                    context.settings.frontlight_presets.remove(index);
                    if context.settings.frontlight_presets.is_empty() {
                        self.toggle_presets(false, hub, context);
                    } else {
                        if context.settings.frontlight_presets.len() == 1 {
                            let index = self.len() - 2;
                            if let Some(button_guess) = self.child_mut(index).downcast_mut::<Button>() {
                                button_guess.disabled = true;
                                hub.send(Event::Render(*button_guess.rect(), UpdateMode::Gui)).unwrap();
                            }
                        }
                        self.update_presets(hub, context);
                    }
                }
                true
            },
            Event::LoadPreset(index) => {
                let frontlight_levels = context.settings.frontlight_presets[index].frontlight_levels;
                self.set_frontlight_levels(&frontlight_levels, hub, context);
                true
            },
            Event::Guess => {
                let lightsensor_level = if CURRENT_DEVICE.has_lightsensor() {
                    context.lightsensor.level().ok()
                } else {
                    None
                };
                if let Some(ref frontlight_levels) = guess_frontlight(lightsensor_level, &context.settings.frontlight_presets) {
                    self.set_frontlight_levels(frontlight_levels, hub, context);
                }
                true
            },
            _ => false,
        }
    }

    fn render(&self, fb: &mut Framebuffer, _fonts: &mut Fonts) {
        let dpi = CURRENT_DEVICE.dpi;

        let border_radius = scale_by_dpi(BORDER_RADIUS_MEDIUM, dpi) as i32;
        let border_thickness = scale_by_dpi(THICKNESS_LARGE, dpi) as u16;

        fb.draw_rounded_rectangle_with_border(&self.rect,
                                              &CornerSpec::Uniform(border_radius),
                                              &BorderSpec { thickness: border_thickness,
                                                            color: BLACK },
                                              &WHITE);
    }

    fn is_background(&self) -> bool {
        true
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
