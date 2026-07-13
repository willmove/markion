//! Emoji shortcode to Unicode mapping.
//!
//! Provides conversion from common emoji shortcodes (`:smile:`, `:heart:`, etc.)
//! to their corresponding Unicode emoji characters.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Global emoji shortcode to Unicode mapping.
static EMOJI_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Smileys & Emotion
    map.insert("smile", "😄");
    map.insert("smiley", "😃");
    map.insert("grin", "😁");
    map.insert("laughing", "😆");
    map.insert("sweat_smile", "😅");
    map.insert("joy", "😂");
    map.insert("rofl", "🤣");
    map.insert("wink", "😉");
    map.insert("blush", "😊");
    map.insert("innocent", "😇");
    map.insert("heart_eyes", "😍");
    map.insert("kissing_heart", "😘");
    map.insert("thinking", "🤔");
    map.insert("neutral_face", "😐");
    map.insert("expressionless", "😑");
    map.insert("no_mouth", "😶");
    map.insert("smirk", "😏");
    map.insert("unamused", "😒");
    map.insert("roll_eyes", "🙄");
    map.insert("grimacing", "😬");
    map.insert("lying_face", "🤥");
    map.insert("relieved", "😌");
    map.insert("pensive", "😔");
    map.insert("sleepy", "😪");
    map.insert("drooling_face", "🤤");
    map.insert("sleeping", "😴");
    map.insert("mask", "😷");
    map.insert("face_with_thermometer", "🤒");
    map.insert("face_with_head_bandage", "🤕");
    map.insert("nauseated_face", "🤢");
    map.insert("sneezing_face", "🤧");
    map.insert("dizzy_face", "😵");
    map.insert("star_struck", "🤩");
    map.insert("zany_face", "🤪");
    map.insert("shushing_face", "🤫");
    map.insert("face_with_symbols_on_mouth", "🤬");
    map.insert("exploding_head", "🤯");
    map.insert("flushed", "😳");
    map.insert("disappointed", "😞");
    map.insert("worried", "😟");
    map.insert("angry", "😠");
    map.insert("rage", "😡");
    map.insert("cry", "😢");
    map.insert("sob", "😭");
    map.insert("scream", "😱");
    map.insert("confounded", "😖");
    map.insert("persevere", "😣");
    map.insert("disappointed_relieved", "😥");
    map.insert("fearful", "😨");
    map.insert("cold_sweat", "😰");
    map.insert("hugs", "🤗");
    map.insert("sunglasses", "😎");
    map.insert("nerd_face", "🤓");
    map.insert("face_with_monocle", "🧐");
    map.insert("confused", "😕");
    map.insert("slightly_frowning_face", "🙁");
    map.insert("slightly_smiling_face", "🙂");
    map.insert("upside_down_face", "🙃");
    map.insert("stuck_out_tongue", "😛");
    map.insert("stuck_out_tongue_winking_eye", "😜");
    map.insert("stuck_out_tongue_closed_eyes", "😝");
    map.insert("money_mouth_face", "🤑");
    map.insert("zipper_mouth_face", "🤐");

    // Hearts
    map.insert("heart", "❤️");
    map.insert("orange_heart", "🧡");
    map.insert("yellow_heart", "💛");
    map.insert("green_heart", "💚");
    map.insert("blue_heart", "💙");
    map.insert("purple_heart", "💜");
    map.insert("black_heart", "🖤");
    map.insert("white_heart", "🤍");
    map.insert("brown_heart", "🤎");
    map.insert("broken_heart", "💔");
    map.insert("heart_exclamation", "❣️");
    map.insert("two_hearts", "💕");
    map.insert("revolving_hearts", "💞");
    map.insert("heartbeat", "💓");
    map.insert("heartpulse", "💗");
    map.insert("sparkling_heart", "💖");
    map.insert("cupid", "💘");
    map.insert("gift_heart", "💝");
    map.insert("heart_decoration", "💟");

    // Gestures & Body Parts
    map.insert("thumbsup", "👍");
    map.insert("thumbsdown", "👎");
    map.insert("ok_hand", "👌");
    map.insert("punch", "👊");
    map.insert("fist", "✊");
    map.insert("v", "✌️");
    map.insert("wave", "👋");
    map.insert("clap", "👏");
    map.insert("raised_hands", "🙌");
    map.insert("pray", "🙏");
    map.insert("handshake", "🤝");
    map.insert("muscle", "💪");

    // People & Fantasy
    map.insert("baby", "👶");
    map.insert("child", "🧒");
    map.insert("boy", "👦");
    map.insert("girl", "👧");
    map.insert("adult", "🧑");
    map.insert("man", "👨");
    map.insert("woman", "👩");
    map.insert("older_adult", "🧓");
    map.insert("older_man", "👴");
    map.insert("older_woman", "👵");

    // Animals & Nature
    map.insert("dog", "🐶");
    map.insert("cat", "🐱");
    map.insert("mouse", "🐭");
    map.insert("hamster", "🐹");
    map.insert("rabbit", "🐰");
    map.insert("fox", "🦊");
    map.insert("bear", "🐻");
    map.insert("panda", "🐼");
    map.insert("koala", "🐨");
    map.insert("tiger", "🐯");
    map.insert("lion", "🦁");
    map.insert("cow", "🐮");
    map.insert("pig", "🐷");
    map.insert("frog", "🐸");
    map.insert("monkey", "🐵");
    map.insert("see_no_evil", "🙈");
    map.insert("hear_no_evil", "🙉");
    map.insert("speak_no_evil", "🙊");
    map.insert("monkey_face", "🐵");
    map.insert("chicken", "🐔");
    map.insert("penguin", "🐧");
    map.insert("bird", "🐦");
    map.insert("baby_chick", "🐤");
    map.insert("bug", "🐛");
    map.insert("butterfly", "🦋");
    map.insert("snail", "🐌");
    map.insert("bee", "🐝");
    map.insert("fish", "🐟");
    map.insert("dolphin", "🐬");
    map.insert("whale", "🐳");
    map.insert("turtle", "🐢");
    map.insert("octopus", "🐙");
    map.insert("crab", "🦀");

    // Food & Drink
    map.insert("pizza", "🍕");
    map.insert("hamburger", "🍔");
    map.insert("fries", "🍟");
    map.insert("hotdog", "🌭");
    map.insert("taco", "🌮");
    map.insert("burrito", "🌯");
    map.insert("popcorn", "🍿");
    map.insert("cookie", "🍪");
    map.insert("cake", "🍰");
    map.insert("birthday", "🎂");
    map.insert("icecream", "🍦");
    map.insert("doughnut", "🍩");
    map.insert("coffee", "☕");
    map.insert("tea", "🍵");
    map.insert("beer", "🍺");
    map.insert("wine_glass", "🍷");
    map.insert("cocktail", "🍸");
    map.insert("apple", "🍎");
    map.insert("banana", "🍌");
    map.insert("strawberry", "🍓");
    map.insert("watermelon", "🍉");
    map.insert("grapes", "🍇");
    map.insert("cherry", "🍒");
    map.insert("peach", "🍑");
    map.insert("pineapple", "🍍");

    // Activities
    map.insert("soccer", "⚽");
    map.insert("basketball", "🏀");
    map.insert("football", "🏈");
    map.insert("baseball", "⚾");
    map.insert("tennis", "🎾");
    map.insert("volleyball", "🏐");
    map.insert("8ball", "🎱");
    map.insert("trophy", "🏆");
    map.insert("medal", "🏅");
    map.insert("dart", "🎯");
    map.insert("guitar", "🎸");
    map.insert("microphone", "🎤");
    map.insert("headphones", "🎧");
    map.insert("video_game", "🎮");
    map.insert("dice", "🎲");
    map.insert("game_die", "🎲");

    // Travel & Places
    map.insert("car", "🚗");
    map.insert("taxi", "🚕");
    map.insert("bus", "🚌");
    map.insert("train", "🚆");
    map.insert("airplane", "✈️");
    map.insert("rocket", "🚀");
    map.insert("bike", "🚲");
    map.insert("house", "🏠");
    map.insert("office", "🏢");
    map.insert("school", "🏫");
    map.insert("hospital", "🏥");
    map.insert("hotel", "🏨");
    map.insert("bank", "🏦");
    map.insert("church", "⛪");
    map.insert("mountain", "⛰️");
    map.insert("beach", "🏖️");
    map.insert("sunrise", "🌅");
    map.insert("sunset", "🌇");
    map.insert("rainbow", "🌈");

    // Objects
    map.insert("book", "📖");
    map.insert("notebook", "📓");
    map.insert("pencil", "✏️");
    map.insert("pen", "🖊️");
    map.insert("lock", "🔒");
    map.insert("unlock", "🔓");
    map.insert("key", "🔑");
    map.insert("hammer", "🔨");
    map.insert("bomb", "💣");
    map.insert("fire", "🔥");
    map.insert("bulb", "💡");
    map.insert("candle", "🕯️");
    map.insert("gift", "🎁");
    map.insert("balloon", "🎈");
    map.insert("tada", "🎉");
    map.insert("confetti_ball", "🎊");
    map.insert("bell", "🔔");
    map.insert("musical_note", "🎵");
    map.insert("notes", "🎶");
    map.insert("phone", "📱");
    map.insert("computer", "💻");
    map.insert("email", "📧");
    map.insert("inbox_tray", "📥");
    map.insert("outbox_tray", "📤");
    map.insert("calendar", "📅");
    map.insert("clock", "🕐");
    map.insert("hourglass", "⌛");
    map.insert("mag", "🔍");
    map.insert("flashlight", "🔦");
    map.insert("camera", "📷");

    // Symbols
    map.insert("check", "✅");
    map.insert("x", "❌");
    map.insert("warning", "⚠️");
    map.insert("question", "❓");
    map.insert("exclamation", "❗");
    map.insert("star", "⭐");
    map.insert("sparkles", "✨");
    map.insert("zap", "⚡");
    map.insert("boom", "💥");
    map.insert("100", "💯");
    map.insert("sos", "🆘");
    map.insert("new", "🆕");
    map.insert("ok", "🆗");
    map.insert("cool", "🆒");
    map.insert("free", "🆓");
    map.insert("arrow_up", "⬆️");
    map.insert("arrow_down", "⬇️");
    map.insert("arrow_left", "⬅️");
    map.insert("arrow_right", "➡️");
    map.insert("recycle", "♻️");
    map.insert("infinity", "♾️");
    map.insert("tm", "™️");
    map.insert("copyright", "©️");
    map.insert("registered", "®️");

    map
});

/// Converts an emoji shortcode to its Unicode representation.
///
/// Returns `Some(unicode)` if the shortcode is recognized, `None` otherwise.
///
/// # Examples
///
/// ```
/// use markdown::emoji::shortcode_to_unicode;
///
/// assert_eq!(shortcode_to_unicode("smile"), Some("😄"));
/// assert_eq!(shortcode_to_unicode("heart"), Some("❤️"));
/// assert_eq!(shortcode_to_unicode("unknown"), None);
/// ```
pub fn shortcode_to_unicode(shortcode: &str) -> Option<&'static str> {
    EMOJI_MAP.get(shortcode).copied()
}

/// Checks if a shortcode is a valid emoji.
pub fn is_valid_shortcode(shortcode: &str) -> bool {
    EMOJI_MAP.contains_key(shortcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_emojis() {
        assert_eq!(shortcode_to_unicode("smile"), Some("😄"));
        assert_eq!(shortcode_to_unicode("heart"), Some("❤️"));
        assert_eq!(shortcode_to_unicode("thumbsup"), Some("👍"));
        assert_eq!(shortcode_to_unicode("rocket"), Some("🚀"));
        assert_eq!(shortcode_to_unicode("fire"), Some("🔥"));
    }

    #[test]
    fn test_invalid_shortcode() {
        assert_eq!(shortcode_to_unicode("invalid_emoji_code"), None);
        assert_eq!(shortcode_to_unicode(""), None);
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid_shortcode("smile"));
        assert!(is_valid_shortcode("heart"));
        assert!(!is_valid_shortcode("invalid"));
    }
}
