// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import enum Gemstone.GemAmountError
import GemstonePrimitives
import Localization
import Primitives

extension GemAmountError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .InvalidNumber, .PriceMissing, .Zero: Localized.Errors.invalidAmount
        case let .BelowMinimum(asset, minimum):
            Localized.Transfer.minimumAmount(ValueFormatter(style: .auto).string(minimum, asset: asset.map()).boldMarkdown())
        case let .InsufficientBalance(asset, _):
            Localized.Transfer.insufficientBalance(Self.title(asset: asset.map()))
        }
    }

    private static func title(asset: Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
    }
}
