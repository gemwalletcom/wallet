// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public struct PriceImpactValue: Equatable, Sendable {
    let type: SwapPriceImpactType
    let value: String
}
