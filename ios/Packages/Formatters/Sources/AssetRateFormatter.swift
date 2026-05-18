// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public struct AssetRateFormatter {
    public enum Direction {
        case direct
        case inverse
    }

    private let formatter: ValueFormatter
    private let numericFormatter: NumericFormatter

    public init(
        formatter: ValueFormatter = ValueFormatter.full,
        numericFormatter: NumericFormatter = NumericFormatter(),
    ) {
        self.formatter = formatter
        self.numericFormatter = numericFormatter
    }

    public func rate(
        fromAsset: Asset,
        toAsset: Asset,
        fromValue: BigInt,
        toValue: BigInt,
        direction: Direction = .direct,
    ) throws -> String {
        let (baseAsset, quoteAsset, baseValue, quoteValue): (Asset, Asset, BigInt, BigInt) = switch direction {
        case .direct: (fromAsset, toAsset, fromValue, toValue)
        case .inverse: (toAsset, fromAsset, toValue, fromValue)
        }

        let baseAmount = try formatter.double(from: baseValue, decimals: baseAsset.decimals.asInt)
        let quoteAmount = try formatter.double(from: quoteValue, decimals: quoteAsset.decimals.asInt)

        let amount = quoteAmount / baseAmount
        let amountString = numericFormatter.string(amount, symbol: quoteAsset.symbol)

        return "1 \(baseAsset.symbol) ≈ \(amountString)"
    }
}
