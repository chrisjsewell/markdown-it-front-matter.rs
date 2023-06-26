use once_cell::sync::Lazy;
use std::str;

use crate::{
    ctype::{isalnum, isalpha},
    utils::autolink_delim,
};

enum Protocol {
    Mailto,
    Xmpp,
    None,
}

/// Match an email address.
/// Return the link and the number of chars to skip.
pub fn match_email(contents: &[u8]) -> Option<(String, usize)> {
    let mut pos = 0;
    let mut protocol = Protocol::None;
    if contents.starts_with(b"mailto:") {
        protocol = Protocol::Mailto;
        pos += 7;
    } else if contents.starts_with(b"xmpp:") {
        protocol = Protocol::Xmpp;
        pos += 5;
    }

    let size = contents.len();

    while pos < size {
        let c = contents[pos];

        if isalnum(c) || EMAIL_OK_SET[c as usize] {
            pos += 1;
            continue;
        }

        if c == b'@' {
            break;
        }

        return None;
    }

    if pos == 0 {
        return None;
    }

    let mut link_end = pos + 1;
    let mut np = 0;
    let mut num_slash = 0;

    while link_end < size {
        let c = contents[link_end];

        if isalnum(c) {
            // empty
        } else if c == b'@' {
            if !matches!(protocol, Protocol::Xmpp) {
                return None;
            }
        } else if c == b'.' && link_end < size - 1 && isalnum(contents[link_end + 1]) {
            np += 1;
        } else if c == b'/' && matches!(protocol, Protocol::Xmpp) && num_slash == 0 {
            // xmpp allows a single `/` in the url
            num_slash += 1;
        } else if c != b'-' && c != b'_' {
            break;
        }

        link_end += 1;
    }

    if link_end < 2
        || np == 0
        || (!isalpha(contents[link_end - 1]) && contents[link_end - 1] != b'.')
    {
        return None;
    }

    link_end = autolink_delim(contents, link_end);
    if link_end == 0 {
        return None;
    }

    let mut url = match protocol {
        Protocol::Mailto => "".to_string(),
        Protocol::Xmpp => "".to_string(),
        Protocol::None => "mailto:".to_string(),
    };

    let text = str::from_utf8(&contents[..link_end]).unwrap();
    url.push_str(text);

    Some((url, text.chars().count()))
}

static EMAIL_OK_SET: Lazy<[bool; 256]> = Lazy::new(|| {
    let mut sc = [false; 256];
    for c in &[b'.', b'+', b'-', b'_'] {
        sc[*c as usize] = true;
    }
    sc
});