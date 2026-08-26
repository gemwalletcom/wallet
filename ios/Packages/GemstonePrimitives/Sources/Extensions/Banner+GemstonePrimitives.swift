// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerKey
import Primitives

public extension Primitives.Banner {
    var gemKey: GemBannerKey {
        get throws {
            try GemBannerKey(
                walletId: wallet?.id.id,
                assetId: asset?.id.identifier,
                chain: chain?.rawValue,
                event: event.json(),
            )
        }
    }
}

public extension Primitives.BannerActionType {
    var gemAction: GemBannerAction {
        get throws {
            switch self {
            case let .event(event): try .event(event: event.json())
            case .button: .button
            case .closeBanner: .close
            }
        }
    }
}
