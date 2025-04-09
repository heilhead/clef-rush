use {
    crate::{keyboard::Key, verovio},
    rand::distr::SampleString as _,
    std::borrow::Cow,
};

const TEMPLATE_LANDMARKS: &str = include_str!("../../../resources/template_landmarks.mei");
const TEMPLATE_TREBLE_NOTES: &str = "{{treble_notes}}";
const TEMPLATE_BASS_NOTES: &str = "{{bass_notes}}";

pub enum Notes {
    Single(Key),
    Chord(Vec<Key>),
}

impl Notes {
    fn to_mei(&self) -> String {
        match self {
            Self::Single(key) => {
                let id = generate_xml_id();
                let pname = key.pos.as_str().chars().next().unwrap().to_lowercase();
                let oct = key.oct;

                format!("<note xml:id=\"{id}\" dur=\"1\" pname=\"{pname}\" oct=\"{oct}\" />")
            }

            Self::Chord(keys) => {
                todo!()
            }
        }
    }
}

fn generate_rest() -> String {
    let id = generate_xml_id();
    format!("<mRest xml:id=\"{id}\" />")
}

pub fn generate_mei(treble: Option<Notes>, bass: Option<Notes>) -> String {
    let treble = treble
        .map(|note| note.to_mei())
        .unwrap_or_else(generate_rest);
    let bass = bass.map(|note| note.to_mei()).unwrap_or_else(generate_rest);

    TEMPLATE_LANDMARKS
        .replacen(TEMPLATE_TREBLE_NOTES, &treble, 1)
        .replacen(TEMPLATE_BASS_NOTES, &bass, 1)
}

pub async fn generate_svg(treble: Option<Notes>, bass: Option<Notes>) -> iced::widget::svg::Handle {
    let svg = verovio::convert_to_svg(generate_mei(treble, bass)).await;
    iced::widget::svg::Handle::from_memory(Cow::Owned(svg.as_bytes().into()))
}

fn generate_xml_id() -> String {
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 6)
}
