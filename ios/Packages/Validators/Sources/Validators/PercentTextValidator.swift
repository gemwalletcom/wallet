// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Localization
import Primitives

public struct PercentTextValidator: TextValidator {
    private let minimum: Double
    private let maximum: Double
    private let formatter = NumericFormatter()

    public init(minimum: Double, maximum: Double) {
        self.minimum = minimum
        self.maximum = maximum
    }

    public func validate(_ text: String) throws {
        guard let value = formatter.double(from: text), value > 0 else { return }

        guard value >= minimum else {
            throw AnyError(Localized.Common.minimumValue("\(formatter.string(minimum))%"))
        }
        guard value <= maximum else {
            throw AnyError(Localized.Common.maximumValue("\(formatter.string(maximum))%"))
        }
    }

    public var id: String {
        "PercentTextValidator"
    }
}
