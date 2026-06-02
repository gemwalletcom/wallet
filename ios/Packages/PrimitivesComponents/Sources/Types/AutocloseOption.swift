// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization

public struct AutocloseOption: WheelPickerDisplayable, Sendable {
    public static var takeProfitOptions: [AutocloseOption] {
        PerpetualConfig.takeProfitOptions.map { .init(value: $0) }
    }

    public static var stopLossOptions: [AutocloseOption] {
        PerpetualConfig.stopLossOptions.map { .init(value: $0) }
    }

    public let value: UInt8

    public init(value: UInt8) {
        self.value = value
    }

    public var id: UInt8 {
        value
    }

    public var displayText: String {
        value == 0 ? Localized.Common.none : "\(value)%"
    }
}
