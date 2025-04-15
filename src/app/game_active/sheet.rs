use {
    crate::{keyboard::Key, verovio},
    derive_more::Display,
    smallvec::SmallVec,
    std::collections::HashMap,
};

#[derive(Debug, Clone)]
pub struct Sheet {
    enable_landmarks: bool,
    clef_split: Key,
    notes: HashMap<Key, Note>,
}

impl Sheet {
    pub fn new(enable_landmarks: bool, notes: &[Note], clef_split: Key) -> Self {
        let notes = HashMap::from_iter(notes.iter().map(|note| (note.key, note.clone())));

        Self {
            enable_landmarks,
            notes,
            clef_split,
        }
    }

    pub fn add_note(&mut self, key: Key, style: Style) {
        self.notes.insert(key, Note::new(key, style));
    }

    pub fn remove_note(&mut self, key: Key) {
        self.notes.remove(&key);
    }

    pub fn set_note_style(&mut self, key: Key, style: Style) {
        self.notes.get_mut(&key).map(|note| note.style = style);
    }

    pub fn render_hint_svg(&self) -> impl Future<Output = String> + use<> {
        let treble_notes = self.treble_iter().collect::<SmallVec<[_; 4]>>();
        let (treble_notes, treble_styles) = if treble_notes.is_empty() {
            (generate_rest(), String::new())
        } else {
            (
                render_notes_mei(&treble_notes),
                render_note_styles(&treble_notes),
            )
        };

        let bass_notes = self.bass_iter().collect::<SmallVec<[_; 4]>>();
        let (bass_notes, bass_styles) = if bass_notes.is_empty() {
            (generate_rest(), String::new())
        } else {
            (
                render_notes_mei(&bass_notes),
                render_note_styles(&bass_notes),
            )
        };

        const TEMPLATE_NO_LANDMARKS: &str = include_str!("../../../resources/template.mei");
        const TEMPLATE_LANDMARKS: &str = include_str!("../../../resources/template_landmarks.mei");
        const TREBLE_NOTES_PAT: &str = "{{treble_notes}}";
        const BASS_NOTES_PAT: &str = "{{bass_notes}}";

        let template = if self.enable_landmarks {
            TEMPLATE_LANDMARKS
        } else {
            TEMPLATE_NO_LANDMARKS
        };

        let mei = template
            .replacen(TREBLE_NOTES_PAT, &treble_notes, 1)
            .replacen(BASS_NOTES_PAT, &bass_notes, 1);

        let styles = format!("{treble_styles} {bass_styles}");

        async move { inject_styles(&verovio::convert_to_svg(mei).await, &styles) }
    }

    fn treble_iter(&self) -> impl Iterator<Item = Note> {
        let clef_split = self.clef_split;

        self.notes
            .values()
            .filter(move |note| note.key >= clef_split)
            .cloned()
    }

    fn bass_iter(&self) -> impl Iterator<Item = Note> {
        let clef_split = self.clef_split;

        self.notes
            .values()
            .filter(move |note| note.key < clef_split)
            .cloned()
    }
}

#[derive(Display, derive_more::Debug, Clone, Copy)]
#[display("id{:016x}", _0)]
#[debug("{}", self)]
pub struct Id(u64);

impl Id {
    pub fn generate() -> Self {
        Self(rand::random())
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Style {
    #[default]
    Default,
    Correct,
    Incorrect,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Id,
    pub key: Key,
    pub style: Style,
}

impl Note {
    fn new(key: Key, style: Style) -> Self {
        Self {
            id: Id::generate(),
            key,
            style,
        }
    }

    fn render_style(&self) -> String {
        const ID_PAT: &str = "{{note-id}}";
        const STYLE_CORRECT: &str = include_str!("../../../resources/styles/note-correct.css");
        const STYLE_INCORRECT: &str = include_str!("../../../resources/styles/note-incorrect.css");

        match &self.style {
            Style::Default => String::new(),
            Style::Correct => STYLE_CORRECT.replacen(ID_PAT, &self.id.to_string(), 1),
            Style::Incorrect => STYLE_INCORRECT.replacen(ID_PAT, &self.id.to_string(), 1),
        }
    }
}

impl From<Key> for Note {
    fn from(key: Key) -> Self {
        Self {
            id: Id::generate(),
            key,
            style: Style::default(),
        }
    }
}

fn generate_accid_sharp() -> String {
    let id = Id::generate();
    format!("<accid xml:id=\"{id}\" accid=\"s\"/>")
}

fn generate_rest() -> String {
    let id = Id::generate();
    format!("<mRest xml:id=\"{id}\" />")
}

fn inject_styles(svg: &str, styles: &str) -> String {
    const REPLACE_PAT: &str = "</style>";
    let styles = format!("{styles}{REPLACE_PAT}");
    svg.replacen(REPLACE_PAT, &styles, 1)
}

fn render_note_styles(notes: &[Note]) -> String {
    notes
        .iter()
        .map(Note::render_style)
        .collect::<SmallVec<[_; 4]>>()
        .join(" ")
}

fn render_notes_mei(notes: &[Note]) -> String {
    if notes.len() == 1 {
        render_note_mei(&notes[0])
    } else {
        let id = Id::generate();
        let mut chord = format!("<chord xml:id=\"{id}\" dur=\"1\">");
        for note in notes {
            chord.push_str(&render_note_mei(&note));
        }
        chord.push_str("</chord>");
        chord
    }
}

fn render_note_mei(note: &Note) -> String {
    let id = &note.id;
    let pname = note.key.pos.pitch_name();
    let oct = note.key.oct;
    let inner = if note.key.pos.is_sharp() {
        generate_accid_sharp()
    } else {
        String::new()
    };

    format!("<note xml:id=\"{id}\" dur=\"1\" pname=\"{pname}\" oct=\"{oct}\">{inner}</note>")
}
