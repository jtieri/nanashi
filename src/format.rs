use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::ListItem;

use crate::model::ThreadPost;

pub(crate) fn format_default(str: &str) -> String {
    format!(" {}", str)
}

pub(crate) fn format_html(str: &str) -> String {
    htmlescape::decode_html(str).unwrap()
}

/// Decode HTML entities and strip tags, leaving plain text.
pub(crate) fn plain_text(html: &str) -> String {
    strip_tags(&format_html(html))
}

/// Remove HTML tags, keeping the text between them.
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

pub(crate) fn format_post_short(
    post: &ThreadPost,
    no: usize,
    len: usize,
    area: Rect,
) -> ListItem<'_> {
    format_post(post, format!("{}/{}", no, len), area, true)
}

pub(crate) fn format_post_full(post: &ThreadPost, no: usize, area: Rect) -> ListItem<'_> {
    format_post(post, format!("#{}", no), area, false)
}

const CUT_MSG: &str = "[...]";
const CUT_MSG_LEN: usize = CUT_MSG.len();

const LIMIT_SHORT: usize = 10;

fn format_post(post: &ThreadPost, no: String, area: Rect, short: bool) -> ListItem<'_> {
    let mut lines = vec![Line::from("")];
    let mut header: Vec<Span> = vec![];

    if !post.sub().is_empty() {
        header.push(Span::styled(
            format_default(&htmlescape::decode_html(post.sub()).unwrap()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    header.push(Span::raw(" "));
    header.push(Span::styled(
        format!(
            "{} {} No.{}",
            htmlescape::decode_html(post.name()).unwrap(),
            format_time(post.time()),
            post.no(),
        ),
        Style::default().add_modifier(Modifier::ITALIC | Modifier::UNDERLINED),
    ));

    header.push(Span::styled(
        format_default(&no),
        Style::default().fg(Color::Yellow),
    ));

    if post.sticky() == 1 {
        header.push(Span::styled(format_default("📌"), Style::default()));
    }

    if post.closed() == 1 {
        header.push(Span::styled(format_default("🔓"), Style::default()));
    }

    lines.push(Line::from(header));

    if post.filename().is_some() && post.ext().is_some() {
        lines.push(Line::from(Span::styled(
            format_default(&format!(
                "{}{}",
                post.filename().as_ref().unwrap(),
                post.ext().as_ref().unwrap()
            )),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    // Cap a post to the pane height. ratatui's List renders nothing for a
    // selected item taller than the viewport, so keep every item within it.
    let max_lines = (area.height as usize).saturating_sub(7).max(1);
    let line_limit = if short {
        LIMIT_SHORT.min(max_lines)
    } else {
        max_lines
    };
    let cut_com = format_post_contents(post.com(), calc_width(area) as usize, line_limit);
    for span in cut_com {
        lines.push(span);
    }

    if short {
        lines.push(Line::from(Span::styled(
            format_default(&format!("{} Replies", post.replies())),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::from(""));
    ListItem::new(Text::from(lines)).style(Style::default())
}

fn format_post_contents(string: &str, sub_len: usize, line_limit: usize) -> Vec<Line<'_>> {
    let string = htmlescape::decode_html(string).unwrap();
    let split = string.split("<br>");
    let lines: Vec<&str> = split.collect();

    let mut spans = Vec::with_capacity(sub_len * line_limit);
    let mut i = 0;

    'line_loop: for line in lines {
        let line = strip_tags(line);
        let line_type = LineType::from_line(&line);

        let mut iter = line.chars();
        let strlen = line.len();
        let mut pos = 0;

        if strlen == 0 {
            spans.push(Line::from(""));

            i += 1;

            if i >= line_limit {
                break;
            }
        }

        while pos < strlen {
            let len = iter
                .by_ref()
                .take(sub_len)
                .fold(0, |acc, ch| acc + ch.len_utf8());

            if i >= line_limit {
                spans.push(Line::from(vec![
                    Span::styled(format_default(cut_line(&line, pos, len)), line_type.style()),
                    Span::styled(CUT_MSG, Style::default().fg(Color::Magenta)),
                ]));
                break 'line_loop;
            }

            spans.push(Line::from(Span::styled(
                format_default(&line[pos..pos + len]),
                line_type.style(),
            )));

            pos += len;
            i += 1;
        }
    }

    spans
}

fn format_time(timestamp: u64) -> String {
    let st = UNIX_EPOCH + Duration::from_secs(timestamp);
    let datetime = DateTime::<Utc>::from(st);

    datetime.format("%m/%d/%y(%a)%H:%M:%S").to_string()
}

fn cut_line(line: &str, pos: usize, cur_len: usize) -> &str {
    let cut = if cur_len < CUT_MSG_LEN {
        cur_len
    } else {
        CUT_MSG_LEN
    };

    &line[pos..pos + cur_len - cut]
}

fn calc_width(area: Rect) -> u16 {
    const MIN_WIDTH: i16 = 10;
    const BORDERS: i16 = 4;

    let area_width = area.width as i16;
    let width = if area_width - BORDERS < MIN_WIDTH {
        MIN_WIDTH
    } else {
        area_width - BORDERS
    };

    width as u16
}

#[derive(Default)]
enum LineType {
    #[default]
    Text,
    Greentext,
    Reply,
}

impl LineType {
    fn from_line(line: &str) -> Self {
        let first = line.chars().next();
        let second = line.chars().nth(1);

        match (first, second) {
            (Some('>'), Some('>')) => Self::Reply,
            (Some('>'), _) => Self::Greentext,
            _ => Self::default(),
        }
    }

    fn style(&self) -> Style {
        match self {
            Self::Text => Style::default(),
            Self::Greentext => Style::default().fg(Color::Green),
            Self::Reply => Style::default().fg(Color::Yellow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(1617810439), "04/07/21(Wed)15:47:19");
        assert_eq!(format_time(1717810439), "06/08/24(Sat)01:33:59");
    }

    #[test]
    fn test_format_default() {
        assert_eq!(format_default("string"), " string");
    }

    #[test]
    fn test_strip_tags() {
        assert_eq!(strip_tags("<b>bold</b> text"), "bold text");
        assert_eq!(strip_tags("no tags"), "no tags");
        assert_eq!(strip_tags("<a href=\"x\">link</a>"), "link");
        assert_eq!(strip_tags(""), "");
    }

    #[test]
    fn test_format_post_contents() {
        const POST: &str = "Natus est Schubert Himmelpfortgrund in vico Alsergrund Vindobonae \
        die 31 Ianuarii 1797. Pater, Franciscus Theodorus Schubert, filius pagani Moraviani, \
        magister scholae paroechialis; mater, Elisabeth (Vietz), filia artificis claustrarii \
        Silesici fuit, quae ante nuptias ut ancilla in familia Vindobonensi laboraverat.";

        // untruncated post formatting
        assert_eq!(format_post_contents(POST, 100, 5), vec![
            Line::from(" Natus est Schubert Himmelpfortgrund in vico Alsergrund Vindobonae die 31 Ianuarii 1797. Pater, Franc"),
            Line::from(" iscus Theodorus Schubert, filius pagani Moraviani, magister scholae paroechialis; mater, Elisabeth ("),
            Line::from(" Vietz), filia artificis claustrarii Silesici fuit, quae ante nuptias ut ancilla in familia Vindobone"),
            Line::from(" nsi laboraverat."),
        ]);

        // truncated post formatting
        assert_eq!(
            format_post_contents(POST, 50, 2),
            vec![
                Line::from(" Natus est Schubert Himmelpfortgrund in vico Alserg"),
                Line::from(" rund Vindobonae die 31 Ianuarii 1797. Pater, Franc"),
                Line::from(vec![
                    Span::from(" iscus Theodorus Schubert, filius pagani Morav"),
                    Span::styled("[...]", Style::default().fg(Color::Magenta))
                ]),
            ]
        );
    }
}
