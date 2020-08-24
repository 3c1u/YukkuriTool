use kana;
use mecab::Tagger;

#[cfg(test)]
mod test;
mod accent_rule;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WordState {
    Categorem,
    Adjunct,
    Other,
}

fn parse_feature(feature: &str) -> Option<Vec<String>> {
    use csv::{ReaderBuilder, StringRecord};

    let mut r = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(feature.as_bytes());

    for r in r.records() {
        let r: StringRecord = if let Ok(r) = r {
            r
        } else {
            continue;
        };

        let k: Vec<&str> = r.into_iter().collect();
        let mut k: Vec<String> = k.iter().map(|v| (*v).to_string()).collect();
        k.resize(30, Default::default());

        return Some(k);
    }

    None
}

fn is_readable(c: char) -> bool {
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんぁぃぅぇぉっゃゅょゎがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽ',、。？ー".contains(c)
}

// 品詞の分解に失敗したとき，とりあえずひらがなに変換できるモノは変換しておく
fn try_fallback_yomi(surface: &str) -> String {
    // 読めない記号は無視する
    match surface {
        "、" | "，" | "," => "、".into(),
        "。" | "．" | "." => "。".into(),
        "！" | "!" => "。".into(),
        "？" | "?" => "？".into(),
        "'" => "'".into(),
        "（" | "(" => "/か'っこ/".into(),
        "）" | ")" => "/か'っことじ/".into(),
        "「" | "」" => ",".into(),
        "ー" | "-" => "ー".into(),
        _ => {
            // 単純なパターンにマッチしないもの（品詞分解に失敗した単語，記号など）を
            // 処理するためのフォールバック関数を用います．
            
            let surface = kana::kata2hira(&surface.to_lowercase());
            surface.chars().filter(|v| is_readable(*v)).collect()
        }
    }
}

fn get_yomi(tagger: Tagger, message: &str, bouyomi: bool) -> String {
    let mut out = String::new();
    let mut t = tagger;

    let res = t
        .parse_to_node(kana::combine(message))
        .unwrap()
        .iter_next();

    let mut number_buf = String::new();
    let mut word_buf = WordState::Other;

    let mut sharp_mode = false;
    let mut sharp_buf: String = String::new();

    'outer: for n in res {
        let surface = &n.surface[..n.length as usize];

        if surface.is_empty() {
            continue;
        }

        if sharp_mode && surface != "#" {
            sharp_buf.push_str(surface);
            continue;
        }

        // 品詞に応じた処理を行う
        let feature: Vec<String> =
            parse_feature(&n.feature).unwrap_or_else(|| vec![String::new(); 30]);

        // 数字に対する特殊な処理
        if feature[1] == "数詞" && surface.chars().all(|c| "0123456789".contains(c)) {
            number_buf.push_str(surface);
            continue;
        } else if number_buf.is_empty() {
            // 数のバッファが空なら何もしない
        } else if surface == "." {
            number_buf.push_str(surface);
            continue;
        } else if feature[2] == "助数詞" {
            let yomi = &feature[6];
            let yomi = kana::kata2hira(yomi).replace("ァ", "ぁ");

            // 〜つの読みは正しく処理されないので、手動で生成する
            if yomi == "つ" {
                'l: loop {
                    out.push_str(match &*number_buf {
                        "1" | "１" => "ひとつ",
                        "2" | "２" => "ふたつ",
                        "3" | "３" => "みっつ",
                        "4" | "４" => "よっつ",
                        "5" | "５" => "いつつ",
                        "6" | "６" => "むっつ",
                        "7" | "７" => "ななつ",
                        "8" | "８" => "やっつ",
                        "9" | "９" => "ここのつ",
                        "108" | "１０８" => "ひゃくやっつ",
                        _ => {
                            break 'l;
                        }
                    });

                    number_buf.clear();
                    word_buf = WordState::Adjunct;

                    continue 'outer;
                }
            }

            out.push_str("<NUMK VAL=");
            out.push_str(&number_buf);
            out.push_str(" COUNTER=");
            out.push_str(&yomi);
            out.push_str(">");

            number_buf.clear();

            word_buf = WordState::Adjunct;
            continue;
        } else {
            // out.push_str(",");

            out.push_str("<NUMK VAL=");
            out.push_str(&number_buf);
            out.push_str(">");

            number_buf.clear();

            word_buf = WordState::Adjunct;
        }

        // 読みを抽出してひらがなに変換する。
        let yomi = &feature[9];
        let yomi = kana::kata2hira(yomi).replace("ァ", "ぁ");

        // アクセント分割用の記号を入力（暫定）
        let word_type = match &*feature[0] {
            "助動詞" | "助詞" => WordState::Adjunct,
            _ if yomi == "*" || yomi.is_empty() => WordState::Other,
            _ => WordState::Categorem,
        };

        let w_fill = match (word_buf, word_type) {
            (WordState::Other, _) | (_, WordState::Other) => "",
            (_, WordState::Adjunct) => {
                while out.ends_with('っ') {
                    out.pop();
                }

                "+"
            }
            (_, WordState::Categorem) => {
                while out.ends_with('っ') {
                    out.pop();
                }

                "/"
            }
        };

        out.push_str(w_fill);

        word_buf = word_type;

        // "#"に対する特殊処理（MeCabバイパス）
        if surface == "#" {
            out.push_str(&kana::kata2hira(&sharp_buf).replace("ァ", "ぁ"));

            sharp_mode = !sharp_mode;
            word_buf = WordState::Categorem; // 独立語

            sharp_buf.clear();

            continue;
        }

        // アクセント情報の取得
        let accent_type = if word_type == WordState::Adjunct && surface == "です" {
            '1'
        } else {
            feature[23].chars().nth(0).unwrap_or('*')
        };

        // アクセントの適用
        let yomi = accent_rule::apply_accent(yomi, accent_type, bouyomi);

        // 最後が'っ'で終わる場合は，+とか/が来て欲しくないので，
        // とりあえずOtherにする
        if yomi
            .trim_end_matches('\'')
            .ends_with(|c| "ぁぃぅぇぉゃゅょっー".contains(c))
        {
            word_buf = WordState::Other;
        }

        let fallback_str;

        out.push_str(if yomi != "*" && !yomi.is_empty() {
            &yomi //.trim_start_matches(|c| "ぁぃぅぇぉゃゅょっー".contains(c))
        } else {
            fallback_str = try_fallback_yomi(surface);
            &fallback_str
        });
    }

    // 数字のバッファを使い切らなかった場合は，バッファを使い切る
    if !number_buf.is_empty() {
        if !out.is_empty() {
            // out.push_str(",");
        }

        out.push_str("<NUMK VAL=");
        out.push_str(&number_buf);
        out.push_str(">");

        number_buf.clear();
    }

    out
}

pub fn get_aquestalk_yomi<T: AsRef<str>>(message: T, bouyomi: bool) -> String {
    let t = if cfg!(target_os = "macos") {
        Tagger::new("-O dump -r /dev/null -d ../unidic")
    } else {
        Tagger::new("-Odump -d ../unidic -r ../mecabrc")
    };
    let t = t.expect("failed to initialize tagger");

    get_yomi(t, message.as_ref(), bouyomi)
}
