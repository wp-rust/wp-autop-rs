use fancy_regex::Regex;
use std::collections::HashMap;

fn html_split(input: &str) -> Vec<String> {
    let comments = String::from("!(?:-(?!->)[^\\-]*)*(?:-->)?");
    let cdata = String::from("!\\[CDATA\\[[^\\]]*(?:](?!]>)[^\\]]*)*?(?:]]>)?");
    let escaped = format!("(?=!--|!\\[CDATA\\[)(?:(?=!-){}|{})", comments, cdata);
    let regex_str = format!("(<(?:{}|[^>]*>?))", escaped);
    let re = Regex::new(&regex_str).unwrap();

    let mut parts = Vec::new();
    let mut last_end = 0;
    
    for mat in re.find_iter(input) {
        if let Ok(m) = mat {
            parts.push(input[last_end..m.start()].to_string());
            parts.push(m.as_str().to_string());
            last_end = m.end();
        }
    }
    
    if last_end < input.len() {
        parts.push(input[last_end..].to_string());
    }
    
    parts
}

fn replace_in_html_tags(haystack: &str, replace_pairs: &HashMap<&str, &str>) -> String {
    let mut text_arr = html_split(haystack);
    let mut changed = false;

    let mut i = 1;
    while i < text_arr.len() {
        for (needle, replacement) in replace_pairs {
            if text_arr[i].contains(needle) {
                // Emulate string replacement for each occurrence inside the tag
                text_arr[i] = text_arr[i].replace(needle, replacement);
                changed = true;
                break;
            }
        }
        i += 2;
    }

    if changed {
        text_arr.join("")
    } else {
        haystack.to_string()
    }
}

pub fn wp_autop(text: &str) -> String {
    wp_autop_br(text, true)
}

pub fn wp_autop_br(text: &str, br: bool) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    let mut text = text.to_string();
    text.push('\n');

    let mut pre_tags = Vec::new();
    if text.contains("<pre") {
        let text_parts: Vec<String> = text.split("</pre>").map(String::from).collect();
        let last_text = text_parts.last().unwrap().clone();
        text = String::new();

        for (i, text_part) in text_parts.iter().take(text_parts.len() - 1).enumerate() {
            if let Some(start) = text_part.find("<pre") {
                let name = format!("<pre wp-pre-tag-{}></pre>", i);
                pre_tags.push((name.clone(), format!("{}</pre>", &text_part[start..])));
                text.push_str(&text_part[..start]);
                text.push_str(&name);
            } else {
                text.push_str(text_part);
            }
        }
        text.push_str(&last_text);
    }

    let re_br = Regex::new(r"(?i)<br\s*\/?>\s*<br\s*\/?>").unwrap();
    text = re_br.replace_all(&text, "\n\n").to_string();

    let all_blocks = "(?:table|thead|tfoot|caption|col|colgroup|tbody|tr|td|th|div|dl|dd|dt|ul|ol|li|pre|form|map|area|blockquote|address|math|style|p|h[1-6]|hr|fieldset|legend|section|article|aside|hgroup|header|footer|nav|figure|figcaption|details|menu|summary)";

    let re_block_open = Regex::new(&format!(r"(?i)(<{}[\s/>])", all_blocks)).unwrap();
    text = re_block_open.replace_all(&text, "\n\n${1}").to_string();

    let re_block_close = Regex::new(&format!(r"(?i)(</{}>)", all_blocks)).unwrap();
    text = re_block_close.replace_all(&text, "${1}\n\n").to_string();

    text = text.replace("\r\n", "\n").replace('\r', "\n");

    let mut replacements = HashMap::new();
    replacements.insert("\n", " <!-- wpnl --> ");
    text = replace_in_html_tags(&text, &replacements);

    if text.contains("<option") {
        let re_option_open = Regex::new(r"(?i)\s*<option").unwrap();
        let re_option_close = Regex::new(r"(?i)</option>\s*").unwrap();
        text = re_option_open.replace_all(&text, "<option").to_string();
        text = re_option_close.replace_all(&text, "</option>").to_string();
    }

    if text.contains("</object>") {
        let re_obj_open = Regex::new(r"(?i)(<object[^>]*>)\s*").unwrap();
        let re_obj_close = Regex::new(r"(?i)\s*</object>").unwrap();
        let re_param_embed = Regex::new(r"(?i)\s*(</?(?:param|embed)[^>]*>)\s*").unwrap();
        text = re_obj_open.replace_all(&text, "${1}").to_string();
        text = re_obj_close.replace_all(&text, "</object>").to_string();
        text = re_param_embed.replace_all(&text, "${1}").to_string();
    }

    if text.contains("<source") || text.contains("<track") {
        let re_audio_video_open = Regex::new(r"(?i)([<\[](?:audio|video)[^>\]]*[>\]])\s*").unwrap();
        let re_audio_video_close = Regex::new(r"(?i)\s*([<\[]/(?:audio|video)[>\]])").unwrap();
        let re_source_track = Regex::new(r"(?i)\s*(<(?:source|track)[^>]*>)\s*").unwrap();
        text = re_audio_video_open.replace_all(&text, "${1}").to_string();
        text = re_audio_video_close.replace_all(&text, "${1}").to_string();
        text = re_source_track.replace_all(&text, "${1}").to_string();
    }

    if text.contains("<figcaption") {
        let re_figcap_open = Regex::new(r"(?i)\s*(<figcaption[^>]*>)").unwrap();
        let re_figcap_close = Regex::new(r"(?i)</figcaption>\s*").unwrap();
        text = re_figcap_open.replace_all(&text, "${1}").to_string();
        text = re_figcap_close.replace_all(&text, "</figcaption>").to_string();
    }

    let re_multi_newlines = Regex::new(r"\n\n+").unwrap();
    text = re_multi_newlines.replace_all(&text, "\n\n").to_string();

    let re_split = Regex::new(r"\n\s*\n").unwrap();
    let texts: Vec<String> = re_split.split(&text).filter_map(|s| s.ok()).filter(|s| !s.is_empty()).map(String::from).collect();

    text = String::new();
    let re_trim = Regex::new(r"^\n*|\n*$").unwrap();
    for text_piece in texts {
        text.push_str("<p>");
        text.push_str(&re_trim.replace_all(&text_piece, ""));
        text.push_str("</p>\n");
    }

    let re_empty_p = Regex::new(r"(?i)<p>\s*</p>").unwrap();
    text = re_empty_p.replace_all(&text, "").to_string();

    let re_p_wrap = Regex::new(r"(?i)<p>([^<]+)</(div|address|form)>").unwrap();
    text = re_p_wrap.replace_all(&text, "<p>${1}</p></${2}>").to_string();

    let re_unwrap_p = Regex::new(&format!(r"(?i)<p>\s*(</?{}[^>]*>)\s*</p>", all_blocks)).unwrap();
    text = re_unwrap_p.replace_all(&text, "${1}").to_string();

    let re_li_p = Regex::new(r"(?i)<p>(<li.+?)</p>").unwrap();
    text = re_li_p.replace_all(&text, "${1}").to_string();

    let re_blockq_p = Regex::new(r"(?i)<p><blockquote([^>]*)>").unwrap();
    text = re_blockq_p.replace_all(&text, "<blockquote${1}><p>").to_string();
    
    let re_p_blockq_close = Regex::new(r"(?i)</blockquote></p>").unwrap();
    text = re_p_blockq_close.replace_all(&text, "</p></blockquote>").to_string();

    let re_p_block_open = Regex::new(&format!(r"(?i)<p>\s*(</?{}[^>]*>)", all_blocks)).unwrap();
    text = re_p_block_open.replace_all(&text, "${1}").to_string();

    let re_block_close_p = Regex::new(&format!(r"(?i)(</?{}[^>]*>)\s*</p>", all_blocks)).unwrap();
    text = re_block_close_p.replace_all(&text, "${1}").to_string();

    if br {
        let re_script_style = Regex::new(r"(?is)<(script|style).*?</\1>").unwrap();
        text = re_script_style.replace_all(&text, |caps: &fancy_regex::Captures| {
            caps.get(0).unwrap().as_str().replace("\n", "<WPPreserveNewline />")
        }).to_string();

        let re_br_norm = Regex::new(r"(?i)<br>|<br\/>").unwrap();
        text = re_br_norm.replace_all(&text, "<br />").to_string();

        let re_br_insert = Regex::new(r"(?i)(<br />)?([ \t]*)\n").unwrap();
        text = re_br_insert.replace_all(&text, |caps: &fancy_regex::Captures| {
            if caps.get(1).is_some() {
                caps.get(0).unwrap().as_str().to_string()
            } else {
                format!("{}<br />\n", caps.get(2).unwrap().as_str())
            }
        }).to_string();

        text = text.replace("<WPPreserveNewline />", "\n");
    }

    let re_br_block_close = Regex::new(&format!(r"(?i)(</?{}[^>]*>)\s*<br />", all_blocks)).unwrap();
    text = re_br_block_close.replace_all(&text, "${1}").to_string();

    let re_br_subset = Regex::new(r"(?i)<br />(\s*</?(?:p|li|div|dl|dd|dt|th|pre|td|ul|ol)[^>]*>)").unwrap();
    text = re_br_subset.replace_all(&text, "${1}").to_string();

    let re_n_p = Regex::new(r"(?i)\n</p>$").unwrap();
    text = re_n_p.replace_all(&text, "</p>").to_string();

    for (name, original) in pre_tags {
        text = text.replace(&name, &original);
    }

    if text.contains("<!-- wpnl -->") {
        let re_wpnl = Regex::new(r"(?i)\s?<!-- wpnl -->\s?").unwrap();
        text = re_wpnl.replace_all(&text, "\n").to_string();
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wp_autop() {
        let input = "Hello world\n\nThis is a test.";
        let expected = "<p>Hello world</p>\n<p>This is a test.</p>\n";
        assert_eq!(wp_autop(input), expected);
    }
}
