use std::fmt::{Display, Formatter, Result as FormatResult};
use std::str::FromStr;

const ALL_GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890_-+*^|&=<>[]()|:/\\!?$%";
const ALPHA_GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHA_LOWER_GLYPHS: &str = "abcdefghijklmnopqrstuvwxyz";
const ALPHA_UPPER_GLYPHS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHANUMERIC_GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const BINARY_GLYPHS: &str = "01";
const DECIMAL_GLYPHS: &str = "0123456789";
const HEX_GLYPHS: &str = "0123456789abcdefABCDEF";
const HEX_LOWER_GLYPHS: &str = "0123456789abcdef";
const HEX_UPPER_GLYPHS: &str = "0123456789ABCDEF";
const MATH_GLYPHS: &str = "0123456789=+-±×÷><∑∫∞√";
const SET_GLYPHS: &str = "QZCRc×∆∩∪∅∖-⊂⊃⊄⊅⊆⊇⊈⊉⊊⊋⊌⟃⟄{}|";
const SPECIAL_GLYPHS: &str = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ";
const SYMBOL_GLYPHS: &str = "_-+*^|&=<>[]()|/\\;:!?$%#@";

pub enum GlyphSet {
    All,
    Alpha,
    AlphaLower,
    AlphaUpper,
    Alphanumeric,
    Binary,
    Decimal,
    Hex,
    HexLower,
    HexUpper,
    Math,
    Set,
    Special,
    Symbol,
}

impl FromStr for GlyphSet {
    type Err = String;

    fn from_str(s: &str) -> Result<GlyphSet, String> {
        match s.to_lowercase().as_str() {
            "all" => Ok(GlyphSet::All),
            "alpha" => Ok(GlyphSet::Alpha),
            "alpha_lower" => Ok(GlyphSet::AlphaLower),
            "alpha_upper" => Ok(GlyphSet::AlphaUpper),
            "alphanum" => Ok(GlyphSet::Alphanumeric),
            "binary" => Ok(GlyphSet::Binary),
            "decimal" => Ok(GlyphSet::Decimal),
            "hex" => Ok(GlyphSet::Hex),
            "hex_lower" => Ok(GlyphSet::HexLower),
            "hex_upper" => Ok(GlyphSet::HexUpper),
            "math" => Ok(GlyphSet::Math),
            "set" => Ok(GlyphSet::Set),
            "special" => Ok(GlyphSet::Special),
            "symbol" => Ok(GlyphSet::Symbol),
            _ => Err(format!("Unknown glyph set '{}'", s)),
        }
    }
}

impl GlyphSet {
    pub fn to_alphabet(&self) -> Vec<char> {
        let alpha: &str = match *self {
            GlyphSet::All => ALL_GLYPHS,
            GlyphSet::Alpha => ALPHA_GLYPHS,
            GlyphSet::AlphaLower => ALPHA_LOWER_GLYPHS,
            GlyphSet::AlphaUpper => ALPHA_UPPER_GLYPHS,
            GlyphSet::Alphanumeric => ALPHANUMERIC_GLYPHS,
            GlyphSet::Binary => BINARY_GLYPHS,
            GlyphSet::Decimal => DECIMAL_GLYPHS,
            GlyphSet::Hex => HEX_GLYPHS,
            GlyphSet::HexLower => HEX_LOWER_GLYPHS,
            GlyphSet::HexUpper => HEX_UPPER_GLYPHS,
            GlyphSet::Math => MATH_GLYPHS,
            GlyphSet::Set => SET_GLYPHS,
            GlyphSet::Special => SPECIAL_GLYPHS,
            GlyphSet::Symbol => SYMBOL_GLYPHS,
        };

        alpha.chars().collect()
    }

    fn name(&self) -> &str {
        match self {
            GlyphSet::All => "all",
            GlyphSet::Alpha => "alpha",
            GlyphSet::AlphaLower => "alpha_lower",
            GlyphSet::AlphaUpper => "alpha_upper",
            GlyphSet::Alphanumeric => "alphanum",
            GlyphSet::Binary => "binary",
            GlyphSet::Decimal => "decimal",
            GlyphSet::Hex => "hex",
            GlyphSet::HexLower => "hex_lower",
            GlyphSet::HexUpper => "hex_upper",
            GlyphSet::Math => "math",
            GlyphSet::Set => "set",
            GlyphSet::Special => "special",
            GlyphSet::Symbol => "symbol",
        }
    }
}

impl Display for GlyphSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        write!(f, "{}", self.name())
    }
}
