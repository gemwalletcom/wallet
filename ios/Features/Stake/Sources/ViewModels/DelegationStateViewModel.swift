// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemDelegationStatus
import Style
import SwiftUI

public struct DelegationStateViewModel {
    private let status: GemDelegationStatus

    public init(status: GemDelegationStatus) {
        self.status = status
    }

    public var title: String {
        status.state.title
    }

    public var color: Color {
        status.tone.color
    }

    public var textStyle: TextStyle {
        TextStyle(font: .callout, color: color)
    }
}
