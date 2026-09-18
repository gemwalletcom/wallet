// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRow
import enum Gemstone.GemListRowTitle

enum PreferencesRowDestination {
    case currency
    case language
    case appearance
    case networks
    case contacts

    init?(row: GemListRow) {
        guard case let .link(title, _, _) = row else { return nil }
        switch title {
        case .currency: self = .currency
        case .language: self = .language
        case .appearance: self = .appearance
        case .networks: self = .networks
        case .contacts: self = .contacts
        default: return nil
        }
    }
}
