// Copyright (c). Gem Wallet. All rights reserved.

enum SwapDetailRow: String, Identifiable {
    case provider
    case rate
    case estimatedTime
    case priceImpact
    case minimumReceive
    case slippage

    var id: String { rawValue }
}
