use crate::Resolver;
use daachorse::{CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathKind {
    Relative,
    AbsoluteWin,
    AbsolutePosix,
    Internal,
    Normal,
}

static ABSOLUTE_WIN_PATTERN_LENGTH_TWO: [&str; 52] = [
    "a:", "b:", "c:", "d:", "e:", "f:", "g:", "h:", "i:", "j:", "k:", "l:", "m:", "n:", "o:", "p:",
    "q:", "r:", "s:", "t:", "u:", "v:", "w:", "x:", "y:", "z:", "A:", "B:", "C:", "D:", "E:", "F:",
    "G:", "H:", "I:", "J:", "K:", "L:", "M:", "N:", "O:", "P:", "Q:", "R:", "S:", "T:", "U:", "V:",
    "W:", "X:", "Y:", "Z:",
];

static ABSOLUTE_WIN_PATTERN_REST: [&str; 104] = [
    "a:\\", "b:\\", "c:\\", "d:\\", "e:\\", "f:\\", "g:\\", "h:\\", "i:\\", "j:\\", "k:\\", "l:\\",
    "m:\\", "n:\\", "o:\\", "p:\\", "q:\\", "r:\\", "s:\\", "t:\\", "u:\\", "v:\\", "w:\\", "x:\\",
    "y:\\", "z:\\", "A:\\", "B:\\", "C:\\", "D:\\", "E:\\", "F:\\", "G:\\", "H:\\", "I:\\", "J:\\",
    "K:\\", "L:\\", "M:\\", "N:\\", "O:\\", "P:\\", "Q:\\", "R:\\", "S:\\", "T:\\", "U:\\", "V:\\",
    "W:\\", "X:\\", "Y:\\", "Z:\\", "a:/", "b:/", "c:/", "d:/", "e:/", "f:/", "g:/", "h:/", "i:/",
    "j:/", "k:/", "l:/", "m:/", "n:/", "o:/", "p:/", "q:/", "r:/", "s:/", "t:/", "u:/", "v:/",
    "w:/", "x:/", "y:/", "z:/", "A:/", "B:/", "C:/", "D:/", "E:/", "F:/", "G:/", "H:/", "I:/",
    "J:/", "K:/", "L:/", "M:/", "N:/", "O:/", "P:/", "Q:/", "R:/", "S:/", "T:/", "U:/", "V:/",
    "W:/", "X:/", "Y:/", "Z:/",
];

static PMA: Lazy<CharwiseDoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    CharwiseDoubleArrayAhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(ABSOLUTE_WIN_PATTERN_REST)
        .unwrap()
});

impl Resolver {
    pub(crate) fn get_target_kind(target: &str) -> PathKind {
        if target.is_empty() {
            return PathKind::Relative;
        }

        let path_kind = if target.starts_with('#') {
            PathKind::Internal
        } else if target.starts_with('/') {
            PathKind::AbsolutePosix
        } else if target == "."
            || target.starts_with("./")
            || target.starts_with("../")
            || target == ".."
        {
            PathKind::Relative
        } else {
            if target.len() == 2 && ABSOLUTE_WIN_PATTERN_LENGTH_TWO.contains(&target) {
                return PathKind::AbsoluteWin;
            }
            let mut iter = PMA.leftmost_find_iter(target);
            if let Some(mat) = iter.next() {
                let match_pattern_len = ABSOLUTE_WIN_PATTERN_REST[mat.value()].len();
                if mat.start() == 0 && mat.end() - mat.start() == match_pattern_len {
                    return PathKind::AbsoluteWin;
                }
            }
            PathKind::Normal
        };
        path_kind
    }
}

#[test]
fn test_resolver() {
    assert!(matches!(Resolver::get_target_kind(""), PathKind::Relative));
    assert!(matches!(Resolver::get_target_kind("."), PathKind::Relative));
    assert!(matches!(
        Resolver::get_target_kind(".."),
        PathKind::Relative
    ));
    assert!(matches!(
        Resolver::get_target_kind("../a.js"),
        PathKind::Relative
    ));
    assert!(matches!(
        Resolver::get_target_kind("./a.js"),
        PathKind::Relative
    ));
    assert!(matches!(
        Resolver::get_target_kind("D:"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("C:path"),
        PathKind::Normal
    ));
    assert!(matches!(
        Resolver::get_target_kind("C:\\a"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("c:/a"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("cc:/a"),
        PathKind::Normal
    ));
    assert!(matches!(Resolver::get_target_kind("fs"), PathKind::Normal));
}
