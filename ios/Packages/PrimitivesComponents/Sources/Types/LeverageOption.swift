// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPickerOption

public struct LeverageOption: WheelPickerDisplayable, Sendable {
    private let option: GemPickerOption

    public init(option: GemPickerOption) {
        self.option = option
    }

    public var value: UInt8 {
        option.value
    }

    public var id: UInt8 {
        option.value
    }

    public var displayText: String {
        option.label.text
    }
}
