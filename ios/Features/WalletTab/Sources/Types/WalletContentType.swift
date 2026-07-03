// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

public enum WalletContentType: String, CaseIterable, Identifiable, Sendable {
    case assets
    case collections
    case defi

    public var id: String {
        rawValue
    }

    public var title: String {
        switch self {
        case .assets: Localized.Assets.title
        case .collections: Localized.Nft.collections
        case .defi: "DeFi"
        }
    }
}
