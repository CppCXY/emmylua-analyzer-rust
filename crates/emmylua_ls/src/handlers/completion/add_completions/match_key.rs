
pub fn match_keyword(key: &str, candidate_key: &str) -> bool {
    if key.is_empty() || candidate_key.is_empty() {
        return false; // 避免空字符串的情况
    }
    let key_first_char = key.chars().next().unwrap().to_lowercase().to_string(); // 获取 key 的首字符并转换为小写
    let words = split_to_words(candidate_key); // 将 candidate_key 分解为词组
    for word in words {
        if word.is_empty() {
            continue; // 忽略空词组，例如连续下划线可能产生空词组
        }
        let word_first_char = word.chars().next().unwrap().to_lowercase().to_string(); // 获取词组的首字符并转换为小写
        if word_first_char == key_first_char {
            return true; // 只要有一个词组的首字符匹配就返回 true
        }
    }
    false // 没有找到任何匹配的词组，返回 false
}

fn split_to_words(candidate_key: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::with_capacity(candidate_key.len());
    let mut last_char_is_ascii = true; // 初始假设上一个字符是 ASCII (英文)

    for char in candidate_key.chars() {
        let current_char_is_ascii = char.is_ascii_alphabetic(); // 判断当前字符是否是 ASCII 字母

        if char == '_' {
            if !current_word.is_empty() {
                words.push(current_word);
                current_word = String::with_capacity(candidate_key.len() / 2);
            }
            last_char_is_ascii = true; // 下划线后重置类型判断，假设之后是英文开头
        } else if char.is_uppercase()
            && !current_word.is_empty()
            && current_word.chars().last().unwrap().is_lowercase()
        {
            // 驼峰命名法分割
            words.push(current_word);
            current_word = String::with_capacity(candidate_key.len() / 2);
            current_word.push(char);
            last_char_is_ascii = current_char_is_ascii;
        } else if current_char_is_ascii != last_char_is_ascii && !current_word.is_empty() {
            // 英文与非英文边界
            words.push(current_word);
            current_word = String::with_capacity(candidate_key.len() / 2);
            current_word.push(char);
            last_char_is_ascii = current_char_is_ascii;
        } else {
            current_word.push(char);
            last_char_is_ascii = current_char_is_ascii;
        }
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_keyword_english() {
        assert_eq!(match_keyword("i", "if"), true);
        assert_eq!(match_keyword("i", "_if"), true);
        assert_eq!(match_keyword("i", "notIf"), true);
        assert_eq!(match_keyword("i", "this_if"), true);
        assert_eq!(match_keyword("I", "If"), true);
        assert_eq!(match_keyword("I", "if"), true);
        assert_eq!(match_keyword("i", "IF"), true);
        assert_eq!(match_keyword("n", "not"), true);
        assert_eq!(match_keyword("t", "this"), true);
        assert_eq!(match_keyword("f", "functionName"), true);
        assert_eq!(match_keyword("g", "_G"), true);
    }

    #[test]
    fn test_match_keyword_chinese() {
        assert_eq!(match_keyword("如", "如果"), true);
        assert_eq!(match_keyword("如", "_如果"), true);
        assert_eq!(match_keyword("如", "Not如果"), true);
        assert_eq!(match_keyword("如", "This_如果"), true);
        assert_eq!(match_keyword("R", "如果"), false);
        assert_eq!(match_keyword("r", "如果"), false);
        assert_eq!(match_keyword("如", "如果If"), true);
        assert_eq!(match_keyword("果", "水果"), false);
    }

    #[test]
    fn test_match_keyword_mixed() {
        assert_eq!(match_keyword("i", "如果If"), true);
        assert_eq!(match_keyword("r", "Not如果"), false);
        assert_eq!(match_keyword("t", "This_如果"), true);
        assert_eq!(match_keyword("n", "not如果"), true);
        assert_eq!(match_keyword("f", "Function如果"), true);
    }

    #[test]
    fn test_match_keyword_empty_input() {
        assert_eq!(match_keyword("", "if"), false);
        assert_eq!(match_keyword("i", ""), false);
        assert_eq!(match_keyword("", ""), false);
    }

    #[test]
    fn test_split_to_words_basic() {
        assert_eq!(split_to_words("if"), vec!["if"]);
        assert_eq!(split_to_words("_if"), vec!["if"]);
        assert_eq!(split_to_words("notIf"), vec!["not", "If"]);
        assert_eq!(split_to_words("this_if"), vec!["this", "if"]);
    }

    #[test]
    fn test_split_to_words_complex() {
        assert_eq!(
            split_to_words("startsWithUpperCase"),
            vec!["starts", "With", "Upper", "Case"]
        );
        assert_eq!(
            split_to_words("MixedCase_with_underscore"),
            vec!["Mixed", "Case", "with", "underscore"]
        );
        assert_eq!(
            split_to_words("___multiple___underscores___"),
            vec!["multiple", "underscores"]
        ); // 多个下划线
        assert_eq!(
            split_to_words("_leadingUnderscore"),
            vec!["leading", "Underscore"]
        ); // 前导下划线
        assert_eq!(
            split_to_words("trailingUnderscore_"),
            vec!["trailing", "Underscore"]
        ); // 后置下划线
    }

    #[test]
    fn test_split_to_words_chinese() {
        assert_eq!(split_to_words("如果"), vec!["如果"]);
        assert_eq!(split_to_words("_如果"), vec!["如果"]);
        assert_eq!(split_to_words("Not如果"), vec!["Not", "如果"]);
        assert_eq!(split_to_words("This_如果"), vec!["This", "如果"]);
        assert_eq!(
            split_to_words("混合Case_中文"),
            vec!["混合", "Case", "中文"]
        );
    }

    #[test]
    fn test_split_to_words_edge_cases() {
        assert_eq!(split_to_words(""), Vec::<String>::new());
        assert_eq!(split_to_words("_"), Vec::<String>::new());
        assert_eq!(split_to_words("__"), Vec::<String>::new());
        assert_eq!(split_to_words("If_"), vec!["If"]);
        assert_eq!(split_to_words("_If"), vec!["If"]);
    }
}
