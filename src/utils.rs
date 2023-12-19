pub fn note_name(note: u8, show_octave: bool) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

    let octave = note / 12;
    let note = note % 12;

    if show_octave {
        format!("{}{}", note_names[note as usize], octave)
    } else {
        note_names[note as usize].to_string()
    }
}
