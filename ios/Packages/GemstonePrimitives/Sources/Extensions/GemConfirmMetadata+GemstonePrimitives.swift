// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConfirmMetadata
import Primitives

public extension GemConfirmMetadata {
    var assetPrice: Primitives.Price? { assetPrice().map { $0.toPrimitives().mapToPrice() } }
    var feePrice: Primitives.Price? { feePrice().map { $0.toPrimitives().mapToPrice() } }

    func price(for assetId: String) -> Primitives.Price? {
        price(assetId: assetId).map { $0.toPrimitives().mapToPrice() }
    }
}
