//! アクセントのルールを適用するためのやつ

pub(crate) fn apply_accent(yomi: String, accent_type: char, bouyomi: bool) -> String {
    match accent_type {
        _ if bouyomi => yomi,
        '1' => {
            let mut new_yomi = String::new();

            for (i, c) in yomi.char_indices().skip(1) {
                if "ぁぃぅぇぉゃゅょっー".contains(c) {
                    continue;
                }

                new_yomi.push_str(&yomi[0..i]);
                new_yomi.push_str("'");
                new_yomi.push_str(&yomi[i..]);
                break;
            }

            if new_yomi.is_empty() {
                yomi
            } else {
                new_yomi
            }
        }
        '2' => {
            let mut yomi = yomi;
            yomi.push('\'');
            yomi
        }
        '0' | '*' | _ => yomi,
    }
}
