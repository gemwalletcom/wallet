// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Perpetuals
import Primitives
import PrimitivesTestKit

extension AutocloseType {
    static func mock(data: AutocloseOpenData = .mock(symbol: "BTC", direction: .long, marketPrice: 100, leverage: 10, size: 1, assetDecimals: 8)) -> AutocloseType {
        .open(data, onComplete: { _ in })
    }
}
