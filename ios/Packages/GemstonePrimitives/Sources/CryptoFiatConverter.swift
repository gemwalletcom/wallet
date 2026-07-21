// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public struct CryptoFiatConverter: Sendable {
    private let converter = Gemstone.CryptoFiatConverter()

    public init() {}

    public func convertToCrypto(fiatAmount: String, decimals: Int, price: Double) throws -> String {
        try converter.convertToCrypto(fiatAmount: fiatAmount, decimals: UInt32(decimals), price: price)
    }
}
