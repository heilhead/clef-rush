use {
    crate::{
        app::Message,
        keyboard::{Key, Keyboard},
    },
    iced::{
        Color,
        Element,
        Point,
        Renderer,
        Size,
        Theme,
        mouse,
        widget::{
            Canvas,
            canvas::{self, Fill, Frame, Stroke},
        },
    },
    std::{cmp::Ordering, collections::HashSet},
};

const SHARP_KEY_HEIGHT: f32 = 0.66;

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Released,
    Pressed,
}

// #[derive(Debug, Clone)]
// pub enum Message {}

#[derive(Default)]
pub struct State {
    pressed_key: Option<Key>,
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
}

impl Piano {
    pub fn new(kbd: Keyboard) -> Self {
        let kbd_width = 1.0;
        let kbd_height = 1.0;
        let num_keys = kbd.num_keys();
        let num_nat_keys = kbd.num_natural_keys();
        let natural_width = kbd_width / num_nat_keys as f32;
        let hammer_width = kbd_width / (num_keys as f32) as f32;

        let mut nat_idx = 0;
        let mut natural_keys = Vec::new();
        let mut sharp_keys = Vec::new();

        for key in kbd.iter_keys() {
            if key.is_natural() {
                natural_keys.push(KeyData {
                    key,
                    offset: Point::new(nat_idx as f32 * natural_width, 0.),
                    size: Size::new(natural_width, kbd_height),
                });
                nat_idx += 1;
            } else {
                let idx = kbd.natural_index(&key.prev().unwrap()).unwrap() as f32;
                sharp_keys.push(KeyData {
                    key,
                    offset: Point::new((idx + 1.0) * natural_width - hammer_width * 0.5, 0.),
                    size: Size::new(hammer_width, kbd_height * SHARP_KEY_HEIGHT),
                });
            }
        }

        Self {
            natural_keys,
            sharp_keys,
            pressed_keys: Default::default(),
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
                let cur_pos = cursor.position_in(bounds)?;
                let cur_pos = Point::new(cur_pos.x / bounds.width, cur_pos.y / bounds.height);

                if let Some(key) = self.find_key(cur_pos) {
                    state.pressed_key = Some(key);

                    let msg = Message::InputEvent(midly::MidiMessage::NoteOn {
                        key: key.to_midi(),
                        vel: 1.into(),
                    });

                    return Some(canvas::Action::publish(msg).and_capture());
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

        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let stroke = Stroke::default().with_width(2.);
        let sharp_fill = Fill::from(Color::BLACK);
        let pressed_fill = Fill::from(Color::from_rgb(0.7, 0.7, 0.7));
        let kbd_width = bounds.width;
        let kbd_height = bounds.height;
        let offset = |pt: Point| Point::new(pt.x * kbd_width, pt.y * kbd_height);
        let size = |sz: Size| Size::new(sz.width * kbd_width, sz.height * kbd_height);

        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::WHITE);

        for key in &self.natural_keys {
            let offset = offset(key.offset);
            let size = size(key.size);

            if self.is_pressed(&key.key) {
                frame.fill_rectangle(offset, size, pressed_fill);
            }

            frame.stroke_rectangle(offset, size, stroke);
        }

        for key in &self.sharp_keys {
            let offset = offset(key.offset);
            let size = size(key.size);

            if self.is_pressed(&key.key) {
                frame.fill_rectangle(offset, size, pressed_fill);
                frame.stroke_rectangle(offset, size, stroke);
            } else {
                frame.fill_rectangle(offset, size, sharp_fill);
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}
