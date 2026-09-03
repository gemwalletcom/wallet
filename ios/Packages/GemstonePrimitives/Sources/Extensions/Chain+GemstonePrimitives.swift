// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import class Gemstone.GemAssetConfigService
import Primitives

private let assetConfig = GemAssetConfigService()

private let chainAssets: [Primitives.Chain: Primitives.ChainAsset] = Primitives.Chain.allCases.reduce(into: [:]) { result, chain in
    guard let chainAsset = try? Primitives.ChainAsset(assetConfig.chainAsset(chain: chain.rawValue)) else {
        preconditionFailure("Invalid chain asset for \(chain)")
    }
    result[chain] = chainAsset
}

public extension Gemstone.Chain {
    func map() throws -> Primitives.Chain {
        try Primitives.Chain(id: self)
    }
}

public extension Primitives.Chain {
    init(core rawValue: String) {
        guard let chain = Primitives.Chain(rawValue: rawValue) else {
            preconditionFailure("failed to decode Chain from Core: \(rawValue)")
        }
        self = chain
    }

    func map() -> Gemstone.Chain {
        rawValue
    }

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

    var isSwapSupported: Bool {
        ChainConfig.config(chain: self).isSwapSupported
    }

    var isTokenSupported: Bool {
        ChainConfig.config(chain: self).isTokenSupported
    }

    var isStakeSupported: Bool {
        ChainConfig.config(chain: self).isStakeSupported
    }

    var isNFTSupported: Bool {
        ChainConfig.config(chain: self).isNftSupported
    }

    var type: ChainType {
        guard let type = ChainType(rawValue: ChainConfig.config(chain: self).chainType) else {
            preconditionFailure("Invalid chain type for \(self)")
        }
        return type
    }

    var iconChain: Primitives.Chain {
        Primitives.Chain(rawValue: ChainConfig.config(chain: self).iconChain) ?? self
    }

    var badgeChain: Primitives.Chain? {
        ChainConfig.config(chain: self).badgeChain.flatMap { Primitives.Chain(rawValue: $0) }
    }

    var supportsNftTransfer: Bool {
        ChainConfig.config(chain: self).supportsNftTransfer
    }

    var defaultAssets: [Primitives.Asset] {
        assetConfig.walletDefaultAssets(chain: map()).map { $0.map() }
    }

    func defaultAsset(type: Primitives.AssetType) -> Primitives.Asset {
        guard let asset = defaultAssets.first(where: { $0.type == type }) else {
            preconditionFailure("Missing \(type) default asset for \(self)")
        }
        return asset
    }

    var isPrivateKeyImportSupported: Bool {
        Gemstone.supportsPrivateKeyImport(chain: rawValue)
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

public extension [Primitives.Chain] {
    func sortByRank() -> [Primitives.Chain] {
        sorted { AssetScore.defaultRank(chain: $0) > AssetScore.defaultRank(chain: $1) }
    }
}

public extension [Primitives.Asset] {
    func matching(query: String) -> [Primitives.Asset] {
        let assets = map { $0.map() }
        return assetConfig.matchingAssets(assets: assets, query: query).map { $0.map() }
    }
}
