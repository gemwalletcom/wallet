// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

enum WalletSearchMode {
    case initial
    case searching
}

struct WalletSearchModel {
    var searchableQuery: String = .empty
}

// MARK: - Limits

extension WalletSearchModel {
    var perpetualsLimit: Int {
        WalletSearchConfig.perpetualsPreviewLimit
    }

    var nftsLimit: Int {
        WalletSearchConfig.nftsPreviewLimit
    }

    static var initialFetchLimit: Int {
        WalletSearchConfig.assetsInitialLimit + 1
    }

    static var searchItemTypes: [SearchItemType] {
        [.asset, .perpetual, .list, .nft]
    }

    var searchMode: WalletSearchMode {
        searchableQuery.isNotEmpty ? .searching : .initial
    }

    var assetsLimit: Int {
        switch searchMode {
        case .initial: WalletSearchConfig.assetsInitialLimit
        case .searching: WalletSearchConfig.assetsSearchLimit
        }
    }

    var fetchLimit: Int {
        switch searchMode {
        case .initial: assetsLimit + 1
        case .searching: WalletSearchConfig.resultsLimit
        }
    }
}
