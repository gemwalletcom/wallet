// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct ChatwootSettings {
    enum DarkMode: String {
        case auto
        case light
        case dark
    }

    let hideMessageBubble: Bool
    let locale: Locale
    let darkMode: DarkMode
    let enableEmojiPicker: Bool
    let enableEndConversation: Bool
}

extension ChatwootSettings {
    static let defaultSettings = ChatwootSettings(
        hideMessageBubble: true,
        locale: .current,
        darkMode: .auto,
        enableEmojiPicker: false,
        enableEndConversation: false,
    )
}
