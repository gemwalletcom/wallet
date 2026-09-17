// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualMarketSection

struct PerpetualMarketSectionViewModel: Identifiable {
    enum Kind {
        case recents
        case positions
        case pinned
        case markets
        case empty
    }

    let section: GemPerpetualMarketSection

    var id: String {
        String(describing: section)
    }

    var title: String {
        section.title
    }

    var kind: Kind {
        switch section {
        case .recents: .recents
        case .positions: .positions
        case .pinned: .pinned
        case .markets: .markets
        case .empty: .empty
        }
    }
}
