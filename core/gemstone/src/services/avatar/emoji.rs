pub const WALLET_AVATAR_EMOJIS: [&str; 76] = [
    "💎",
    "🦄",
    "🚀",
    "❤️",
    "😍",
    "🔥",
    "💩",
    "😭",
    "🏆",
    "🏴‍☠️",
    "✅",
    "⚠️",
    "💰",
    "🎁",
    "🎈",
    "🌈",
    "⭐️",
    "👑",
    "💔",
    "🔒",
    "🏦",
    "🥷",
    "👨‍💻",
    "🛢",
    "🔑",
    "🛡",
    "📈",
    "📉",
    "💥",
    "👽",
    "🔮",
    "⚡️",
    "🌍",
    "⏳",
    "🤖",
    "🛰",
    "🐉",
    "🐙",
    "🦅",
    "👀",
    "💪",
    "🔷",
    "👻",
    "🌪",
    "🕶",
    "👾",
    "🕵️‍♂️",
    "⌛️",
    "✨",
    "🍀",
    "☠️",
    "💀",
    "🕸",
    "🕷",
    "🎰",
    "☄️",
    "🏔",
    "🏜",
    "🌊",
    "🎆",
    "🎖",
    "🔭",
    "⛽️",
    "🏭",
    "🌉",
    "🏰",
    "🔨",
    "🧰",
    "💼",
    "🏷",
    "♟",
    "⚓️",
    "🎡",
    "🎢",
    "🎃",
    "📦",
];

#[uniffi::export]
pub fn wallet_avatar_emojis() -> Vec<String> {
    WALLET_AVATAR_EMOJIS.iter().map(|emoji| emoji.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_avatar_list_leads_with_the_gem_and_has_no_duplicates() {
        let emojis = wallet_avatar_emojis();
        assert_eq!(emojis.first().map(String::as_str), Some("\u{1f48e}"));
        assert_eq!(emojis.len(), WALLET_AVATAR_EMOJIS.len());
        let unique: std::collections::HashSet<&String> = emojis.iter().collect();
        assert_eq!(unique.len(), emojis.len());
    }
}
