// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Localization
import Primitives

public enum TransferAmountCalculatorError: Equatable {
    case insufficientBalance(Asset, requirement: BalanceRequirement)
    case insufficientNetworkFee(Asset, requirement: BalanceRequirement?)
    case minimumAccountBalanceTooLow(Asset, requirement: BalanceRequirement)
}

extension TransferAmountCalculatorError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .insufficientBalance(asset, _):
            Localized.Transfer.insufficientBalance(Self.title(asset: asset))
        case let .insufficientNetworkFee(asset, _):
            Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
        case let .minimumAccountBalanceTooLow(asset, requirement):
            Localized.Transfer.minimumAccountBalance(Self.formattedValue(requirement.required, asset: asset))
        }
    }

    private static func title(asset: Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }

    private static func formattedValue(_ value: BigInt, asset: Asset) -> String {
        ValueFormatter(style: .full).string(value, asset: asset).boldMarkdown()
    }
}
