// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives

public struct ValueConverter: Sendable {
    private let formatter: ValueFormatter
    private let converter = CryptoFiatConverter()

    public init(formatter: ValueFormatter = .auto) {
        self.formatter = formatter
    }

    public func convertToFiat(
        amount: String,
        price: AssetPrice,
    ) throws -> Decimal {
        let value = try formatter.number(amount: amount)
        return value * Decimal(price.price)
    }

    public func convertToDisplayedAmount(
        fiatValue: String,
        price: AssetPrice,
        decimals: Int,
    ) throws -> BigInt {
        let fiatNumber = try formatter.number(amount: fiatValue)
        let amount = try calculateAssetAmount(fiat: fiatNumber, price: price, decimals: decimals)
        let value = try formatter.displayedNumber(from: amount, decimals: decimals)
        guard !value.isZero else {
            throw AnyError("Cannot format zero amount")
        }
        return value
    }
}

// MARK: - Private

extension ValueConverter {
    private func calculateAssetAmount(
        fiat: Decimal,
        price: AssetPrice,
        decimals: Int,
    ) throws -> Decimal {
        let value = try converter.toCrypto(fiatAmount: "\(fiat)", decimals: decimals, price: price.price)
        guard let amount = Decimal(string: value) else {
            throw AnyError("Invalid amount: \(value)")
        }
        return amount
    }
}
