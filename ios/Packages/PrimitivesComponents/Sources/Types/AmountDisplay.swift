// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.GemAmountSign
import GemstonePrimitives
import Primitives
import Style

public protocol AmountDisplayable: Sendable {
    var amount: TextValue { get }
    var fiat: TextValue? { get }
    var assetImage: AssetImage? { get }
}

public struct AmountDisplayStyle: Sendable {
    public let sign: GemAmountSign
    public let formatter: ValueFormatter
    public let currencyCode: String
    public let textStyle: TextStyle?

    public init(
        sign: GemAmountSign = .none,
        formatter: ValueFormatter = .auto,
        currencyCode: String,
        textStyle: TextStyle? = nil,
    ) {
        self.sign = sign
        self.formatter = formatter
        self.currencyCode = currencyCode
        self.textStyle = textStyle
    }
}

public enum AmountDisplay: Sendable {
    case numeric(NumericViewModel)
    case symbol(SymbolViewModel)
}

extension AmountDisplay: AmountDisplayable {
    public var amount: TextValue {
        switch self {
        case let .numeric(viewModel): viewModel.amount
        case let .symbol(viewModel): viewModel.amount
        }
    }

    public var fiat: TextValue? {
        switch self {
        case let .numeric(viewModel): viewModel.fiat
        case let .symbol(viewModel): viewModel.fiat
        }
    }

    public var assetImage: AssetImage? {
        switch self {
        case let .numeric(viewModel): viewModel.assetImage
        case let .symbol(viewModel): viewModel.assetImage
        }
    }

    func fiatVisibility(_ visible: Bool) -> AmountDisplay {
        switch self {
        case let .numeric(model):
            let style = AmountDisplayStyle(
                sign: model.style.sign,
                formatter: model.style.formatter,
                currencyCode: model.style.currencyCode,
                textStyle: model.style.textStyle,
            )
            return .numeric(
                NumericViewModel(
                    data: AssetValuePrice(
                        asset: model.data.asset,
                        value: model.data.value,
                        price: visible ? model.data.price : nil,
                    ),
                    style: style,
                ),
            )
        case .symbol:
            return self
        }
    }
}

// MARK: - FACTORY

public extension AmountDisplay {
    internal static func symbol(
        asset: Asset,
    ) -> AmountDisplay {
        .symbol(SymbolViewModel(asset: asset))
    }

    static func numeric(
        data: AssetValuePrice,
        style: AmountDisplayStyle,
    ) -> AmountDisplay {
        .numeric(NumericViewModel(data: data, style: style))
    }

    static func numeric(
        asset: Asset,
        price: Price? = nil,
        value: BigInt,
        sign: GemAmountSign = .none,
        currency: String,
        formatter: ValueFormatter = .full,
        textStyle: TextStyle? = nil,
    ) -> AmountDisplay {
        .numeric(
            data: AssetValuePrice(asset: asset, value: value, price: price),
            style: AmountDisplayStyle(
                sign: sign,
                formatter: formatter,
                currencyCode: currency,
                textStyle: textStyle,
            ),
        )
    }
}
