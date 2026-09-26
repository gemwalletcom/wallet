// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum Emoji {
    public static let checkmark = "✅"
    public static let reject = "❌"
    public static let random = "🎲"

    public enum WalletAvatar: String {
        case gem = "💎"
        case moneyBag = "💰"
        case lock = "🔒"
        case warning = "⚠️"
        case firework = "🎆"
        case gift = "🎁"
    }
}

// MARK: - Previews

#Preview {
    let symbols = [
        (Emoji.checkmark, "Checkmark"),
        (Emoji.reject, "Reject"),
        (Emoji.random, "Random"),
    ]

    return List {
        ForEach(symbols, id: \.1) { symbol in
            Section(header: Text(symbol.1)) {
                Text(symbol.0)
                    .frame(width: .list.image, height: .list.image)
                    .padding(.extraSmall)
            }
        }
    }
    .listStyle(InsetGroupedListStyle())
    .padding()
}
