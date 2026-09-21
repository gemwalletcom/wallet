// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemPickerOption
import Components

extension Gemstone.GemPickerOption: @retroactive Identifiable {}

extension Gemstone.GemPickerOption: WheelPickerDisplayable {
    public var id: UInt8 {
        value
    }

    public var displayText: String {
        label.text
    }
}
