// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemExplorerServiceProtocol
import GemstonePrimitives
import Foundation
import Localization
import Primitives

struct AddAssetViewModel {
    let asset: Asset
    let explorerService: any GemExplorerServiceProtocol

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

    private var tokenLink: BlockExplorerLink? {
        explorerService.getTokenUrl(chain: asset.chain.rawValue, address: asset.tokenId ?? "").map { BlockExplorerLink($0) }
    }

    var explorerText: String? {
        guard let link = tokenLink else {
            return .none
        }
        return Localized.Transaction.viewOn(link.name)
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
        tokenLink?.url
    }
}
