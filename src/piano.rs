use {
    crate::{
        app::Message,
        keyboard::{Key, Keyboard},
    },
    iced::{
        Color,
        Element,
        Point,
        Rectangle,
        Renderer,
        Size,
        Theme,
        Vector,
        mouse,
        widget::{
            Canvas,
            canvas::{self, Fill, Frame, Stroke},
        },
    },
    std::{cmp::Ordering, collections::HashSet},
};

const SHARP_KEY_HEIGHT: f32 = 0.6;

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Released,
    Pressed,
}

#[derive(Default)]
pub struct State {
    pressed_key: Option<Key>,
    bounds: Rectangle,
    translation: Vector,
    scale: Vector,
}

impl State {
    fn update_translation(&mut self, kbd: &Keyboard) {
        let kbd_height = 150.;
        let kbd_width = 23.5 * kbd.num_natural_keys() as f32;
        let desired_aspect_ratio = kbd_width / kbd_height;
        let bounds = self.bounds;
        let widget_aspect_ratio = bounds.width / bounds.height;

        let (width, height) = if widget_aspect_ratio > desired_aspect_ratio {
            (bounds.height * desired_aspect_ratio, bounds.height)
        } else {
            (bounds.width, bounds.width / desired_aspect_ratio)
        };

        self.translation = Vector::new((bounds.width - width) / 2., (bounds.height - height) / 2.);
        self.scale = Vector::new(width, height);
    }

    fn translate(&self, pt: Point) -> Point {
        Point::new(
            (pt.x - self.translation.x) / self.scale.x,
            (pt.y - self.translation.y) / self.scale.y,
        )
    }
}

#[derive(Debug, Clone)]
struct KeyData {
    key: Key,
    offset: Point,
    size: Size,
}

pub struct Piano {
    natural_keys: Vec<KeyData>,
    sharp_keys: Vec<KeyData>,
    pressed_keys: HashSet<Key>,
    keyboard: Keyboard,
}

impl Piano {
    pub fn new(keyboard: Keyboard) -> Self {
        let kbd_width = 1.0;
        let kbd_height = 1.0;
        let num_natural_keys = keyboard.num_natural_keys() as f32;
        let natural_width = kbd_width / num_natural_keys;
        let octave_width = natural_width * 7.;
        let sharp_width = octave_width / 12.;
        let first_key = keyboard.first();
        let octave_offset =
            first_key.pos.natural_idx().unwrap_or_default() as f32 / 7.0 * octave_width;
        let first_octave = first_key.oct;

        let mut nat_idx = 0;
        let mut natural_keys = Vec::new();
        let mut sharp_keys = Vec::new();

        for key in keyboard.iter_keys() {
            if key.is_natural() {
                natural_keys.push(KeyData {
                    key,
                    offset: Point::new(nat_idx as f32 * natural_width, 0.),
                    size: Size::new(natural_width, kbd_height),
                });

                nat_idx += 1;
            } else {
                let cur_octave_offset =
                    (key.oct - first_octave) as f32 * octave_width - octave_offset;
                let scale_idx = key.pos.scale_idx() as f32;

                sharp_keys.push(KeyData {
                    key,
                    offset: Point::new(scale_idx * sharp_width + cur_octave_offset, 0.),
                    size: Size::new(sharp_width, kbd_height * SHARP_KEY_HEIGHT),
                });
            }
        }

        Self {
            natural_keys,
            sharp_keys,
            pressed_keys: Default::default(),
            keyboard,
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    pub fn set_key_state(&mut self, key: Key, state: KeyState) {
        match state {
            KeyState::Pressed => {
                self.pressed_keys.insert(key);
            }

            KeyState::Released => {
                self.pressed_keys.remove(&key);
            }
        }
    }

    fn is_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }

    fn find_key(&self, pt: Point) -> Option<Key> {
        let cmp = |key: &KeyData| {
            if key.offset.x < pt.x {
                if key.offset.x + key.size.width > pt.x {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Greater
            }
        };

        if pt.y < SHARP_KEY_HEIGHT {
            let key = self
                .sharp_keys
                .binary_search_by(cmp)
                .map(|idx| self.sharp_keys[idx].key)
                .ok();

            if key.is_some() {
                return key;
            }
        }

        self.natural_keys
            .binary_search_by(cmp)
            .map(|idx| self.natural_keys[idx].key)
            .ok()
    }
}

impl canvas::Program<Message> for Piano {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                if let Some(cur_pos) = cursor.position_in(bounds) {
                    if let Some(key) = self.find_key(state.translate(cur_pos)) {
                        state.pressed_key = Some(key);

                        let msg = Message::InputEvent(midly::MidiMessage::NoteOn {
                            key: key.to_midi(),
                            vel: 1.into(),
                        });

                        return Some(canvas::Action::publish(msg).and_capture());
                    }
                }
            }

            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if let Some(key) = state.pressed_key.take() {
                    let msg = Message::InputEvent(midly::MidiMessage::NoteOff {
                        key: key.to_midi(),
                        vel: 0.into(),
                    });

                    return Some(canvas::Action::publish(msg).and_capture());
                }
            }

            _ => {}
        };

        if state.bounds != bounds {
            state.bounds = bounds;
            state.update_translation(&self.keyboard);
            Some(canvas::Action::request_redraw())
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let natural_stroke = Stroke::default().with_width(1.);
        let sharp_fill = Fill::from(Color::BLACK);
        let pressed_fill = Fill::from(Color::from_rgb(0.7, 0.7, 0.7));

        let mut frame = Frame::new(renderer, bounds.size());

        frame.with_save(|frame| {
            frame.translate(state.translation);
            frame.scale_nonuniform(state.scale);
            frame.fill_rectangle(Point::ORIGIN, Size::new(1., 1.), Color::WHITE);

            for key in &self.natural_keys {
                if self.is_pressed(&key.key) {
                    frame.fill_rectangle(key.offset, key.size, pressed_fill);
                }

                frame.stroke_rectangle(key.offset, key.size, natural_stroke);
            }
        });

        // TODO: Figure out why sharp keys need an extra 1px offset to align with
        // natural keys.
        frame.with_save(|frame| {
            frame.translate(state.translation + Vector::new(1., 0.));
            frame.scale_nonuniform(state.scale);

            for key in &self.sharp_keys {
                if self.is_pressed(&key.key) {
                    frame.fill_rectangle(key.offset, key.size, pressed_fill);
                } else {
                    frame.fill_rectangle(key.offset, key.size, sharp_fill);
                }
            }
        });

        vec![frame.into_geometry()]
    }
}
