// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TransactionMetadata {
    func mapToAnyCodableValue() -> AnyCodableValue? {
        switch self {
        case let .perpetual(perpetualMetadata):
            (try? Primitives.TransactionPerpetualMetadata(perpetualMetadata)).flatMap(AnyCodableValue.encode)
        case let .swap(swapMetadata):
            (try? swapMetadata.map()).flatMap(AnyCodableValue.encode)
        }
    }
}
