// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import GemstonePrimitives
import Primitives

public struct AutocloseValidator: TextValidator {
    private let type: TpslType
    private let direction: PerpetualDirection
    private let marketPrice: Double
    private let formatter = NumericFormatter()

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

        guard let price = formatter.double(from: text) else {
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
