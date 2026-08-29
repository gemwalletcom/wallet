// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemPerpetualCloseInput
import enum Gemstone.GemPerpetualOrderAction
import struct Gemstone.GemPerpetualOrderInput
import class Gemstone.GemPerpetual
import Primitives

public extension PerpetualPositionAction {
    func order(
        usdcAmount: BigInt,
        usdcDecimals: Int,
        leverage: UInt8,
        takeProfit: String? = .none,
        stopLoss: String? = .none,
    ) throws -> PerpetualType {
        let data = transferData
        let input = try GemPerpetualOrderInput(
            action: orderAction(),
            direction: data.direction.json(),
            marginType: data.marginType.json(),
            baseAsset: data.baseAsset.json(),
            asset: data.asset.json(),
            assetIndex: Int32(data.assetIndex),
            price: data.price,
            usdcAmount: usdcAmount.description,
            usdcDecimals: Int32(usdcDecimals),
            leverage: leverage,
            slippage: .none,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
        return try PerpetualType(GemPerpetual(provider: data.provider.map()).order(input: input))
    }

    private func orderAction() throws -> GemPerpetualOrderAction {
        switch self {
        case .open: .open
        case .increase: .increase
        case let .reduce(_, _, positionDirection): .reduce(positionDirection: try positionDirection.json())
        }
    }
}

public extension PerpetualPosition {
    func closeOrder(assetIndex: Int32, perpetual: Perpetual, asset: Asset, baseAsset: Asset) throws -> PerpetualConfirmData {
        let input = try GemPerpetualCloseInput(
            assetIndex: assetIndex,
            direction: direction.json(),
            marginType: marginType.json(),
            baseAsset: baseAsset.json(),
            asset: asset.json(),
            marketPrice: perpetual.price,
            size: size,
            leverage: leverage,
            pnl: pnl,
            entryPrice: entryPrice,
            marginAmount: marginAmount,
            slippage: .none,
        )
        return try PerpetualConfirmData(GemPerpetual(provider: perpetual.provider.map()).closeOrder(input: input))
    }
}
