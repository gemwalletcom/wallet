// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import class Gemstone.CryptoFiatConverter
import Primitives

public struct AssetValueConverter: Sendable {
    private let formatter: ValueFormatter
    private let converter = CryptoFiatConverter()

    public init(formatter: ValueFormatter = .auto) {
        self.formatter = formatter
    }

    public func convertToFiat(
        amount: String,
        price: AssetPrice,
        decimals: Int,
    ) throws -> Decimal {
        let value = try formatter.displayedNumber(from: formatter.number(amount: amount), decimals: decimals)
        let fiat = try converter.toFiat(value: value, decimals: UInt32(decimals), price: price.price)
        guard let result = Decimal(string: fiat) else {
            throw AnyError("Invalid fiat value: \(fiat)")
        }
        return result
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

extension AssetValueConverter {
    private func calculateAssetAmount(
        fiat: Decimal,
        price: AssetPrice,
        decimals: Int,
    ) throws -> Decimal {
        let value = try converter.toCrypto(fiatAmount: "\(fiat)", decimals: UInt32(decimals), price: price.price)
        guard let amount = Decimal(string: value) else {
            throw AnyError("Invalid amount: \(value)")
        }
        return amount
    }
}
