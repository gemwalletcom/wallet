// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle

enum SettingsRowDestination {
    case wallets
    case security
    case notifications
    case preferences
    case walletConnect
    case support
    case rewards
    case aboutUs
    case developer

    init?(row: GemListRow) {
        guard case let .link(title, _, _) = row else { return nil }
        switch title {
        case .wallets: self = .wallets
        case .security: self = .security
        case .notifications: self = .notifications
        case .preferences: self = .preferences
        case .walletConnect: self = .walletConnect
        case .support: self = .support
        case .rewards: self = .rewards
        case .aboutUs: self = .aboutUs
        case .developer: self = .developer
        default: return nil
        }
    }
}
