// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContext
import struct Gemstone.GemBannerKey
import Primitives

public extension Primitives.Banner {
    var gemKey: GemBannerKey {
        GemBannerKey(
            walletId: walletId?.id,
            assetId: asset?.id.identifier,
            event: event.map(),
        )
    }
}

public extension Primitives.BannerActionType {
    var gemAction: GemBannerAction {
        switch self {
        case let .event(event): .event(event: event.map())
        case .button: .button
        case .closeBanner: .close
        }
    }
}
