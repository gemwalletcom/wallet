// Copyright (c). Gem Wallet. All rights reserved.

// Mappers for the types that cannot cross the FFI as JSON yet.
//
// Every type here reaches `Primitives.Asset`, whose `chain` and `tokenId` are
// `#[typeshare(skip)]` in the Rust primitives crate with no serde default. The
// platforms therefore cannot produce them, and an inbound decode would fail on
// the missing fields — so these still need a hand-written mapper.
//
// Both fields are exactly derivable from `id`, which is what the mappers below
// already do. Giving `Asset` a `Deserialize` that fills them from `id` would let
// all of these join `JsonCodable+GemstonePrimitives.swift` and delete this file.

import BigInt
import Foundation
import Gemstone
import Primitives

// MARK: - Asset

public extension GemAsset {
    func map() throws -> Primitives.Asset {
        try Primitives.Asset(
            id: AssetId(id: id),
            name: name,
            symbol: symbol,
            decimals: decimals,
            type: Primitives.AssetType(assetType),
        )
    }
}

public extension Primitives.Asset {
    func map() throws -> GemAsset {
        try GemAsset(
            id: id.identifier,
            chain: id.chain.rawValue,
            tokenId: id.tokenId,
            name: name,
            symbol: symbol,
            decimals: decimals,
            assetType: type.json(),
        )
    }
}

// MARK: - PerpetualType

public extension Gemstone.PerpetualType {
    func map() throws -> Primitives.PerpetualType {
        switch self {
        case let .open(confirmData): try .open(confirmData.map())
        case let .close(confirmData): try .close(confirmData.map())
        case let .increase(confirmData): try .increase(confirmData.map())
        case let .reduce(reduceData): try .reduce(reduceData.map())
        case let .modify(data): try .modify(data.map())
        }
    }
}

public extension Primitives.PerpetualType {
    func map() throws -> Gemstone.PerpetualType {
        switch self {
        case let .open(data), let .increase(data): try .open(data.map())
        case let .reduce(data): try .open(data.data.map())
        case let .close(data): try .close(data.map())
        case let .modify(data): try .modify(data.map())
        }
    }
}

// MARK: - PerpetualConfirmData

public extension Gemstone.PerpetualConfirmData {
    func map() throws -> Primitives.PerpetualConfirmData {
        try Primitives.PerpetualConfirmData(
            direction: Primitives.PerpetualDirection(direction),
            marginType: Primitives.PerpetualMarginType(marginType),
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            price: price,
            fiatValue: fiatValue,
            size: size,
            slippage: slippage,
            leverage: leverage,
            pnl: pnl,
            entryPrice: entryPrice,
            marketPrice: marketPrice,
            marginAmount: marginAmount,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }
}

public extension Primitives.PerpetualConfirmData {
    func map() throws -> Gemstone.PerpetualConfirmData {
        try Gemstone.PerpetualConfirmData(
            direction: direction.json(),
            marginType: marginType.json(),
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            price: price,
            fiatValue: fiatValue,
            size: size,
            slippage: slippage,
            leverage: leverage,
            pnl: pnl,
            entryPrice: entryPrice,
            marketPrice: marketPrice,
            marginAmount: marginAmount,
            takeProfit: takeProfit,
            stopLoss: stopLoss,
        )
    }
}

// MARK: - PerpetualReduceData

public extension Gemstone.PerpetualReduceData {
    func map() throws -> Primitives.PerpetualReduceData {
        try Primitives.PerpetualReduceData(
            data: data.map(),
            positionDirection: Primitives.PerpetualDirection(positionDirection),
        )
    }
}

// MARK: - PerpetualModifyConfirmData

public extension Gemstone.PerpetualModifyConfirmData {
    func map() throws -> Primitives.PerpetualModifyConfirmData {
        try Primitives.PerpetualModifyConfirmData(
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            modifyTypes: modifyTypes.map { try $0.map() },
            takeProfitOrderId: takeProfitOrderId,
            stopLossOrderId: stopLossOrderId,
        )
    }
}

public extension Primitives.PerpetualModifyConfirmData {
    func map() throws -> Gemstone.PerpetualModifyConfirmData {
        try Gemstone.PerpetualModifyConfirmData(
            baseAsset: baseAsset.map(),
            assetIndex: assetIndex,
            modifyTypes: modifyTypes.map { try $0.map() },
            takeProfitOrderId: takeProfitOrderId,
            stopLossOrderId: stopLossOrderId,
        )
    }
}

// MARK: - PerpetualModifyPositionType

public extension Gemstone.PerpetualModifyPositionType {
    func map() throws -> Primitives.PerpetualModifyPositionType {
        switch self {
        case let .tpsl(data):
            try .tpsl(data.map())
        case let .cancel(orders):
            try .cancel(orders.map { try $0.map() })
        }
    }
}

public extension Primitives.PerpetualModifyPositionType {
    func map() throws -> Gemstone.PerpetualModifyPositionType {
        switch self {
        case let .tpsl(data):
            try .tpsl(data.map())
        case let .cancel(orders):
            try .cancel(orders.map { try $0.map() })
        }
    }
}

// MARK: - TPSLOrderData

public extension Gemstone.TpslOrderData {
    func map() throws -> Primitives.TPSLOrderData {
        try Primitives.TPSLOrderData(
            direction: Primitives.PerpetualDirection(direction),
            takeProfit: takeProfit,
            stopLoss: stopLoss,
            size: size,
        )
    }
}

public extension Primitives.TPSLOrderData {
    func map() throws -> Gemstone.TpslOrderData {
        try Gemstone.TpslOrderData(
            direction: direction.json(),
            takeProfit: takeProfit,
            stopLoss: stopLoss,
            size: size,
        )
    }
}

// MARK: - CancelOrderData

public extension Gemstone.CancelOrderData {
    func map() throws -> Primitives.CancelOrderData {
        Primitives.CancelOrderData(assetIndex: assetIndex, orderId: orderId)
    }
}

public extension Primitives.CancelOrderData {
    func map() -> Gemstone.CancelOrderData {
        Gemstone.CancelOrderData(assetIndex: assetIndex, orderId: orderId)
    }
}
