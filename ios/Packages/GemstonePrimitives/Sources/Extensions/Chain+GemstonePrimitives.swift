// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import class Gemstone.GemAssetConfigService
import Primitives

private let chainAssets: [Primitives.Chain: Primitives.ChainAsset] = Primitives.Chain.allCases.reduce(into: [:]) { result, chain in
    result[chain] = GemAssetConfigService.shared.chainAsset(chain: chain.rawValue).toPrimitives()
}

public extension Primitives.Chain {
    var asset: Primitives.Asset {
        chainAsset.asset
    }

    var networkName: String {
        chainAsset.networkName
    }

    var isMemoSupported: Bool {
        ChainConfig.config(chain: self).isMemoSupported
    }

    var type: Primitives.ChainType {
        ChainConfig.config(chain: self).chainType.toPrimitives()
    }

    var iconChain: Primitives.Chain {
        Primitives.Chain(core: ChainConfig.config(chain: self).iconChain)
    }

    func defaultAsset(type: Primitives.AssetType) -> Primitives.Asset {
        guard let asset = GemAssetConfigService.shared.defaultAsset(chain: toGem(), assetType: type.toGem()) else {
            preconditionFailure("Missing \(type) default asset for \(self)")
        }
        return asset.toPrimitives()
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
