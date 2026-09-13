// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum Emoji {
    public static let greenCircle = "🟢"
    public static let orangeCircle = "🟠"
    public static let redCircle = "🔴"
    public static let checkmark = "✅"
    public static let reject = "❌"
    public static let random = "🎲"
    public static let rocket = "🚀"
    public static let turtle = "🐢"
    public static let gem = "💎"
    public static let party = "🎉"

    public enum WalletAvatar: String {
        case gem = "💎"
        case moneyBag = "💰"
        case lock = "🔒"
        case warning = "⚠️"
        case firework = "🎆"
        case gift = "🎁"
    }

    public enum FeeRate: String {
        case slow = "⏱️"
        case normal = "💎"
        case fast = "⚡️"
        case custom = "⚙️"
    }
}

// MARK: - Previews

#Preview {
    let symbols = [
        (Emoji.greenCircle, "Green Circle"),
        (Emoji.orangeCircle, "Orange Circle"),
        (Emoji.redCircle, "Red Circle"),
        (Emoji.checkmark, "Checkmark"),
        (Emoji.reject, "Reject"),
        (Emoji.random, "Random"),
        (Emoji.rocket, "Rocket"),
        (Emoji.turtle, "Turtle"),
        (Emoji.gem, "Gem"),
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
