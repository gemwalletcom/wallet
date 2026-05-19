// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives

public struct LeverageOption: WheelPickerDisplayable, Comparable, Sendable {
    public static let allOptions: [LeverageOption] = Config.shared.perpetualConfig().leverageOptions.map { .init(value: $0) }

    public let value: UInt8

    public init(value: UInt8) {
        self.value = value
    }

    public var id: UInt8 {
        value
    }

    public var displayText: String {
        "\(value)x"
    }

    public static func < (lhs: LeverageOption, rhs: LeverageOption) -> Bool {
        lhs.value < rhs.value
    }

    public static func option(desiredValue: UInt8, from available: [LeverageOption]) -> LeverageOption {
        LeverageOption(
            value: Config.shared.selectLeverage(
                desired: desiredValue,
                options: Data(available.map { $0.value }),
            ),
        )
    }
}
