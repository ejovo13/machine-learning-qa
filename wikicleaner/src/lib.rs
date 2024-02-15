use std::error::Error;
use regex::Regex;
use once_cell::sync::Lazy;

use pyo3::prelude::*;


#[derive(Debug, Clone)]
enum BracketType {
    Square,
    Curly,
    // Angle
}

#[derive(Debug, Clone)]
struct BracketLocation {
    start_pos: usize,
    end_pos: usize,
    bracket_type: BracketType,
}

impl BracketLocation {
    // /// Check if we should completely eliminate the string encoded in this bracket.
    // fn should_be_removed(&self, source_str: &str) -> bool {
    //     match self.bracket_type {
    //         BracketType::Curly => true,
    //         BracketType::Square => {
    //             // We only care about the slice starting 2 indices after our start position
    //             // with length 4
    //             let test_string: String = source_str[self.start_pos..]
    //                 .chars()
    //                 .skip(2)
    //                 .take(4)
    //                 .collect();
    //             test_string == *"File"
    //         }
    //     }
    // }

    /// Check if this bracket location is doubled up [[]] or singled []
    fn is_doubled(&self, source_str: &str) -> bool {
        // Honestly just check if the first two characters are the same
        let mut first_two = source_str[self.start_pos..]
            .chars()
            .take(2)
            .collect::<String>();
        let second = first_two.pop().unwrap_or('<');
        let first = first_two.pop().unwrap_or('>');

        second == first
    }

    /// Check if the inner content of our location has an alias (e.g. `[[Referenced Article | appearence in this article]]`)
    fn is_aliased_link(&self, source_str: &str) -> bool {
        self.inner(source_str).contains('|')
    }

    /// Return only the name of the referenced article if the inner content is an aliased link
    ///
    /// See [BracketLocation::is_aliased_link] for more details
    fn referenced_article(&self, source_str: &str) -> String {

        let inner = self.inner(source_str);
        let pipe_index = inner.find('|');

        match pipe_index {
            // Then we are dealing with an aliased link!
            Some(pipe_index) => String::from(&inner[pipe_index + 1..inner.len()]),
            None => inner
        }
    }

    /// Check if the inner content of our location is an annotation (e.g. `[[File:SOME_FILE.jpg]]`)
    ///
    /// This function naively checks for the presence of a semicolon (`':'`) character.
    fn is_annotation(&self, source_str: &str) -> bool {
        match self.bracket_type {
            BracketType::Square => self.inner(source_str).contains(':'),
            BracketType::Curly => false,
            // BracketType::Angle => false
        }
    }

    /// Check to make sure that the location generated is valid.
    fn is_valid(&self) -> bool {
        self.start_pos < self.end_pos
    }

    fn slice(&self, source_str: &str) -> String {
        source_str[self.start_pos..self.end_pos + 1].to_string()
    }

    /// Count the number of characters between the start and endpoints
    fn len(&self, source_str: &str) -> usize {
        let slice = &source_str[self.start_pos..self.end_pos + 1];
        slice.chars().count()
    }

    fn n_brackets(&self, source_str: &str) -> usize {
        if self.is_doubled(source_str) {
            2
        } else {
            1
        }
    }

    /// Return the contents of the sting inside of our brackets. Necessary to avoid a silly UTF-8 error that I made.
    fn inner(&self, source_str: &str) -> String {
        // We want to skip the first two characters, and drop the last two!
        let slice = &source_str[self.start_pos..self.end_pos];
        let num_brackets = self.n_brackets(source_str);
        // println!("Num_brackets: {}, slice: {}", num_brackets, self.slice(source_str));

        let len_inner = self.len(source_str);
        if len_inner == 0 {
            "".to_string()
        } else if len_inner < num_brackets {
            dbg!(self);
            panic!("What the fuck is going on? \n source: {}\n len_inner: {}, slice: {}", source_str, len_inner, self.slice(source_str));
        } else {

            slice
                .chars()
                .take(self.len(source_str) - num_brackets)
                .skip(num_brackets)
                .collect()
        }
    }

    fn next_index(&self, source_str: &str) -> usize {
        let remaining_slice = &source_str[self.end_pos..source_str.len()];
        let (next_idx, _) = remaining_slice.char_indices().next().unwrap();
        next_idx + self.end_pos + 1
    }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Result of [validate_square_bracket_placement]. Beware, [BracketValidation::Equal] does not guarentee
/// that our string is properly formed.
enum BracketValidation {
    /// There is at least one dangling ']' bracket
    DanglingRight,
    /// There is at least one unmatched '[' bracket
    UnmatchedLeft,
    Equal
}

/// Count to make sure that there are the same number of '[' as there are ']'
fn validate_square_bracket_placement(article_text: &str) -> BracketValidation {

    // +1 for a left bracket, -1 for a right bracket
    let mut bracket_count = 0;

    for c in article_text.chars() {
        match c {
            '[' => {
                bracket_count += 1;
            }
            ']' => {
                // Protect against dangling ']' character
                bracket_count -= 1;
                if bracket_count < 0 { return BracketValidation::DanglingRight }
            },
            _ => ()
        }
    }

    if bracket_count == 0 {
        BracketValidation::Equal
    } else {
        BracketValidation::UnmatchedLeft
    }
}

// /// Try and remove all html tags and comments from our article
// fn strip_html(article_text: &str) -> String {

//     let re = Regex::new(r"(?P<html><.*?>)").unwrap();

//     let caps = re.captures(article_text).unwrap();

//     for capture in caps {
//     }

//     todo!()
// }

/// Remove any comments of the form `"<!-- ATTENTION CLOSING ADMINISTRATOR -->"` using a regex
fn strip_html_comments(article_text: &str) -> String {
    static RE_HTML: Lazy<Regex> = Lazy::new(|| Regex::new(r"<!.*?>").unwrap());
    RE_HTML.replace_all(article_text, "").to_string()
}

fn strip_newline(article_text: &str) -> String {
    static RE_NEWLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n").unwrap());
    RE_NEWLINE.replace_all(article_text, "").to_string()
}

/// Remove
fn strip_references(article_text: &str) -> String {
    static RE_REFERENCES: Lazy<Regex> = Lazy::new(|| Regex::new(r"==References==").unwrap());
    RE_REFERENCES.replace_all(article_text, "").to_string()
}

/// Retrieve the very first bracket location
fn first_bracket_location(article_text: &str) -> Option<BracketLocation> {
    let locations = mark_bracket_locations(article_text);
    locations.first().cloned()
}

/// Mark the locations of curly and square brackets.
///
/// This function is robust against dangling right `']'` brackets, but cannot
/// protect agains unmatched left `'['` brackets. Use [[validate_square_bracket_placement]] to
/// defend against [BracketValidation::DanglingLeft] edge cases.
fn mark_bracket_locations(article_text: &str) -> Vec<BracketLocation> {
    let mut locations: Vec<BracketLocation> = vec![];

    // Square bracket counts
    let mut lsb_count = 0;
    let mut rsb_count = 0;
    let mut lsb_open_index = 0;

    // Curly bracket counts
    let mut lcb_count = 0;
    let mut rcb_count = 0;
    let mut lcb_open_index = 0;

    // // Angle bracket counts.. hmm we are starting to repeat code...
    // let mut lab_count = 0;
    // let mut rab_count = 0;
    // let mut rab_open_index = 0;

    // for (idx, c) in article_text.graphemes(true).enumerate() {
    for (idx, c) in article_text.char_indices() {
        // Initial bookkeeping
        match c {
            '[' => {
                if lsb_count == 0 {
                    lsb_open_index = idx;
                }
                lsb_count += 1
            }
            ']' => {
                // Protect against danglins ']' character
                if lsb_count != 0 {
                    rsb_count += 1
                }
            },
            '{' => {
                if lcb_count == 0 {
                    lcb_open_index = idx;
                }
                lcb_count += 1
            }
            '}' => rcb_count += 1,
            _ => (),
        }

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

/// Modify our locations to avoid some supreme bullshit
///
/// Consider this edge case:
///
/// [[some text {{FUCK YOU BITCH}} ]]
fn sanitize_locations(locations: Vec<BracketLocation>) -> Vec<BracketLocation> {
    let n = locations.len();

    if n > 1 {
        let mut collected = locations[1..n]
            .iter()
            .zip(&locations[0..n - 1])
            .filter_map(|(next, prev)| {
                if next.start_pos > prev.end_pos {
                    Some(next.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<BracketLocation>>();
        collected.insert(0, locations[0].clone());
        collected
    } else {
        locations
    }
}

/// Blindly remove all encoded by `locations` from `article_text`.
fn blindly_strip_locations(article_text: &str, locations: Vec<BracketLocation>) -> String {
    // Now we want to completely remove all of the curly locations!
    if locations.is_empty() {
        article_text.to_string()
    } else {
        // Initialize with the first chunk
        let first_location = locations.first().unwrap();
        let mut out_string = article_text[0..first_location.start_pos].to_string();

        if locations.len() != 1 {
            // Create a paired iterator
            let prev_locations = &locations[0..locations.len() - 1];
            let next_locations = &locations[1..locations.len()];
            let paired_iter = prev_locations.iter().zip(next_locations);

            // Handle the middle chunks
            for (prev_location, next_location) in paired_iter {
                out_string.push_str(
                    &article_text[prev_location.next_index(article_text)..next_location.start_pos],
                );
            }
        }

        // Finally handle the final chunk
        let final_location = locations.last().unwrap();

        out_string
            .push_str(&article_text[final_location.next_index(article_text)..article_text.len()]);

        out_string
    }
}

/// Remove double brackets from `article_text`
fn strip_double_brackets(article_text: &str) -> String {
    let locations = mark_bracket_locations(article_text);
    let locations: Vec<BracketLocation> = locations
        .into_iter()
        .filter(|b| b.is_doubled(article_text))
        .collect();

    let locations = sanitize_locations(locations);

    // Now we want to effectively remove the brackets by building up our string
    if locations.is_empty() {
        article_text.to_string()
    } else {
        // Initialize with the first chunk
        let first_location = locations.first().unwrap();
        let mut out_string = article_text[0..first_location.start_pos].to_string();

        if locations.len() != 1 {
            // Create a paired iterator
            let prev_locations = &locations[0..locations.len() - 1];
            let next_locations = &locations[1..locations.len()];
            let paired_iter = prev_locations.iter().zip(next_locations);

            // Handle the middle chunks
            for (prev_location, next_location) in paired_iter {
                out_string.push_str(&prev_location.referenced_article(article_text));
                out_string.push_str(
                    &article_text[prev_location.next_index(article_text)..next_location.start_pos],
                );
            }
        }

        // Finally handle the final chunk
        let final_location = locations.last().unwrap();

        out_string.push_str(&final_location.referenced_article(article_text));
        out_string.push_str(
            &article_text[final_location.next_index(article_text)..article_text.len()]
        );

        out_string
    }
}

/// Remove any instance of {{ANY TEXT HERE}} from `article_text`.
fn strip_double_brackets_curly(article_text: &str) -> String {
    let locations = mark_bracket_locations(article_text);
    let curly_locations: Vec<BracketLocation> = locations
        .into_iter()
        .filter(|b| matches!(b.bracket_type, BracketType::Curly))
        .collect();

    blindly_strip_locations(article_text, curly_locations)
}

/// Remove any instances of [[File:*]] from `article_text`.
fn strip_annotations(article_text: &str) -> String {
    let locations = mark_bracket_locations(article_text);
    // Only keep annotations
    let locations: Vec<BracketLocation> = locations
        .into_iter()
        .filter(|b| b.is_annotation(article_text))
        .collect();

    blindly_strip_locations(article_text, locations)
}

/// Strip annotations `"[[File:annotation]]"` => `""`, double brackets `"[[String]]"` => `"String"`, and tags `"{{tag}}"` => `""` from `article_text`
#[pyfunction]
fn clean_article_text(article_text: &str) -> String {
    let sans_curly = strip_double_brackets_curly(article_text);
    let sans_annotations = strip_annotations(&sans_curly);
    strip_double_brackets(&sans_annotations)
}

/// Strip html comments and other minor cleanup like removing newline characters.
#[pyfunction]
fn post_processing(article_text: &str) -> String {
    let sans_comments = strip_html_comments(article_text);
    let sans_newline = strip_newline(&sans_comments);
    strip_references(&sans_newline)
}

/// A Python module implemented in Rust.
#[pymodule]
fn wikicleaner(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(clean_article_text, m)?)?;
    m.add_function(wrap_pyfunction!(post_processing, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        clean_article_text, mark_bracket_locations, sanitize_locations, strip_annotations, strip_double_brackets, strip_double_brackets_curly, strip_html_comments, strip_newline
    };

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
    fn test_next_pos() {
        let raw_string = r#"[[Test string]]"#;
        let locations = mark_bracket_locations(raw_string);
        let loc = &locations[0];
        dbg!(loc.next_index(raw_string));
        dbg!(loc.inner(raw_string));
        dbg!(loc.slice(raw_string));
    }



    #[test]
    fn something_wrong() {
        let raw_string = r#""{{monththisyear|8}}\n'''August''' (Aug.) is the eighth [[month]] of the [[year]] in the [[Gregorian calendar]], coming between [[July]] and [[September]]. It has 31 [[day]]s. It is named after the Roman emperor [[Augustus Caesar]].\n\nAugust does not begin on the same day of the week as any other month in [[common year]]s, but begins on the same day of the week as [[February]] in [[leap year]]s. August always ends on the same day of the week as [[November]].\n\n== The Month ==\n[[File:Hw-augustus.jpg|thumb|120px|right|Roman Emperor [[Augustus Caesar]], after whom August is named]]\nThis month was first called ''Sextilis'' in [[Latin]], because it was the sixth month in the old [[Roman calendar]]. The Roman calendar began in March about 735&nbsp;BC with [[Romulus and Remus|Romulus]]. [[October]] was the eighth month. August was the eighth month when January or February were added to the start of the year by King [[Numa Pompilius]] about 700&nbsp;BC. Or, when those two months were moved from the end to the beginning of the year by the decemvirs about 450&nbsp;BC (Roman writers disagree). In [[153 BC]] [[January 1]] was determined as the beginning of the year.\n\nAugust is named for [[Augustus Caesar]] who became [[Roman consul]] in this month.<ref name=\"infoplease\">{{Citation |title=History of August |url=http://www.infoplease.com/spot/history-of-august.html }}</ref> The month has 31 days because [[Julius Caesar]] added two days when he created the [[Julian calendar]] in [[45 BC]]. August is after July and before September.\n\nAugust, in either [[hemisphere]], is the seasonal equivalent of [[February]] in the other. In the [[Northern hemisphere]] it is a [[summer]] month and it is a [[winter]] month in the [[Southern hemisphere]].\n\nNo other month in [[common year]]s begins on the same day of the week as August, but August begins on the same day of the week as [[February]] in [[leap year]]s. August ends on the same day of the week as [[November]] every year, as each other's last days are 13 weeks (91 days) apart.\n\nIn common years, August starts on the same day of the week as [[March]] and [[November]] of the previous year, and in leap years, [[June]] of the previous year. In common years, August finishes on the same day of the week as March and June of the previous year, and in leap years, [[September]] of the previous year. In common years immediately after other common years, August starts on the same day of the week as February of the previous year.\n\nIn years immediately before common years, August starts on the same day of the week as [[May]] of the following year, and in years immediately before leap years, [[October]] of the following year. In years immediately before common years, August finishes on the same day of the week as May of the following year, and in years immediately before leap years, [[February]] and October of the following year.\n\n== August observances ==\n=== Fixed observances and events ===\n[[File:Schweizerflaggen St. Gallen.JPG|thumb|150px|right|[[Flag]]s celebrating [[Switzerland]]'s national day on [[August 1]].]]\n* [[August 1]] {{ndash}} National Day of [[Switzerland]]\n* [[August 1]] {{ndash}} Independence Day ([[Benin]])\n* [[August 1]] {{ndash}} Emancipation Day ([[Bermuda]], [[Guyana]], [[Jamaica]], [[Barbados]], [[Trinidad and Tobago]])\n* [[August 1]] {{ndash}} Army Day ([[People's Republic of China]])\n* [[August 1]] {{ndash}} [[Lammas Day|Lammas]], cross-quarter day in the [[Celts|Celtic]] [[calendar]]\n* [[August 1]] {{ndash}} Statehood Day ([[Colorado]])\n* [[August 2]] {{ndash}} Republic Day ([[Republic of Macedonia]])\n* [[August 2]] {{ndash}} Emancipation Day ([[Bahamas]])\n* [[August 3]] {{ndash}} Independence Day ([[Niger]])\n* [[August 5]] {{ndash}} Independence Day ([[Burkina Faso]])\n* [[August 5]] {{ndash}} Victory Day ([[Croatia]])\n* [[August 6]] {{ndash}} Independence Day ([[Bolivia]])\n* [[August 6]] {{ndash}} Independence Day ([[Jamaica]])\n* [[August 7]] {{ndash}} Independence Day ([[Ivory Coast]])\n* [[August 8]] {{ndash}} Father's Day ([[Taiwan]])\n* [[August 9]] {{ndash}} National Day of [[Singapore]]\n* [[August 9]] {{ndash}} Day of the Indigenous People ([[Suriname]])\n* [[August 9]] {{ndash}} National Women's Day ([[South Africa]])\n* [[August 10]] {{ndash}} Independence Day ([[Ecuador]])\n* [[August 10]] {{ndash}} [[Missouri]] Day\n* [[August 11]] {{ndash}} Independence Day ([[Chad]])\n* [[August 12]] {{ndash}} Perseid [[Meteor]] Shower\n* [[August 12]] {{ndash}} Queen Sirikit's Birthday ([[Thailand]])\n* [[August 13]] {{ndash}} Independence Day ([[Central African Republic]])\n* [[August 14]] {{ndash}} Independence Day ([[Pakistan]])\n* [[August 15]] {{ndash}} [[Assumption of Mary]] in Western [[Christianity]]\n* [[August 15]] {{ndash}} Independence Day ([[India]])\n* [[August 15]] {{ndash}} Independence Day ([[Republic of the Congo]])\n* [[August 15]] {{ndash}} Independence Day ([[Bahrain]])\n* [[August 15]] {{ndash}} National Day of [[South Korea]]\n* [[August 15]] {{ndash}} National Day of [[Liechtenstein]]\n* [[August 15]] {{ndash}} [[Victory]] in [[Japan]] Day\n* [[August 17]] {{ndash}} Independence Day ([[Indonesia]])\n* [[August 17]] {{ndash}} Independence Day ([[Gabon]])\n* [[August 19]] {{ndash}} World [[Humanitarian]] Day\n* [[August 19]] {{ndash}} Independence Day ([[Afghanistan]])\n* [[August 20]] {{ndash}} Feast day of Stephen I of [[Hungary]]\n* [[August 20]] {{ndash}} Regaining of Independence ([[Estonia]])\n* [[August 21]] {{ndash}} Admission Day ([[Hawaii]])\n* [[August 21]] {{ndash}} Ninoy Aquino Day ([[Philippines]])\n* [[August 21]] {{ndash}} [[Saint Helena]] Day\n* [[August 22]] {{ndash}} Start of [[Ashenda]] ([[Ethiopia]] and [[Eritrea]])\n* [[August 23]] {{ndash}} National Heroes Day ([[Philippines]])\n* [[August 24]] {{ndash}} Independence Day ([[Ukraine]])\n* [[August 25]] {{ndash}} Independence Day ([[Uruguay]])\n* [[August 26]] {{ndash}} Heroes' Day ([[Namibia]])\n* [[August 27]] {{ndash}} Independence Day ([[Moldova]])\n* [[August 28]] {{ndash}} Assumption of Mary (Eastern [[Christianity]])\n* [[August 29]] {{ndash}} National Uprising Day ([[Slovakia]])\n* [[August 30]] {{ndash}} Constitution Day ([[Kazakhstan]])\n* [[August 30]] {{ndash}} Republic Day ([[Tatarstan]])\n* [[August 30]] {{ndash}} Victory Day ([[Turkey]])\n* [[August 31]] {{ndash}} Independence Day ([[Kyrgyzstan]])\n* [[August 31]] {{ndash}} Independence Day ([[Malaysia]])\n* [[August 31]] {{ndash}} Independence Day ([[Trinidad and Tobago]])\n\n=== Moveable and Monthlong events ===\n[[File:PipesAndDrums.jpg|thumb|150px|right|Military Tattoo at [[Edinburgh Castle]].]]\n* [[Edinburgh]] Festival, including the Military Tattoo at [[Edinburgh Castle]], takes place through most of August and beginning of [[September]].\n* [[UK]] Bank Holidays: First [[Monday]] in [[Scotland]], last Monday in [[England]] and [[Wales]]\n* National Eisteddfod, cultural celebration in [[Wales]]: First week in August\n* Children's Day in [[Uruguay]]: Second Sunday in August\n* [[Monday]] after [[August 17]]: Holiday in [[Argentina]], commemorating [[Jos\u00e9 de San Mart\u00edn|Jos\u00e9 de San Martin]]\n* Discovery Day in [[Canada]]: third [[Monday]] in August\n* [[Summer Olympics]], often held in [[July]] and/or August\n\n== Selection of Historical Events ==\n[[File:Bundesbrief2.jpg|thumb|150px|right|Foundation Document of [[Switzerland]]]]\n[[File:Indonesia declaration of independence 17 August 1945.jpg|thumb|150px|right|[[Sukarno]] declaring [[Indonesia]] independent.]]\n[[File:Krakatoa eruption lithograph.jpg|thumb|150px|right|[[Krakatoa]] exploded on [[August 27]], [[1883]].]]\n[[File:Martin Luther King - March on Washington.jpg|thumb|150px|right|[[Martin Luther King, Jr.]] at the March on Washington on [[August 28]], [[1963]].]]\n* [[August 1]] {{ndash}} [[1291]]: Traditional founding date of [[Switzerland]].\n* [[August 1]] {{ndash}} [[1914]]: [[World War I]] begins.\n* [[August 1]] {{ndash}} [[1944]]: [[Anne Frank]] makes the last entry in her diary.\n* [[August 1]] {{ndash}} [[1960]]: [[Dahomey]] (now called [[Benin]]) becomes independent.\n* [[August 2]] {{ndash}} [[1990]]: [[Iraq]] invades [[Kuwait]].\n* [[August 3]] {{ndash}} [[1492]]: [[Christopher Columbus]] sets sail on his first voyage.\n* [[August 3]] {{ndash}} [[1960]]: [[Niger]] becomes independent.\n* [[August 4]] {{ndash}} [[1944]]: [[Anne Frank]] and her family are captured by the [[Gestapo]] in [[Amsterdam]].\n* [[August 4]] {{ndash}} [[1984]]: [[Upper Volta]]'s name is changed to [[Burkina Faso]].\n* [[August 5]] {{ndash}} [[1960]]: [[Upper Volta]] becomes independent.\n* [[August 5]] {{ndash}} [[1962]]: Film actress [[Marilyn Monroe]] is found dead at her home.\n* [[August 6]] {{ndash}} [[1825]]: [[Bolivia]]n independence.\n* [[August 6]] {{ndash}} [[1945]]: The Atomic Bomb is dropped on [[Hiroshima]].\n* [[August 6]] {{ndash}} [[1962]]: [[Jamaica]] becomes independent.\n* [[August 7]] {{ndash}} [[1960]]: [[Ivory Coast]] becomes independent.\n* [[August 9]] {{ndash}} [[1945]]: The Atomic Bomb is dropped on [[Nagasaki]].\n* [[August 9]] {{ndash}} [[1965]]: [[Singapore]] becomes independent.\n* [[August 9]] {{ndash}} [[1974]]: US President [[Richard Nixon]] resigns following the [[Watergate scandal]], with [[Gerald Ford]] replacing him.\n* [[August 10]] {{ndash}} [[1792]]: Storming of the Tuileries Palace during the [[French Revolution]]\n* [[August 10]] {{ndash}} [[1809]]: Beginning of [[Ecuador]]ean independence movement.\n* [[August 11]] {{ndash}} [[1960]]: [[Chad]] becomes independent.\n* [[August 13]] {{ndash}} [[1960]]: The [[Central African Republic]] becomes independent.\n* [[August 13]] {{ndash}} [[1961]]: Building of the [[Berlin Wall]] begins.\n* [[August 14]] {{ndash}} [[1945]]: [[Japan]] announces its surrender at the end of [[World War II]].\n* [[August 14]]/[[August 15|15]] {{ndash}} [[1947]]: [[India]] is partitioned at independence from the UK, as the new mainly [[Islam]]ic state of [[Pakistan]] is created.\n* [[August 15]] {{ndash}} [[1960]]: The [[Republic of the Congo]] becomes independent.\n* [[August 15]] {{ndash}} [[1971]]: [[Bahrain]] becomes independent.\n* [[August 16]] {{ndash}} [[1977]]: [[Elvis Presley]] dies aged 42, leading to a worldwide outpouring of grief.\n* [[August 17]] {{ndash}} [[1945]]: [[Indonesia]] declares independence from the [[Netherlands]].\n* [[August 17]] {{ndash}} [[1960]]: [[Gabon]] becomes independent.\n* [[August 17]] {{ndash}} [[1962]]: [[Peter Fechter]] becomes the first person to be shot dead at the [[Berlin Wall]].\n* [[August 19]] {{ndash}} [[43 BC]]: [[Augustus]] becomes [[Roman consul]].\n* [[August 19]] {{ndash}} [[14]]: [[Augustus]] dies.\n* [[August 19]] {{ndash}} [[1919]]: [[Afghanistan]] becomes independent.\n* [[August 19]] {{ndash}} [[1991]]: The August Coup against [[Mikhail Gorbachev]], in the [[Soviet Union]], begins.\n* [[August 20]] {{ndash}} [[1940]]: [[Leon Trotsky]] is fatally wounded with an ice pick in [[Mexico]].\n* [[August 20]] {{ndash}} [[1968]]: The [[Prague]] Spring uprising is crushed.\n* [[August 20]] {{ndash}} [[1991]]: [[Estonia]] regains its independence from the [[Soviet Union]].\n* [[August 21]] {{ndash}} [[1959]]: [[Hawaii]] becomes the 50th State of the [[US]].\n* [[August 24]] {{ndash}} [[79]]: [[Vesuvius]] erupts, destroying [[Pompeii]] and neighbouring [[Herculaneum]].\n* [[August 24]] {{ndash}} [[1991]]: [[Ukraine]] regains independence from the [[Soviet Union]].\n* [[August 24]] {{ndash}} [[2006]]: [[Pluto]] is demoted to a [[dwarf planet]].\n* [[August 25]] {{ndash}} [[1825]]: [[Uruguay]] declares independence from [[Brazil]].\n* [[August 25]] {{ndash}} [[1989]]: [[Voyager 2]] flies by the planet [[Neptune]].\n* [[August 27]] {{ndash}} [[1883]]: [[Krakatoa]], in the Sunda Strait between [[Sumatra]] and [[Java]], explodes, after a very violent eruption.\n* [[August 27]] {{ndash}} [[1991]]: [[Moldova]] becomes independent from the [[Soviet Union]].\n* [[August 28]] {{ndash}} [[1963]]: The [[March on Washington for Jobs and Freedom]] takes place, where [[Martin Luther King, Jr.]] makes his \"I Have a Dream\" speech for Civil Rights in the [[United States]].\n* [[August 29]] {{ndash}} [[2005]]: [[Hurricane Katrina]] wreaks devastation in [[Alabama]], [[Mississippi]] and [[Louisiana]]. [[New Orleans]] is flooded.\n* [[August 31]] {{ndash}} [[1957]]: [[Malaysia]], then the [[Malaysia|Federation of Malaya]], becomes independent.\n* [[August 31]] {{ndash}} [[1962]]: [[Trinidad and Tobago]] becomes independent.\n* [[August 31]] {{ndash}} [[1991]]: [[Kyrgyzstan]] becomes independent.\n* [[August 31]] {{ndash}} [[1997]]: [[Diana, Princess of Wales]] is killed in a car crash in [[Paris]], leading to a big outpouring of grief.\n\n== Trivia ==\n* Along with [[July]], August is one of two calendar months to be named after people who really lived (July was named for [[Julius Caesar]] and August was named for [[Augustus]]).\n* Only one [[President of the United States|US President]] has died in August, [[Warren G. Harding]], on [[August 2]], [[1923]].\n* August's flower is the [[Gladiolus]] with the birthstone being [[peridot]].\n* [[August 1]] is the only day in August during a [[common year]] to start within the seventh twelfth of the [[calendar]] year.\n* The astrological signs for August are Leo ([[July 22]] - [[August 21]]) and Virgo ([[August 22]] - [[September 21]]).\n*August is the second of two months beginning with 'A', the other being April, with both April 21 and August 21 falling either side of the Northern summer solstice.\n== References ==\n{{reflist}}\n\n{{Months|nocat=1}}\n[[Category:Months|*08]]",
    "Art": "[[File:Pierre-Auguste_Renoir,_Le_Moulin_de_la_Galette.jpg|thumb|300x300px|A painting by [[Renoir]] is a work of art.]]\n\n'''Art''' is a creative activity. It produces a product, an object. '''Art''' is a diverse range of human activities in creating visual, performing subjects, and expressing the author's thoughts. The product of art is called a '''work of art''', for others to experience.<ref>Various definitions in: Wilson, Simon & Lack, Jennifer 2008. ''The Tate guide to modern art terms''. Tate Publishing. ISBN 978-1-85437-750-0</ref><ref>E.H. Gombrich 1995. ''The story of art''. London: Phaidon. ISBN 978-0714832470</ref><ref>Kleiner, Gardner, Mamiya and Tansey. 2004. ''Art through the ages''. 12th ed. 2 volumes, Wadsworth. ISBN 0-534-64095-8 (vol 1) and ISBN 0-534-64091-5 (vol 2)</ref>\n\nSome art is useful in a practical sense, such as a sculptured clay [[bowl]] that can be used. That kind of art is sometimes called a ''[[craft]]''.\n\nThose who make art are called [[artist]]s. They hope to affect the [[emotion]]s of people who experience it. Some people find art relaxing, exciting or informative. Some say people are driven to make art due to their inner [[creativity]].\n\n\"[[The arts]]\" is a much broader term. It includes [[drawing]], [[painting]], [[sculpting]], [[photography]], [[Performing arts|performance art]], [[dance]], [[music]], [[poetry]], [[prose]] and [[theatre]].\n\n== Types of art ==\n[[File:Est\u00e1tuas_de_Botero_em_frente_ao_Pal\u00e1cio_de_Cultura_(Botero's_in_front_of_Culture_Palace).jpg|thumb|Statues{{Dead link|date=September 2021 |bot=InternetArchiveBot |fix-attempted=yes }} made by Botero, in front of the Culture Palace in Medellin, Colombia]]\n[[File:Chicago2_(MdB).jpg|thumb|A scene from the Musical Chicago, performed at a theatre, in [[Brno]].]]\n[[File:Nude_recumbent_woman_by_Jean-Christophe_Destailleur.jpg|thumb|247x247px|Nude Recumbent Woman, is a photograph by Jean-Christophe Destailleur. It was a featured image on Commons and is also a work of art.]]\nArt is divided into the [[plastic arts]], where something is made, and the [[performing arts]], where something is done by humans in action. The other division is between pure arts, done for themselves, and practical arts, done for a practical purpose, but with artistic content.\n\n* Plastic art\n** Fine art is expression by making something [[Beauty|beautiful]] or appealing to the [[emotion]]s by visual means: [[drawing]], [[painting]], [[printmaking]], [[sculpture]]\n** Literature: [[poetry]], creative [[writing]]\n* Performing art\n** Performing arts are expression using the body: [[drama]], [[dance]], [[acting]], [[singing]]\n** Auditory art (expression by making [[sound]]s): [[music]], [[singing]]\n* Practical art\n** Culinary art (expression by making [[flavor]]s and [[taste]]s): [[cooking]]\n** The practical arts (expression by making things and structures: [[architecture]], [[Movie|filming]], [[fashion]], [[photography]], [[video games]])\n\n== What \"art\" means ==\nSome people say that art is a product or item that is made with the intention of stimulating the human senses as well as the [[Mind|human mind]], [[spirit]] and [[soul]].  An artwork is normally judged by how much impact it has on people, the number of people who can relate to it, and how much they appreciate it. Some people also get inspired.\n\nThe first and broadest sense of \"art\" means \"arrangement\" or \"to arrange.\" In this sense, art is created when someone arranges things found in the world into a new or different design or form; or when someone arranges colors next to each other in a painting to make an image or just to make a pretty or interesting design.\n\nArt may express [[emotion]].  Artists may feel a certain emotion and wish to express it by creating something that means something to them.  Most of the art created in this case is made for the artist rather than an [[audience]].  However, if an audience is able to connect with the emotion as well, then the art work may become publicly successful.\n\n== History of art ==\nThere are sculptures, [[cave painting]] and [[rock art]] dating from the [[Upper Palaeolithic|Upper Paleolithic]] era.\n\nAll of the great ancient civilizations, such as [[Ancient Egypt]], [[Ancient India|India]], [[Ancient China|China]], [[Ancient Greece|Greece]], [[Ancient Rome|Rome]] and [[Persia]] had works and styles of art. In the [[Middle Ages]], most of the art in [[Europe]] showed people from the [[Bible]] in [[painting]]s, [[stained glass|stained-glass]] windows, and [[mosaic]] tile floors and walls.\n\n[[Islamic]] art includes [[geometric]] patterns, Islamic [[calligraphy]], and [[architecture]]. In [[India]] and [[Tibet]], painted sculptures, dance, and religious painting were done. In China, arts included [[jade]] carving, bronze, [[pottery]], [[poetry]], calligraphy, music, painting, drama, and fiction. There are many Chinese artistic styles, which are usually named after the ruling dynasty.\n\nIn Europe, after the [[Middle Ages]], there was a \"[[Renaissance]]\" which means \"rebirth\". People rediscovered [[science]] and artists were allowed to paint subjects other than religious subjects. People like [[Michelangelo]] and [[Leonardo da Vinci]] still painted religious pictures, but they also now could paint mythological pictures too. These artists also invented [[Perspective (graphical)|perspective]] where things in the distance look smaller in the picture. This was new because in the Middle Ages people would paint all the figures close up and just overlapping each other. These artists used [[nudity]] regularly in their art.\n\nIn the late 1800s, artists in Europe, responding to [[Industrialization|Modernity]] created many new painting styles such as [[Classicism]], [[Romanticism]], [[Realism]], and [[Impressionism]]. The history of twentieth century art includes [[Expressionism]], [[Fauvism]], [[Cubism]], [[Dada]]ism, [[Surrealism]], and [[Minimalism]].\n\n== Roles of art ==\nIn some [[Society|societies]], people think that art belongs to the person who made it. They think that the artist put his or her \"[[talent]]\" and industry into the art. In this view, the art is the [[property]] of the artist, protected by [[copyright]].\n\nIn other societies, people think that art belongs to no one. They think that society has put its [[social capital]] into the artist and the artist's work. In this view, society is a [[collective]] that has made the art, through the artist.\n\n=== Functions of art ===\nThe [[wikt:simple:function|functions]] of art include:<ref>{{cite book|title=Culturology|last=Bagdasaryan|first=Nadejda|year=2000|isbn=5-06-003475-5|pages=511|language=Russian|chapter=7. Art as a phenomenon of culture}}</ref>\n\n1) Cognitive function\n\n: Works of art let us know about what the author knew, and about what the surroundings of the author were like.\n\n2) Aesthetic function\n\n: Works of art can make people happy by being beautiful.\n\n3) Prognostic function\n\n: Some artists draw what they see the future like, and some of them are right, but most are not...\n\n4) Recreation function\n\n: Art makes us think about it, not about reality; we have a rest.\n\n5) Value function\n\n: What did the artist value? What aims did they like/dislike in human activity? This usually is clearly seen in artists' works.\n\n6) Didactic function\n\n: What message, criticism or political change did the artist wish to achieve?\n\n== Related pages ==\n* [[Art history]]\n* [[Modern art]]\n* [[Abstract art]]\n* [[Magnum opus]]\n* [[Painting]]\n* [[Sculpture]]\n* [[Street art]]\n\n== References ==\n{{commonscat}}\n<references />\n\n[[Category:Art| ]]\n[[Category:Non-verbal communication]]\n[[Category:Basic English 850 words]]""#;

        // Now try and strip stuff from this string
        let locations = mark_bracket_locations(raw_string);
        // Here we have to make sure that the locations are valid

        for location in locations {
            assert!(location.is_valid());
            dbg!(&location);
            dbg!(location.slice(raw_string));
            dbg!(location.inner(raw_string));
        }

        dbg!(clean_article_text(raw_string));
    }

    #[test]
    fn test_more_cases() {
        let raw_string = r#"\n[[File:Sauerstoffgehalt-1000mj2.png|thumb|Oxygen content of the atmosphere over the last billion years<ref></ref><ref>[http://www.nap.edu/openbook/0309100615/gifmid/30.gif http://www.nap.edu/openbook/0309100615/gifmid/30.gif]</ref>]]\n"#;

        dbg!(clean_article_text(raw_string));
    }

    #[test]
    fn test_art() {
        let raw_string = r#"== Types of art ==
        [[File:Estátuas_de_Botero_em_frente_ao_Palácio_de_Cultura_(Botero's_in_front_of_Culture_Palace).jpg|thumb|Statues{{Dead link|date=September 2021 |bot=InternetArchiveBot |fix-attempted=yes }} made by Botero, in front of the Culture Palace in Medellin, Colombia]]
        [[File:Chicago2_(MdB).jpg|thumb|A scene from the Musical Chicago, performed at a theatre, in [[Brno]].]]
        [[File:Nude_recumbent_woman_by_Jean-Christophe_Destailleur.jpg|thumb|247x247px|Nude Recumbent Woman, is a photograph by Jean-Christophe Destailleur. It was a featured image on Commons and is also a work of art.]]
        Art is divided into the [[plastic arts]], where something is made, and the [[performing arts]], where something is done by humans in action. The other division is between pure arts, done for themselves, and practical arts, done for a practical purpose, but with artistic content."#;

        let locations = mark_bracket_locations(raw_string);

        dbg!(&locations);

        for location in locations.iter() {
            println!("{}", location.slice(raw_string));
        }

        dbg!(clean_article_text(raw_string));

        let sanitized = sanitize_locations(locations);

        for location in sanitized.iter() {
            println!("{}", location.slice(raw_string));
        }
    }

    #[test]
    fn test_curly() {
        let raw_string = r#"This is my {{curly test}}."#;
        let expected_string = "This is my .".to_string();
        assert_eq!(strip_double_brackets_curly(raw_string), expected_string);

        let raw_string = "{{erase me}}{{and me}}{{and me too!}}";
        let expected_string = "".to_string();
        assert_eq!(strip_double_brackets_curly(raw_string), expected_string);

        let raw_string = "Before {{erase me}}middle{{and me}}{{and me too!}} and after";
        let expected_string = "Before middle and after".to_string();
        assert_eq!(strip_double_brackets_curly(raw_string), expected_string);
    }

    #[test]
    fn test_annotations() {
        let raw_string = r#"This is my [[annotation test]]."#;
        let expected_string = "This is my [[annotation test]].".to_string();
        assert_eq!(strip_annotations(raw_string), expected_string);

        let raw_string = "{{erase me}}{{and me}}{{and me too!}}";
        let expected_string = "{{erase me}}{{and me}}{{and me too!}}".to_string();
        assert_eq!(strip_annotations(raw_string), expected_string);

        let raw_string = "Here is a [[Annotation:bitch]] yup";
        let expected_string = "Here is a  yup".to_string();
        assert_eq!(strip_annotations(raw_string), expected_string);
    }

    #[test]
    fn test_strip_double() {

        let raw_string = r#"This is my [[annotation test]]."#;
        let expected_string = "This is my annotation test.".to_string();
        assert_eq!(strip_double_brackets(raw_string), expected_string);

        let raw_string = "{{erase me}}{{and me}}{{and me too!}}";
        let expected_string = "erase meand meand me too!".to_string();
        assert_eq!(strip_double_brackets(raw_string), expected_string);

        let raw_string = "Here is a [[Annotation:bitch]] yup";
        let expected_string = "Here is a Annotation:bitch yup".to_string();
        assert_eq!(strip_double_brackets(raw_string), expected_string);
    }

    #[test]
    fn test_failing() {
        let raw_string = r#"[[File:The Star-Spangled Banner.JPG|thumb|300px|right|An 1814 copy of the Star-Spangled Banner]]
        [[File:Star Spangled Banner Flag on display at the Smithsonian's National Museum of History and Technology, around 1964.jpg|thumb|300px|right|The flag from the song.]]
        "'''The Star-Spangled Banner'''" is the [[national anthem]] of the [[United States of America]]. [[Francis Scott Key]] wrote the words to it in 1814, after seeing [[Britain|British]] ships attacking [[Fort McHenry]] in [[Baltimore, Maryland]] during the [[War of 1812]].

        The words are set to the music of a British drinking song called "[[To Anacreon in Heaven]]". The song has 4 [[stanza]]s, but only the first one is usually sung. <ref></ref> <ref></ref>

        == Lyrics ==
        Although the United States does not have an official language, [[American English|English]] is the most used language in everyday life; thus, the official lyrics are in English. However, through the years, "The Star-Spangled Banner" has been translated into other languages. These languages are spoken by [[Americans|people]] living in the United States, who trace their roots to other parts of the globe. These languages include [[Spanish language|Spanish]], [[German language|German]], [[Yiddish]], [[Czech language|Czech]], [[Polish language|Polish]], [[French language|French]], [[Italian language|Italian]], [[Japanese language|Japanese]], [[Korean language|Korean]], [[Chinese language|Chinese]], and [[Arabic]].

        It has also been translated into languages spoken by [[native Americans]], such as [[Navajo language|Navajo]]. A fairly well-known Navajo version called "Dah Naatʼaʼí Sǫʼ bił Sinil" was translated by singer and former [[beauty pageant]] titleholder [[Radmilla Cody]].

        === English original ===
        The full poem consists of four stanzas with a total of thirty-two lines. But usually, just the first stanza is sung and is the most well-known among Americans.

        <div style="padding-left:20px;"><poem>O say can you see, by the dawn's early light,
        What so proudly we hailed at the twilight's last gleaming,
        Whose broad stripes and bright stars through the perilous fight,
        O'er the ramparts we watched, were so gallantly streaming?
        And the rocket's red glare, the bombs bursting in air,
        Gave proof through the night that our flag was still there;
        O say does that star-spangled banner yet wave
        O'er the land of the free and the home of the brave?

        On the shore dimly seen through the mists of the deep,
        Where the foe's haughty host in dread silence reposes,
        What is that which the breeze, o'er the towering steep,
        As it fitfully blows, half conceals, half discloses?
        Now it catches the gleam of the morning's first beam,
        In full glory reflected now shines in the stream:
        'Tis the star-spangled banner, O long may it wave
        O'er the land of the free and the home of the brave.

        And where is that band who so vauntingly swore
        That the havoc of war and the battle's confusion,
        A home and a country, should leave us no more?
        Their blood has washed out their foul footsteps' pollution.
        No refuge could save the hireling and slave
        From the terror of flight, or the gloom of the grave:
        And the star-spangled banner in triumph doth wave,
        O'er the land of the free and the home of the brave.

        O thus be it ever, when freemen shall stand
        Between their loved homes and the war's desolation.
        Blest with vict'ry and peace, may the Heav'n rescued land
        Praise the Power that hath made and preserved us a nation!
        Then conquer we must, when our cause it is just,
        And this be our motto: 'In God is our trust.'
        And the star-spangled banner in triumph shall wave
        O'er the land of the free and the home of the brave!</poem></div>

        === Spanish version ===
        Three versions of "The Star-Spangled Banner" have been translated into the Spanish language. The first one was done by Francis Haffkine Snow for the [[United States Bureau of Education]].<ref name="loc">[https://loc.gov/item/ihas.100000007 La bandera de las estrellas]. G. Schirmer, New York, NY, 1919.<br>"Spanish translation by Francis Haffkine Snow. This version of the song was prepared by the U.S. Bureau of Education."</ref><ref>[https://enparranda.com/artista-himnos-de-paises/letra-himno-de-estados-unidos-%5Ben-espanol%5D Letra de Himno de estados unidos en español de Himnos De Países] . ''En Parranda''.</ref><ref>[https://www.buenastareas.com/ensayos/Himno-De-Usa/25045451.html himno de usa - Ensayos universitarios - 1718 Palabras]. ''Buenas Tareas''.</ref><ref>[https://www.taringa.net/+info/himno-de-estados-unidos-en-espanol_hxn4t Himno de estados unidos en español: Himno nacional - La Bandera de Estrellas]</ref>

        The second one was done by a [[Peruvian American]] musician named [[Clotilde Arias]], for a competition held by then-president [[Franklin D. Roosevelt]], as a part of his [[Good Neighbor policy]] in an effort to promote American ideals in [[Latin America]]. This musician was the winner of this contest and her Spanish version was accepted by the [[United States Department of State]] in 1946.<ref>[https://www.bbc.com/news/magazine-29215415 From star-spangled to estrellado: US Anthem translator celebrated] (2014-09-18). Sparrow, Thomas. ''BBC Mundo''.</ref><ref>[https://americanhistory.si.edu/documentsgallery/exhibitions/arias/8.html «The Star-Spangled Banner ~ Not Lost in Translation: The Life of Clotilde Arias | Albert H. Small Documents Gallery | Smithsonian NMAH».]</ref><ref>[https://web.archive.org/web/20150210234444/http://voxxi.com/2012/10/12/clotilde-arias-spanish-star-spangled/ Clotilde Arias honored for Spanish version of Star-Spangled Banner] (2012-10-12). Baumann, Susana. ''VOXXI News''.</ref>

        Another version of "The Star-Spangled Banner" in Spanish is a single by many recording artists and singer-songwriters. It is probably the most well-known version. This version is titled "Nuestro Himno" (meaning "Our Anthem"), written by [[Adam Kidron]] and [[Eduardo Reyes]].<ref>[https://usatoday30.usatoday.com/news/nation/2006-04-28-spanish-anthem_x.htm Spanish 'Banner' draws protest] (2006-04-28). ''USA Today''.</ref> Kidron started the whole idea because he wanted to show support for Hispanic immigrants in the U.S. In 2006, a [[2006 United States immigration reform protests|change]] to U.S. immigration policy ticked off many people in the United States. "Nuestro Himno" was created in response to this change. The song was released on April 28, 2006 for their album ''Somos Americanos'' (meaning "We are Americans"). Many artists including Andy Andy, Autoridad de la Sierra, [[Aventura (band)|Aventura]], [[Ivy Queen]], [[Wyclef Jean]], [[Kalimba (singer)|Kalimba]], Kany, LDA, N Klabe, Patrulla 81, [[Pitbull (rapper)|Pitbull]], Ponce Carlos, Rayito, [[Reik]], [[Frank Reyes]], [[Tony Sunshine]], [[Olga Tañón]], [[Gloria Trevi]], Voz a Voz and [[Yemọja]] were involved in the making of this song. It was recorded in many cities including [[New York City]], [[Miami]], [[Los Angeles]], [[San Juan]], [[Mexico City]], and [[Madrid]]. The first verse is based on the first verse of the version by Francis Haffkine Snow in 1909.<ref name="loc"/> Although it quickly gained popularity, there have been some people who disliked this idea. Such people included then-president [[George W. Bush]], who did not approve of foreigners changing the national anthem into a language other than English,<ref>[https://www.billboard.com/music/music-news/billboard-bits-nuestro-himno-cracker-marty-stuart-58648/ Billboard Bits: ‘Nuestro Himno,’ Cracker, Marty Stuart]. ''Billboard''.</ref><ref>[https://www.washingtontimes.com/national/20060505-122343-2183r.htm Bush tells immigrants to learn English] (2006-05-05). ''The Washington Times''.</ref><ref>[https://www.nytimes.com/2006/04/28/us/bush-says-anthem-should-be-in-english.html Bush Says Anthem Should Be in English] (2006-04-28). Holusha, John. ''The New York Times''.</ref> as well as by a relative of Francis Scott Key—the original author of the national anthem.<ref>[https://abcnews.go.com/WNT/story?id=1898460 Spanish 'Star Spangled Banner' -- Touting the American Dream or Offensive Rewrite?] (2006-04-28). Avila, Jim. ''ABC News''.</ref>



        === German version ===
        In 1861, a version of "The Star-Spangled Banner" was translated by German American poet and immigrant named [[Niclas Müller]].<ref>[https://ingeb.org/songs/thestars.html The Star-Spangled Banner / O say can you see]. ''Ingeb.org''.</ref>

        <div style="padding-left:20px;"><poem>O, sagt, könnt ihr seh'n bei der Dämmerung Schein,
        Was so stolz wir begrüßten in Abendroths Gluten?
        Dess Streiffen und Sterne, durch Kämpfender Reih'n,
        Auf dem Walle wir sahen so wenniglich fluten;
        Die Raketen am Ort und die Bomben vom Fort,
        Sie zeigten bei Nacht, daß die Flagge noch dort.
        O sagt, ob das Banner mit Sternen besäet
        Über'm Lande der Frei'n und der Tapfern noch weht?

        Am Strand, kaum geseh'n durch den Nebel jetzt noch,
        Wo des Feinds stolzer Haufen in Schweigsamkeit waltet;
        Was ist's, daß der Wind, auf dem Thurme so hoch,
        Wenn er günstig d'ran bläst, halb verdeckt, halb entfaltet?
        Und jetzt faßt es den Strahl, wie er fällt in das Thal,
        Und glanzet in Herrlichkeit jetzt auf dem Pfahl.
        O das ist das Banner mit Sternen besäet,
        Das noch über den Frei'n und den Tapferen weht!

        Und wo ist der Schwarm, der vermaß sich so sehr,
        Daß des Krieges Gewühl und Verwirrung der Schlachten,
        Kein Land, keine Heimath gewähre uns mehr?
        Ihr Blut hat verwischet ihr freventlich Trachten.
        Und umsonst hat gesucht sklav und Miethling die Flucht
        Beim Schrecken des Kampfs und der tödtlichen Wucht.
        Und siegreich das Banner mit Sternen besäet,
        Über'm Lande der Frei'n und der Tapfern noch weht!

        Und so soll es sein stets, wo Männer die Hand
        Sich reichen, entgegen des Aufruhrs Gewalten;
        Mit Frieden und Sieg mag gesegnet das Land
        Dann preisen die Macht, die uns einig erhalten;
        Denn der Sieg muß uns sein, wo die Sache so rein;
        Und das sei der Wahlspruch: "Auf Gott trau allein!"
        Und siegreich das Banner mit Sternen besäet
        Über'm Lande der Frei'n und der Tapfern noch weht!</poem></div>

        === French version ===
        A version of "The Star-Spangled Banner" was translated into French by a Cajun named David Émile Marcantel.<ref>[https://web.archive.org/web/20130517004403/http://www.musiqueacadienne.com/banniere.htm La Bannière Étoilée, l'hymne national américain (The Star Spangled Banner)] (Trad., P.D., French words David Émile Marcantel, Vocal arrangement Jeanette Aguillard). ''MusiqueAcadienne.com''.</ref>

        <div style="padding-left:20px;"><poem>O dites, voyez-vous
        Dans la lumière du jour
        Le drapeau qu'on saluait
        À la tombée de la nuit ?
        Dont les trois couleurs vives
        Pendant la dure bataille
        Au-dessus des remparts
        Inspiraient notre pays.
        Et l'éclair des fusées,
        Des bombes qui explosaient,
        Démontraient toute la nuit
        Que le drapeau demeurait.
        Est-ce que la bannière étoilée
        Continue toujours à flotter
        Au-dessus d'une nation brave,
        Terre de la liberté ?</poem></div>

        === Navajo version ===
        A Navajo version of "The Star-Spangled Banner" was performed by model and singer Radmilla Cody. It is titled "'''Dah Naatʼaʼí Sǫʼ bił Sinil'''" in the [[Navajo language]], under her first album ''Within the Four Directions''.<ref>[https://lyricstranslate.com/en/radmilla-cody-dah-naat%CA%BCa%CA%BC%C3%AD-s%C7%AB%CA%BC-bi%C5%82-sinil-lyrics.html Dah Naatʼaʼí Sǫʼ bił Sinil lyrics]</ref><ref>[https://nv.wikipedia.org/wiki/Dah_Naat%CA%BCa%CA%BC%C3%AD_S%C7%AB%CA%BC_bi%C5%82_Sinil Dah Naatʼaʼí Sǫʼ bił Sinil] — Navajo Wikipedia]</ref>

        <div style="padding-left:20px;"><poem>Yá shoo danółʼį́į́ʼ
        Hayoołkááł biyiʼdę́ę́ʼ
        Baa dahwiiʼniihgo átʼé
        Dah naatʼaʼí éí yéigo nihił nilíinii.

        Noodǫ́ǫ́z dóó bizǫʼ disxǫs
        Naabaahii yitaayá
        Bitsʼą́ą́ honiyéeʼgo deiníłʼį́
        Nihichʼįʼ ínidída ndi baa ííníidlį́

        Áh, hoolʼáágóó bineʼ neidą́
        Báhádzid dahólǫ́ǫ ndi
        Éí yeeʼ bee tʼáá sih hasin
        Tʼóó nihá dah siłtzoos ndi

        Tʼóó shį́į́ éí sǫʼ bił sinilgo
        Dah naatʼá, áh hoolʼáa doo
        Nihikéyah bikʼihígíí
        Kʼad hózhǫ́ náhásdlį́į́ʼ</poem></div>

        === Yiddish version ===
        A [[Yiddish]] version titled "'''Di Shtern-Batsirte Fon'''" was translated by a [[Jewish American]] poet named Dr. Avrom Aisen, on the hundredth anniversary of Scott Key's death. It was published in 1943 by the Educational Alliance located New York City.<ref>[https://museumoffamilyhistory.com/yw-ssb.htm The Star Spangled Banner IN YIDDISH Translated by Dr. Avrom Aisen (Asen). Courtesy of the Educational Alliance, New York, New York] (1943). ''The Museum of Family History'' website.</ref>

        === Samoan version ===
        [[Samoan language|Samoan]] is a language spoken in [[American Samoa]], the American part of the [[Pacific Ocean|Pacific]] [[island]] of [[Samoa]].

        :Aue! Se'i e vaai, le malama o ataata mai
        :Na sisi a'e ma le mimita, i le sesega mai o le vaveao
        :O ai e ona tosi ma fetu, o alu a'e i taimi vevesi tu
        :I luga o 'Olo mata'utia, ma loto toa tausa'afia
        :O roketi mumu fa'aafi, o pomu ma fana ma aloi afi
        :E fa'amaonia i le po atoa, le fu'a o lo'o tu maninoa
        :Aue! ia tumau le fe'ilafi mai, ma agiagia pea
        :I eleele o sa'olotoga, ma nofoaga o le au totoa.

        == Media ==




        == References ==


        == Other websites ==


        * [https://www.loc.gov/exhibits/treasures/trm065.html Library of Congress] article
        * [http://americanhistory.si.edu/ssb/6_thestory/6b_osay/fs6b.html National Museum of American History]  article
        * [http://www.mdoe.org/starspangban.html Maryland Online Encyclopedia] article
        * [http://www.atlascom.us/defender.htm British Attack on Ft. McHenry Launched from Bermuda]
        * [http://www.si.edu/resource/faq/nmah/starflag.htm Encyclopedia Smithsonian article on "The Star-Spangled Banner"]
        * [http://www.infoplease.com/spot/starmangledbanner.html "Star-Mangled Banner: A look at some controversial, and botched, renditions of our national anthem"]
        * [http://www.worldwideschool.org/library/books/hst/northamerican/TheStarSpangledBanner/Chap1.html "The Star-Spangled Banner" by John A. Carpenter]
        * [http://www.easybyte.org Easybyte]—free easy piano arrangement of "The Star-Spangled Banner / Anacreon in Heaven"
        * [http://www.citypages.com/databank/22/1074/article9676.asp "Stars and Stripes Forever"]  City Pages, July 4, 2001
        * [http://www.sptimes.com/News/012801/SuperBowl2001/The_toughest_2_minute.shtml "The Toughest 2 Minutes"]


        [[Category:1814]]
        [[Category:19th-century American songs]]
        [[Category:Songs about the United States]]
        [[Category:Symbols of the United States]]
        [[Category:1810s songs]]
        [[Category:North American anthems]]"#;

        let sans_curly = strip_double_brackets_curly(raw_string);

        let locations = mark_bracket_locations(&sans_curly);
        for loc in locations {
            dbg!(loc.slice(&sans_curly));
        }

        let failing_pos = 13097;
        let span = 100;

        // dbg!(raw_string.get(failing_pos - span..failing_pos + span));

        let sans_annotations = strip_annotations(&sans_curly);


    }

    #[test]
    fn test_strip_comments() {

        let sample = r#"<!-- ATTENTION CLOSING ADMINISTRATOR -->\n\n<!-- Please change it to either  or . The archival templates  and  are placed at the top and bottom respectively.-->\n\n=== Molecular mass ===\n: \u00b7 \nRathfelder ''has nominated this page for deletion for the reason:'' Not encyclopedic.  () 22:32, 31 January 2024 (UTC)\n\n''Please discuss this request below, but keep in mind that  and that there may be options other than \"keep\" or \"delete\", such as merging.''\n\n====Discussion====\n\n\n<!--Please add any discussion above this comment.-->\nThis request is due to close on"#;

        dbg!(strip_html_comments(sample));
        println!();
        dbg!(strip_newline(&strip_html_comments(sample)));
    }

}
