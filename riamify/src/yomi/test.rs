use super::get_yomi;
use mecab::Tagger;

fn tagger() -> Tagger {
    Tagger::new("-O dump -r /dev/null -d ../../unidic").unwrap()
}

#[test]
fn generate_yomi_01() {
    assert_eq!("おざき/ゆか+って", get_yomi(tagger(), "尾崎由香って", true));
}

#[test]
fn generate_yomi_02() {
    assert_eq!("みゃー", get_yomi(tagger(), "みゃー", true));
}

#[test]
fn generate_yomi_03() {
    assert_eq!("し+て/みろ+って", get_yomi(tagger(), "してみろって", true));
}
