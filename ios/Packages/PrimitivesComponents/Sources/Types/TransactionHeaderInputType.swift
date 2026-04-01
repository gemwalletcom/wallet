// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public enum TransactionHeaderInputType: Sendable {
    case amount(showFiat: Bool)
    case nft(name: String?, id: String)
    case swap(SwapHeaderInput)
    case symbol
    case assetImage
}
