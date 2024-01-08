use html2md;
use regex::Regex;

pub fn remove_html_tags(html: String) -> String {
    Regex::new(r"<.*?>|<\/.*?>")
        .unwrap()
        .replace_all(&html.replace("<br>", "\n"), "")
        .to_string()
}

pub fn to_markdown(html: String) -> String {
    html2md::parse_html(&html)
}
