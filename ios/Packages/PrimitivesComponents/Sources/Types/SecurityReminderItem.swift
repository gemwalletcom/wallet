// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSecurityReminderItem
import enum Gemstone.GemSecurityReminderKind
import enum Gemstone.GemWalletSecret
import Style

extension GemSecurityReminderItem {
    var image: ListItemImageStyle? {
        switch self {
        case .keepSafe: .emoji(Emoji.WalletAvatar.lock.rawValue)
        case .doNotShare: .emoji(Emoji.WalletAvatar.warning.rawValue)
        case .noRecovery: .emoji(Emoji.WalletAvatar.gem.rawValue)
        }
    }
}

public extension GemWalletSecret {
    var securityReminderKind: GemSecurityReminderKind {
        switch self {
        case .words: .phrase
        case .privateKey: .privateKey
        }
    }
}
