
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

// The very first thing we should do is rip through the entire text and find where our double bracket annotations begin and end


#[derive(Debug)]
enum BracketType {
    Square,
    Curly
}

#[derive(Debug)]
struct BracketLocation {
    start_pos: usize,
    end_pos: usize,
    bracket_type: BracketType
}

impl BracketLocation {

    /// Check if we should completely eliminate the string encoded in this bracket.
    fn should_be_removed(&self, source_str: &str) -> bool {

        match self.bracket_type {
            BracketType::Curly => true,
            BracketType::Square => {
                // We only care about the slice starting 2 indices after our start position
                // with length 4
                let testing_slice = &source_str[self.start_pos + 2..self.start_pos + 6];
                testing_slice == "File"
            }
        }
    }

}

fn mark_bracket_locations(article_text: &str) -> Vec<BracketLocation> {

    let mut locations: Vec<BracketLocation> = vec![];

    // Square bracket counts
    let mut lsb_count= 0;
    let mut rsb_count = 0;
    let mut lsb_open_index = 0;

    // Curly bracket counts
    let mut lcb_count = 0;
    let mut rcb_count = 0;
    let mut lcb_open_index = 0;

    for (idx, c) in article_text.chars().enumerate() {
        // Initial bookkeeping
        match c {
            '[' => {
                if lsb_count == 0 {
                    lsb_open_index = idx;
                }
                lsb_count += 1
            },
            ']' => rsb_count += 1,
            '{' => {
                if lcb_count == 0 {
                    lcb_open_index = idx;
                }
                lcb_count += 1
            },
            '}' => rcb_count += 1,
            _ => ()
        }

        // TODO: consider edge case where \[ is used!

        // Then we have completed a cycle
        if lsb_count != 0 && lsb_count == rsb_count {

            let new_location = BracketLocation {
                start_pos: lsb_open_index,
                end_pos: idx,
                bracket_type: BracketType::Square,
            };

            locations.push(new_location);

            lsb_count = 0;
            rsb_count = 0;
        }

        if lcb_count != 0 && lcb_count == rcb_count {

            let new_location = BracketLocation {
                start_pos: lcb_open_index,
                end_pos: idx,
                bracket_type: BracketType::Curly,
            };

            locations.push(new_location);

            lcb_count = 0;
            rcb_count = 0;
        }

    }

    locations
}

/// A function that strips file links from our article text
#[pyfunction]
fn strip_file_annotations(article_text: &str) -> String {

    let locations = mark_bracket_locations(article_text);

    if locations.is_empty() {
        article_text.to_string()
    } else {

        let first_location = locations.first().unwrap();
        // Initialize with the first chunk
        let mut out_string = article_text[0..first_location.start_pos].to_string();

        if locations.len() != 1 {

            // Handle the middle chunks
            for (prev_location, next_location) in locations[0..locations.len() - 1].iter().zip(&locations[1..locations.len()]) {
                if prev_location.should_be_removed(article_text) {
                    // Push the text from after this location to the start of the next location
                    out_string.push_str(&article_text[prev_location.end_pos + 1..next_location.start_pos]);
                } else {
                    // Let's push the string starting from this location to the start of the next location
                    out_string.push_str(&article_text[prev_location.start_pos + 2..prev_location.end_pos - 1]);
                    out_string.push_str(&article_text[prev_location.end_pos + 1..next_location.start_pos]);
                }
            }
        }

        // Finally handle the final location
        let final_location = locations.last().unwrap();

        if final_location.should_be_removed(article_text) {
            out_string.push_str(&article_text[final_location.end_pos + 1..article_text.len()]);
        } else {
            out_string.push_str(&article_text[final_location.start_pos + 2..final_location.end_pos - 1]);
            out_string.push_str(&article_text[final_location.end_pos + 1..article_text.len()]);
        }

        out_string
    }

}

/// A Python module implemented in Rust.
#[pymodule]
fn wikicleaner(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(strip_file_annotations, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{mark_bracket_locations, strip_file_annotations};

    #[test]
    fn test_marking_locations() {

        let raw_string = r#"== The Month ==
        [[File:Colorful spring garden.jpg|thumb|180px|right|[[Spring]] flowers in April in the [[Northern Hemisphere]].]]
        April comes between [[March]] and [[May]], making it the fourth month of the year. It also comes first in the year out of the four months that have 30 days, as [[June]], [[September]] and [[November]] are later in the year. {{test hehe}}
        "#;

        let locations = mark_bracket_locations(raw_string);
        dbg!(&locations);
    }

    #[test]
    fn test_stripping_brackets() {

        let raw_string = r#"[[Test string]]"#;
        assert_eq!(strip_file_annotations(raw_string), String::from("Test string"));

        let raw_string = r#"[[File:Colorful spring garden.jpg|thumb|180px|right|[[Spring]] flowers in April in the [[Northern Hemisphere]].]] April comes between [[March]] and [[May]]"#;

        let stripped_string = " April comes between March and May";
        assert_eq!(strip_file_annotations(raw_string), String::from(stripped_string));

        let erase_me = "{{durasdjf;alsd}}";
        assert_eq!(strip_file_annotations(erase_me), "".to_string());

    }


}