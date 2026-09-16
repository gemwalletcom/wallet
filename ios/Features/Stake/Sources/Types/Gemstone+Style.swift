// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationTone
import Style
import SwiftUI

extension GemDelegationTone {
    var color: Color {
        switch self {
        case .positive: Colors.green
        case .pending: Colors.orange
        case .negative: Colors.red
        }
    }
}
