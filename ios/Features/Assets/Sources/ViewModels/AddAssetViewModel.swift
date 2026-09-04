// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

struct AddAssetViewModel {
    let asset: Asset
    let link: BlockExplorerLink?

    var nameTitle: String {
        Localized.Asset.name
    }

    var symbolTitle: String {
        Localized.Asset.symbol
    }

    var decimalsTitle: String {
        Localized.Asset.decimals
    }

    var typeTitle: String {
        Localized.Common.type
    }

    var explorerText: String? {
        link.map { Localized.Transaction.viewOn($0.name) }
    }

    var name: String {
        asset.name
    }

    var symbol: String {
        asset.symbol
    }

    var decimals: String {
        asset.decimals.asString
    }

    var type: String {
        asset.id.assetType?.rawValue ?? ""
    }

    var explorerUrl: URL? {
        link?.url
    }
}
