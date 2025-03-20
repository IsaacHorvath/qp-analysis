use yew::prelude::*;
use time::PrimitiveDateTime;
use time::macros::format_description;
use regex::RegexBuilder;
use crate::error_page;
//use log::info;

#[derive(Properties, PartialEq)]
pub struct SpeechBoxProps {
    pub name: String,
    pub start: PrimitiveDateTime,
    pub end: PrimitiveDateTime,
    pub link: String,
    pub text: String,
    pub word: String,
}

#[function_component(SpeechBox)]
pub fn speech_box(props: &SpeechBoxProps) -> Html {
    let date_format = format_description!("[weekday repr:long], [month repr:long] [day padding:none], [year]");
    let time_format = format_description!("[hour padding:none repr:12]:[minute] [period case:upper]");
    let Ok(date) = props.start.date().format(date_format) else { return error_page(); };
    let Ok(start) = props.start.time().format(time_format) else { return error_page(); };
    let Ok(end) = props.end.time().format(time_format) else { return error_page(); };
    let time = start + " - " + &end;
    
    let punc_match = r#"[\.\,\;\:\!\?\'\"”“’‘\(\)\[\]\{\}«»]*"#;
    let mut reg_pattern = String::from("(");
    reg_pattern.push_str(punc_match);
    reg_pattern.push_str(")(");
    for (i, c) in props.word.chars().enumerate() {
        if c == ' ' {
            reg_pattern.push_str("(?:$|\\s)");
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
    
    let Ok(re) = RegexBuilder::new(&reg_pattern)
        .case_insensitive(true)
        .build()
        else { return error_page() };
        
    let mut inner_html = String::from("<p style=\"color: #aaaaaa; text-align: justify; text-justify: inter-word\">");
    inner_html.push_str(&re.replace_all(&props.text, "$1<strong style=\"color: #f6d32d\">$2</strong>$3"));
    inner_html.push_str("</p>");
    
    html! {
        <div class="speech-box">
            <div class="speech-box-container">
                <div class="speech-box-heading">
                    <p>{props.name.clone()}</p>
                    <a target="_blank" href={props.link.clone()}>{date}</a>
                    <p>{time}</p>
                </div>
                {Html::from_html_unchecked(AttrValue::from(inner_html))}
            </div>
        </div>
    }
}
