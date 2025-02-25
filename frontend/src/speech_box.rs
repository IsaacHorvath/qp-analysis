use yew::prelude::*;
use time::PrimitiveDateTime;
use time::macros::format_description;
use regex::RegexBuilder;
use log::info;

#[derive(Properties, PartialEq)]
pub struct SpeechBoxProps {
  pub start: PrimitiveDateTime,
  pub end: PrimitiveDateTime,
  pub text: String,
  pub word: String,
}

#[function_component(SpeechBox)]
pub fn speech_box(props: &SpeechBoxProps) -> Html {
    let date_format = format_description!("[weekday repr:long], [month repr:long] [day padding:none], [year]");
    let time_format = format_description!("[hour padding:none repr:12]:[minute] [period case:upper]");
    let date = props.start.date().format(date_format).unwrap();
    let time = props.start.time().format(time_format).unwrap() + " - " + &props.end.time().format(time_format).unwrap();
    
    let punc_match = r#"[\.\,\;\:\!\?\'\"”“’‘\(\)\[\]\{\}«»]*"#;
    let mut reg_pattern = String::from("(");
    reg_pattern.push_str(punc_match);
    reg_pattern.push_str(")(");
    for (i, c) in props.word.chars().enumerate() {
        if c == ' ' {
            reg_pattern.push_str("\\s");
        }
        else {
            reg_pattern.push(c);
        }
        if i == props.word.len() - 1 {
            reg_pattern.push_str(")(");
        }
        reg_pattern.push_str(punc_match);
    }
    reg_pattern.push(')');
    
    info!("{}", reg_pattern);
    
    let re = RegexBuilder::new(&reg_pattern)
        .case_insensitive(true)
        .build()
        .unwrap();
        
    let mut inner_html = String::from("<p style=\"color: #aaaaaa; text-align: justify; text-justify: inter-word\">");
    inner_html.push_str(&re.replace_all(&props.text, "$1<strong style=\"color: #ffeba9\">$2</strong>$3"));
    inner_html.push_str("</p>");
    
    html! {
        <div style="border: 2px solid #717171; border-radius: 15px; background-color: #121212; margin-top: 1em; margin-left: 5%; margin-right: 5%; padding-block: 10px; padding-inline: 7px; min-width: 200px; flex: 0 1 min-content; max-height: 20em">
            <div style="height: 100%; overflow-y: auto; padding-right: 10px">
                <div style="display: flex; flex-wrap: wrap; column-gap: 3em">
                    <h3 style="color: #fee17d; margin: 0;">{date}</h3>
                    <h3 style="color: #fee17d; margin: 0;">{time}</h3>
                </div>
                {Html::from_html_unchecked(AttrValue::from(inner_html))}
            </div>
        </div>
    }
}
