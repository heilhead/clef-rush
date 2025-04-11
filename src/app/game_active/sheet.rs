use {
    crate::{keyboard::Key, verovio},
    core::{iter, slice},
    rand::distr::SampleString as _,
};

const TEMPLATE_NO_LANDMARKS: &str = include_str!("../../../resources/template.mei");
const TEMPLATE_LANDMARKS: &str = include_str!("../../../resources/template_landmarks.mei");
const TEMPLATE_TREBLE_NOTES: &str = "{{treble_notes}}";
const TEMPLATE_BASS_NOTES: &str = "{{bass_notes}}";

pub async fn generate_svg(treble: Option<&Notes>, bass: Option<&Notes>) -> String {
    verovio::convert_to_svg(generate_mei(treble, bass)).await
}

#[derive(Debug, Clone)]
pub enum Notes {
    Single(Key),
    Chord(Vec<Key>),
}

impl Notes {
    pub fn keys(&self) -> impl Iterator<Item = Key> {
        Iter::new(self)
    }

    fn to_mei(&self) -> String {
        match self {
            Self::Single(key) => {
                let id = generate_xml_id();
                let pname = key.pos.pitch_name();
                let oct = key.oct;
                let inner = if key.pos.is_sharp() {
                    generate_accid_sharp()
                } else {
                    String::new()
                };

                format!(
                    "<note xml:id=\"{id}\" dur=\"1\" pname=\"{pname}\" \
                     oct=\"{oct}\">{inner}</note>"
                )
            }

            Self::Chord(_) => {
                todo!()
            }
        }
    }
}

pub enum Iter<'a> {
    Single(iter::Once<Key>),
    Multiple(iter::Cloned<slice::Iter<'a, Key>>),
}

impl<'a> Iter<'a> {
    fn new(notes: &'a Notes) -> Self {
        match notes {
            Notes::Single(key) => Self::Single(iter::once(key.clone())),
            Notes::Chord(keys) => Self::Multiple(keys.iter().cloned()),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(iter) => iter.next(),
            Self::Multiple(iter) => iter.next(),
        }
    }
}

impl IntoIterator for Notes {
    type IntoIter = IntoIter;
    type Item = Key;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

pub enum IntoIter {
    Single(iter::Once<Key>),
    Multiple(<Vec<Key> as IntoIterator>::IntoIter),
}

impl IntoIter {
    fn new(notes: Notes) -> Self {
        match notes {
            Notes::Single(key) => Self::Single(iter::once(key)),
            Notes::Chord(keys) => Self::Multiple(keys.into_iter()),
        }
    }
}

impl Iterator for IntoIter {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(iter) => iter.next(),
            Self::Multiple(iter) => iter.next(),
        }
    }
}

fn generate_mei(treble: Option<&Notes>, bass: Option<&Notes>) -> String {
    let treble = treble
        .map(|note| note.to_mei())
        .unwrap_or_else(generate_rest);
    let bass = bass.map(|note| note.to_mei()).unwrap_or_else(generate_rest);

    TEMPLATE_LANDMARKS
        .replacen(TEMPLATE_TREBLE_NOTES, &treble, 1)
        .replacen(TEMPLATE_BASS_NOTES, &bass, 1)
}

fn generate_xml_id() -> String {
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 6)
}

fn generate_accid_sharp() -> String {
    let id = generate_xml_id();
    format!("<accid xml:id=\"{id}\" accid=\"s\"/>")
}

fn generate_rest() -> String {
    let id = generate_xml_id();
    format!("<mRest xml:id=\"{id}\" />")
}
