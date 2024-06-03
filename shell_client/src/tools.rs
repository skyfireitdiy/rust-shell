/*!
 * 工具
 */

/// 判断一个字符串是否是另一个字符串的前缀
pub fn is_prefix(src: &str, prefix: &str) -> bool {
    src.len() >= prefix.len() && prefix == &src[..prefix.len()]
}

/// 判断一个字符串是否是另一个字符串的前缀，忽略大小写
pub fn is_prefix_nocase(src: &str, prefix: &str) -> bool {
    src.len() >= prefix.len() && prefix.to_uppercase() == src[0..prefix.len()].to_uppercase()
}

/// 判断一个字符串是否是另一个字符串的子串，忽略大小写
pub fn contain_nocase(src: &str, substr: &str) -> bool {
    src.len() >= substr.len()
        && src
            .to_uppercase()
            .find(&substr.to_uppercase())
            .map_or(false, |_| true)
}
