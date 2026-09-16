// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemVerificationLevel
import Style
import SwiftUI

extension GemVerificationLevel {
    var image: Image {
        switch self {
        case .verified: Images.Transaction.State.success
        case .unverified: Images.TokenStatus.warning
        case .suspicious: Images.TokenStatus.risk
        }
    }

    var textStyle: TextStyle {
        switch self {
        case .verified: TextStyle(font: .callout, color: Colors.green)
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
    }
}
