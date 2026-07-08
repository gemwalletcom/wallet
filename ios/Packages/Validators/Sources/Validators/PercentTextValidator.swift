// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Localization
import Primitives

public struct PercentTextValidator: TextValidator {
    private let maximum: Double
    private let formatter = NumericFormatter()

    public init(maximum: Double) {
        self.maximum = maximum
    }

    public func validate(_ text: String) throws {
        guard let value = formatter.double(from: text) else { return }

        guard value <= maximum else {
            throw AnyError(Localized.Swap.slippageMax("\(Int(maximum))%"))
        }
    }

    public var id: String {
        "PercentTextValidator"
    }
}
