// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerContext
import struct Gemstone.GemBannerItem
import protocol Gemstone.GemBannerServiceProtocol
import struct Gemstone.GemBannerKey
import Primitives

public extension Primitives.Banner {
    var gemKey: GemBannerKey {
        get throws {
            GemBannerKey(
                walletId: walletId?.id,
                assetId: asset?.id.identifier,
                event: event.json(),
            )
        }
    }
}

public extension Primitives.BannerActionType {
    var gemAction: GemBannerAction {
        get throws {
            switch self {
            case let .event(event): .event(event: event.json())
            case .button: .button
            case .closeBanner: .close
            }
        }
    }
}

public extension GemBannerServiceProtocol {
    func content(for banner: Banner) -> GemBannerContent {
        bannerContent(event: banner.event.json(), asset: banner.asset?.map())
    }
}

public extension GemBannerContext {
    func visibleBanners(_ banners: [Banner], walletId: WalletId?, asset: Asset?) throws -> [Banner] {
        let stored = banners.map { GemBannerItem(event: $0.event.json(), state: $0.state.json()) }
        return try visibleBanners(stored: stored).map { item in
            let event = try Primitives.BannerEvent(item.event)
            return try banners.first { $0.event == event } ?? Banner(
                walletId: walletId,
                asset: asset,
                event: event,
                state: Primitives.BannerState(item.state),
            )
        }
    }
}
