// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public enum TransferAmountCalculatorError: Equatable {
    case insufficientBalance(Asset, requirement: BalanceRequirement)
}

extension TransferAmountCalculatorError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .insufficientBalance(asset, _):
            Localized.Transfer.insufficientBalance(Self.title(asset: asset))
        }
    }

    private static func title(asset: Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }
}
