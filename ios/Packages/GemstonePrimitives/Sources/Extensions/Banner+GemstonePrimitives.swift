// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContext
import struct Gemstone.GemBannerItem
import struct Gemstone.GemBannerKey
import Primitives

public extension Primitives.Banner {
    var gemKey: GemBannerKey {
        get throws {
            GemBannerKey(
                walletId: walletId?.id,
                assetId: asset?.id.identifier,
                event: event.map(),
            )
        }
    }
}

public extension Primitives.BannerActionType {
    var gemAction: GemBannerAction {
        get throws {
            switch self {
            case let .event(event): .event(event: event.map())
            case .button: .button
            case .closeBanner: .close
            }
        }
    }
}

public extension GemBannerContext {
    func visibleBanners(_ banners: [Banner], walletId: WalletId?, asset: Asset?) -> [Banner] {
        let stored = banners.map { GemBannerItem(event: $0.event.map(), state: $0.state.map()) }
        return visibleBanners(stored: stored).map { item in
            let event = item.event.map()
            return banners.first { $0.event == event } ?? Banner(
                walletId: walletId,
                asset: asset,
                event: event,
                state: item.state.map(),
            )
        }
    }
}
