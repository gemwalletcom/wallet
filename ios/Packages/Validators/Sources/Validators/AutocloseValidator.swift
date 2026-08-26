// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import GemstonePrimitives
import Primitives

public struct AutocloseValidator: TextValidator {
    private let type: TpslType
    private let direction: PerpetualDirection
    private let marketPrice: Double

    public init(
        type: TpslType,
        direction: PerpetualDirection,
        marketPrice: Double,
    ) {
        self.type = type
        self.direction = direction
        self.marketPrice = marketPrice
    }

    public func validate(_ text: String) throws {
        guard !text.isEmpty else { return }

        let formatter = NumberFormatter()
        formatter.locale = Locale.current
        formatter.numberStyle = .decimal

        guard let price = formatter.number(from: text)?.doubleValue else {
            throw TransferError.invalidAmount
        }

        let validator = try GemstonePrimitives.AutocloseValidator(type: type, direction: direction, marketPrice: marketPrice)
        switch validator.validate(price) {
        case .valid:
            break
        case .invalidAmount:
            throw TransferError.invalidAmount
        case .triggerMustBeHigher:
            throw PerpetualError.triggerPriceMustBeHigher
        case .triggerMustBeLower:
            throw PerpetualError.triggerPriceMustBeLower
        }
    }

    public var id: String {
        "AutocloseValidator"
    }
}
