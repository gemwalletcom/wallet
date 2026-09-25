// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

public protocol AmountDisplayable: Sendable {
    var amount: TextValue { get }
    var fiat: TextValue? { get }
    var assetImage: AssetImage? { get }
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
}

// MARK: - FACTORY

extension AmountDisplay {
    static func symbol(
        asset: Asset,
    ) -> AmountDisplay {
        .symbol(SymbolViewModel(asset: asset))
    }
}
