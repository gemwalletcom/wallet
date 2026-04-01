// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

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
