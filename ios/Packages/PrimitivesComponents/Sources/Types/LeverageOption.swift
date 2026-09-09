// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemPerpetual
import enum Gemstone.PerpetualProvider

public struct LeverageOption: WheelPickerDisplayable, Sendable {
    public static let allOptions: [LeverageOption] = options(maxLeverage: nil)

    public static func options(maxLeverage: UInt8?) -> [LeverageOption] {
        GemPerpetual(provider: .hypercore).leverageOptions(maxLeverage: maxLeverage).map { .init(value: $0) }
    }

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
}
