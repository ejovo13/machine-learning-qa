
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

// The very first thing we should do is rip through the entire text and find where our double bracket annotations begin and end


#[derive(Debug, Clone)]
enum BracketType {
    Square,
    Curly
}

#[derive(Debug, Clone)]
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
                let test_string: String = source_str[self.start_pos..].chars().skip(2).take(4).collect();
                test_string == *"File"
            }
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

    /// Return the contents of the sting inside of our brackets. Necessary to avoid a silly UTF-8 error that I made.
    fn inner(&self, source_str: &str) -> String {
        // We want to skip the first two characters, and drop the last two!
        let slice = &source_str[self.start_pos..self.end_pos];
        slice.chars().take(self.len(source_str) - 2).skip(2).collect()
    }

    fn next_index(&self, source_str: &str) -> usize {
        let remaining_slice = &source_str[self.end_pos..source_str.len()];
        let (next_idx, _) = remaining_slice.char_indices().next().unwrap();
        next_idx + self.end_pos + 1
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

    // for (idx, c) in article_text.graphemes(true).enumerate() {
    for (idx, c) in article_text.char_indices() {
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

/// Modify our locations to avoid some supreme bullshit
///
/// Consider this edge case:
///
/// [[some text {{FUCK YOU BITCH}} ]]
fn sanitize_locations(locations: Vec<BracketLocation>) -> Vec<BracketLocation> {

    let n = locations.len();

    if n > 1 {
        let mut collected = locations[1..n].iter()
            .zip(&locations[0..n-1])
            .filter_map(|(next, prev)| if next.start_pos > prev.end_pos {
                Some(next.clone())
            } else {
                None
            })
            .collect::<Vec<BracketLocation>>();
        collected.insert(0, locations[0].clone());
        collected
    } else {
        locations
    }
}

/// A function that strips file links from our article text
#[pyfunction]
fn strip_file_annotations(article_text: &str) -> String {

    let locations = sanitize_locations(mark_bracket_locations(article_text));

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

                    let begin = prev_location.next_index(article_text);
                    let end = next_location.start_pos;

                    if end < begin {
                        dbg!(prev_location);
                        dbg!(prev_location.slice(article_text));
                        dbg!(next_location);
                        dbg!(next_location.slice(article_text));
                        panic!("Something's wrong hmmm");
                    }

                    out_string.push_str(&article_text[prev_location.next_index(article_text)..next_location.start_pos]);
                } else {
                    // Let's push the string starting from this location to the start of the next location
                    out_string.push_str(&prev_location.inner(article_text));
                    out_string.push_str(&article_text[prev_location.next_index(article_text)..next_location.start_pos]);
                }
            }
        }

        // Finally handle the final location
        let final_location = locations.last().unwrap();

        if final_location.should_be_removed(article_text) {
            out_string.push_str(&article_text[final_location.next_index(article_text)..article_text.len()]);
        } else {
            out_string.push_str(&final_location.inner(article_text));
            out_string.push_str(&article_text[final_location.next_index(article_text)..article_text.len()]);
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
    fn test_next_pos() {
        let raw_string = r#"[[Test string]]"#;
        let locations = mark_bracket_locations(raw_string);
        let loc = &locations[0];
        dbg!(loc.next_index(raw_string));
        dbg!(loc.inner(raw_string));
        dbg!(loc.slice(raw_string));
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

    dbg!(strip_file_annotations(raw_string));

    }

}