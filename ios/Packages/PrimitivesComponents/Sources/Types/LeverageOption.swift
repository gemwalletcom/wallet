// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives

public struct LeverageOption: WheelPickerDisplayable, Sendable {
    public static let allOptions: [LeverageOption] = PerpetualConfig.leverageOptions.map { .init(value: $0) }

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

    public static func option(desiredValue: UInt8, from available: [LeverageOption]) -> LeverageOption {
        LeverageOption(
            value: PerpetualConfig.selectLeverage(
                desired: desiredValue,
                options: available.map(\.value),
            ),
        )
    }
}
