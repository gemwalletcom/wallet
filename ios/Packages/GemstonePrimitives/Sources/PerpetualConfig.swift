// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import class Gemstone.Config
import Primitives

public struct PerpetualConfig {
    private init() {}

    public static var defaultLeverage: UInt8 {
        Config.shared.getPerpetualConfig().defaultLeverage
    }

    public static var depositAddress: String {
        Config.shared.getPerpetualConfig().depositAddress
    }

    public static var depositAssetId: String {
        Config.shared.getPerpetualConfig().depositAssetId
    }

    public static var depositAsset: Asset {
        guard let assetId = try? AssetId(id: depositAssetId) else {
            preconditionFailure("Invalid perpetual deposit asset id: \(depositAssetId)")
        }
        let usdc = Asset.hypercoreUSDC()
        return Asset(id: assetId, name: usdc.name, symbol: usdc.symbol, decimals: usdc.decimals, type: .token)
    }

    public static var minDeposit: BigInt {
        BigInt(Config.shared.getPerpetualConfig().minDeposit)
    }

    public static var minWithdraw: BigInt {
        BigInt(Config.shared.getPerpetualConfig().minWithdraw)
    }

    public static var pricesUpdateIntervalSeconds: TimeInterval {
        TimeInterval(Config.shared.getPerpetualConfig().pricesUpdateIntervalSeconds)
    }

    public static var leverageOptions: [UInt8] {
        Array(Config.shared.getPerpetualConfig().leverageOptions)
    }

    public static var takeProfitOptions: [UInt8] {
        Array(Config.shared.getPerpetualConfig().takeProfitPercentOptions)
    }

    public static var stopLossOptions: [UInt8] {
        Array(Config.shared.getPerpetualConfig().stopLossPercentOptions)
    }

    public static func selectLeverage(desired: UInt8, options: [UInt8]) -> UInt8 {
        Config.shared.selectLeverage(desired: desired, options: Data(options))
    }

    public static func autocloseSuggestions(leverage: UInt8) -> [UInt8] {
        Array(Config.shared.getAutocloseSuggestions(leverage: leverage))
    }
}
