// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import protocol Gemstone.GemAddressServiceProtocol
import BigInt
import Foundation
import Gemstone
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
    func map() -> Gemstone.Chain {
        rawValue
    }

    var asset: Primitives.Asset {
        chainAsset.asset
    }

    var networkName: String {
        chainAsset.networkName
    }

    var accountActivationFeeUrl: URL? {
        guard let url = ChainConfig.config(chain: self).accountActivationFeeUrl else {
            return .none
        }
        return URL(string: url)
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

    var hasNativeAsset: Bool {
        ChainConfig.config(chain: self).hasNativeAsset
    }

    var isDefiSupported: Bool {
        ChainConfig.config(chain: self).isDefiSupported
    }

    var type: ChainType {
        guard let type = ChainType(rawValue: ChainConfig.config(chain: self).chainType) else {
            preconditionFailure("Invalid chain type for \(self)")
        }
        return type
    }

    var feeUnitType: FeeUnitType {
        guard let feeUnitType = FeeUnitType(rawValue: ChainConfig.config(chain: self).feeUnitType) else {
            return .native
        }
        return feeUnitType
    }

    var feeUnitDecimals: Int {
        Int(FeeConfig.config(chain: self).decimals)
    }

    func feeRateDecimals(assetDecimals: Int) -> Int {
        switch feeUnitType {
        case .satVb, .gwei: feeUnitDecimals
        case .native: assetDecimals
        }
    }

    var maxCustomFeeRateMultiplier: Int {
        Int(FeeConfig.config(chain: self).maxMultiplier)
    }

    var minimumCustomFeeRate: BigInt? {
        FeeConfig.config(chain: self).minimumCustomFeeRate.map { BigInt($0) }
    }

    var customFeeEnabled: Bool {
        FeeConfig.config(chain: self).customFeeEnabled
    }

    var blockTime: UInt32 {
        ChainConfig.config(chain: self).blockTime
    }

    var transactionTimeoutSeconds: UInt32 {
        ChainConfig.config(chain: self).transactionTimeout / 1000
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

    var feeAssetIds: [Primitives.AssetId] {
        assetConfig.chainFeeAssetIds(chain: rawValue).compactMap { try? Primitives.AssetId(id: $0) }
    }

    var defaultAssets: [Primitives.Asset] {
        assetConfig.walletDefaultAssets(chain: map()).map { asset in
            guard let asset = try? Primitives.Asset(asset) else {
                preconditionFailure("Invalid default asset for \(self)")
            }
            return asset
        }
    }

    func defaultAsset(type: Primitives.AssetType) -> Primitives.Asset {
        guard let asset = defaultAssets.first(where: { $0.type == type }) else {
            preconditionFailure("Missing \(type) default asset for \(self)")
        }
        return asset
    }

    func isValidAddress(_ address: String, addressService: any GemAddressServiceProtocol) -> Bool {
        addressService.validate(address: checksumAddress(address, addressService: addressService), chain: rawValue)
    }

    func checksumAddress(_ address: String, addressService: any GemAddressServiceProtocol) -> String {
        addressService.checksum(address: address, chain: rawValue)
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
        let assets = map { $0.json() }
        return assetConfig.matchingAssets(assets: assets, query: query).compactMap { try? Primitives.Asset($0) }
    }
}
