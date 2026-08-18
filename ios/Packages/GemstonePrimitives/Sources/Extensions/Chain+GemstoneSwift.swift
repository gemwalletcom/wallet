// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import func Gemstone.chainAssetWrapper
import Primitives

private let chainAssets: [Primitives.Chain: ChainAsset] = Primitives.Chain.allCases.reduce(into: [:]) { result, chain in
    let wrapper = chainAssetWrapper(chain: chain.rawValue)
    result[chain] = ChainAsset(
        asset: try! wrapper.asset.map(),
        networkName: wrapper.networkName,
    )
}

public extension Gemstone.Chain {
    func map() throws -> Primitives.Chain {
        try Primitives.Chain(id: self)
    }
}

public extension Primitives.Chain {
    var asset: Asset {
        chainAsset.asset
    }

    var networkName: String {
        chainAsset.networkName
    }

    var accountActivationFee: Int32? {
        ChainConfig.config(chain: self).accountActivationFee
    }

    var accountActivationFeeUrl: URL? {
        guard let url = ChainConfig.config(chain: self).accountActivationFeeUrl else {
            return .none
        }
        return URL(string: url)
    }

    var tokenActivateFee: BigInt {
        BigInt(ChainConfig.config(chain: self).tokenActivationFee ?? 0)
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

    var isStakeSupported: Bool {
        ChainConfig.config(chain: self).isStakeSupported
    }

    var isNFTSupported: Bool {
        ChainConfig.config(chain: self).isNftSupported
    }

    var isDefiSupported: Bool {
        ChainConfig.config(chain: self).isDefiSupported
    }

    var type: ChainType {
        ChainType(rawValue: ChainConfig.config(chain: self).chainType)!
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

    var transactionStateConfig: Primitives.JobConfiguration {
        Gemstone.transactionStateConfig(chain: rawValue).map()
    }

    var defaultAssets: [Asset] {
        switch self {
        case .tron:
            [
                .tronUSDT(),
            ]
        case .hyperCore:
            [
                .hypercoreUSDC(),
                .hypercoreSpotUSDC(),
            ]
        case .tempo:
            [
                .tempoUSDC(),
            ]
        default:
            []
        }
    }

    func isValidAddress(_ address: String) -> Bool {
        Gemstone.validateAddress(address: checksumAddress(address), chain: rawValue)
    }

    func checksumAddress(_ address: String) -> String {
        Gemstone.checksumAddress(address: address, chain: rawValue)
    }

    var isPrivateKeyImportSupported: Bool {
        Gemstone.supportsPrivateKeyImport(chain: rawValue)
    }

    func matches(query: String) -> Bool {
        networkName.localizedCaseInsensitiveContains(query) ||
            rawValue.localizedCaseInsensitiveContains(query) ||
            asset.name.localizedCaseInsensitiveContains(query) ||
            asset.symbol.localizedCaseInsensitiveContains(query)
    }
}

private extension Primitives.Chain {
    var chainAsset: ChainAsset {
        chainAssets[self]!
    }
}

public extension [Primitives.Chain] {
    func sortByRank() -> [Primitives.Chain] {
        sorted { AssetScore.defaultRank(chain: $0) > AssetScore.defaultRank(chain: $1) }
    }

    func filter(query: String) -> [Primitives.Chain] {
        guard !query.isEmpty else { return self }
        return filter { $0.matches(query: query) }
    }
}
