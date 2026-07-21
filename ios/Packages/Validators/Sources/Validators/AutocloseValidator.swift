// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import GemstonePrimitives
import Primitives

public struct AutocloseValidator: TextValidator {
    private let type: TpslType
    private let direction: PerpetualDirection
    private let validator: GemstonePrimitives.AutocloseValidator

    public init(
        type: TpslType,
        marketPrice: Double,
        direction: PerpetualDirection,
    ) {
        self.type = type
        self.direction = direction
        validator = GemstonePrimitives.AutocloseValidator(type: type, direction: direction, marketPrice: marketPrice)
    }

    public func validate(_ text: String) throws {
        guard !text.isEmpty else { return }

        let formatter = NumberFormatter()
        formatter.locale = Locale.current
        formatter.numberStyle = .decimal

        guard let price = formatter.number(from: text)?.doubleValue else {
            throw TransferError.invalidAmount
        }

        switch validator.validate(price) {
        case .valid:
            break
        case .invalidAmount:
            throw TransferError.invalidAmount
        case .triggerMustBeHigher, .triggerMustBeLower:
            throw PerpetualError.invalidAutoclose(type: type, direction: direction)
        }
    }

    public var id: String {
        "AutocloseValidator"
    }
}
