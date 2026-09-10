// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.PaymentPrice
import Primitives

public enum TransactionHeaderInputType: Sendable {
    case amount(showFiat: Bool)
    case payment(PaymentPrice)
    case nft(name: String?, id: String)
    case swap(SwapHeaderInput)
    case symbol
    case assetImage
}
