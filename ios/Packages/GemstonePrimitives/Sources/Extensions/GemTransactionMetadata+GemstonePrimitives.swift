// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TransactionMetadata {
    func mapToAnyCodableValue() -> AnyCodableValue? {
        switch self {
        case let .perpetual(perpetualMetadata):
            .encode(TransactionPerpetualMetadata(
                pnl: perpetualMetadata.pnl,
                price: perpetualMetadata.price,
                direction: perpetualMetadata.direction.map(),
                isLiquidation: perpetualMetadata.isLiquidation,
                provider: perpetualMetadata.provider?.map(),
            ))
        }
    }
}
