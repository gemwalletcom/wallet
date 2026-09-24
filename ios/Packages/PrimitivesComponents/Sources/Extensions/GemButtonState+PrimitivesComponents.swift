// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemButtonState
import Style

public extension GemButtonState {
    var state: ButtonState {
        switch self {
        case .disabled: .disabled
        case .loading: .loading(showProgress: true)
        case .enabled: .normal
        }
    }
}
