// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import class Gemstone.GemAssetConfigService
import Gemstone
import Primitives

private let chainAssets: [Primitives.Chain: Primitives.ChainAsset] = Primitives.Chain.allCases.reduce(into: [:]) { result, chain in
    result[chain] = GemAssetConfigService.shared.chainAsset(chain: chain.rawValue).map()
}

public extension Primitives.Chain {

    var asset: Primitives.Asset {
        chainAsset.asset
    }

    var networkName: String {
        chainAsset.networkName
    }

    var minimumAccountBalance: BigInt {
        BigInt(ChainConfig.config(chain: self).minimumAccountBalance ?? .zero)
    }

    var isMemoSupported: Bool {
        ChainConfig.config(chain: self).isMemoSupported
    }

    var type: Primitives.ChainType {
        ChainConfig.config(chain: self).chainType.map()
    }

    var iconChain: Primitives.Chain {
        Primitives.Chain(core: ChainConfig.config(chain: self).iconChain)
    }

    func defaultAsset(type: Primitives.AssetType) -> Primitives.Asset {
        guard let asset = GemAssetConfigService.shared.defaultAsset(chain: map(), assetType: type.map()) else {
            preconditionFailure("Missing \(type) default asset for \(self)")
        }
        return asset.map()
    }

}

private extension Primitives.Chain {
    var chainAsset: Primitives.ChainAsset {
        guard let asset = chainAssets[self] else {
            preconditionFailure("Missing chain asset for \(self)")
        }
        return asset
    }
}

public extension [Primitives.Asset] {
    func matching(query: String) -> [Primitives.Asset] {
        let assets = map { $0.map() }
        return GemAssetConfigService.shared.matchingAssets(assets: assets, query: query).map { $0.map() }
    }
}
