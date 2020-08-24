// Aho-Corasick法を用いた辞書と置換アルゴリズムの実装

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use std::path::Path;

#[derive(Clone)]
pub struct Dictionary {
    ac: Option<AhoCorasick>,
    from_patterns: Vec<String>,
    replace_patterns: Vec<String>,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pattern<'a> {
    pub priority: i64,
    pub from: &'a str,
    pub to: &'a str,
}

impl Default for Dictionary {
    fn default() -> Self {
        let mut dict = Dictionary {
            ac: None,
            from_patterns: vec![],
            replace_patterns: vec![],
        };

        dict.load_default();
        dict
    }
}

impl Dictionary {
    pub fn load_custom(&mut self, custom: &[Pattern]) {
        self.from_patterns.push("#".into());
        self.replace_patterns.push("# しゃ'ーぷ #".into());

        for p in custom {
            self.from_patterns.push(p.from.into());
            self.replace_patterns.push(format!("# {} #", p.to)); // 変換したくないモノはシャープで囲む
        }
    }

    pub fn load_default(&mut self) {
        let from = "１２３４５６７８９０ＡＢＣＤＥＦＧＨＩＪＫＬＭＮＯＰＱＲＳＴＵＶＷＸＹＺａｂｃｄｅｆｇｈｉｊｋｌｍｎｏｐｑｒｓｔｕｖｗｘｙｚ";
        let to = "1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        for (from, to) in from.chars().zip(to.chars()) {
            let mut from_str = String::new();
            let mut to_str = String::new();

            from_str.push(from);
            to_str.push(to);

            self.from_patterns.push(from_str);
            self.replace_patterns.push(to_str);
        }
    }

    pub fn build_ac(&mut self) {
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&self.from_patterns);

        self.ac = Some(ac);
    }

    pub fn try_replace<S: AsRef<str>>(&self, string: S) -> String {
        if let Some(ref ac) = self.ac {
            ac.replace_all(string.as_ref(), &*self.replace_patterns)
        } else {
            string.as_ref().into()
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let f = File::open(path)?;
        let mut f = BufReader::new(f);

        let mut buf = String::new();

        let mut pat_from: Vec<String> = vec![];
        let mut pat_to: Vec<String> = vec![];

        while let Ok(size) = f.read_line(&mut buf) {
            if size == 0 {
                break;
            }

            let mut buf_split = buf.split_whitespace();
            let from = if let Some(n) = buf_split.next() {
                n
            } else {
                continue;
            };
            let to = if let Some(n) = buf_split.next() {
                n
            } else {
                continue;
            };
            let pat_priority = if let Some(n) = buf_split.next() {
                n
            } else {
                "0"
            };

            let mut entry: Vec<Pattern> = to
                .split(',')
                .zip(
                    pat_priority
                        .split(',')
                        .map(str::parse)
                        .map(Result::unwrap_or_default),
                )
                .map(|(to, priority)| Pattern { from, to, priority })
                .collect();
            
            // 優先順位を満たすようにソート
            entry.sort_unstable();
            entry.reverse();

            for e in entry {
                pat_from.push(e.from.into());
                pat_to.push(e.to.into());
            }

            buf.clear();
        }

        self.from_patterns.extend(pat_from.into_iter());
        self.replace_patterns.extend(pat_to.into_iter());

        Ok(())
    }
}
