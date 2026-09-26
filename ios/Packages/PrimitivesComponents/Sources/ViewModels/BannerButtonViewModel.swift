// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerButton
import struct Gemstone.GemBannerKey
import Primitives
import Style

struct BannerButtonViewModel: Identifiable {
    let button: GemBannerButton
    let key: GemBannerKey

    var id: String {
        String(describing: button)
    }

    var title: String {
        button.title
    }

    @MainActor
    var style: ColorButtonStyle {
        button.style
    }

    var action: BannerAction {
        BannerAction(key: key, type: .button(button))
    }
}
