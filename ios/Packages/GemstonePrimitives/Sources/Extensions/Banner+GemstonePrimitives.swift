// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerAction
import struct Gemstone.GemBannerContext
import struct Gemstone.GemBannerItem
import protocol Gemstone.GemBannerServiceProtocol
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

public extension GemBannerServiceProtocol {
    func visibleBanners(_ banners: [Banner], wallet: Wallet?, asset: Asset?, context: GemBannerContext) throws -> [Banner] {
        let stored = try banners.map { try GemBannerItem(event: $0.event.json(), state: $0.state.json()) }
        return try visibleBanners(stored: stored, context: context).map { item in
            let event = try Primitives.BannerEvent(item.event)
            return try banners.first { $0.event == event } ?? Banner(
                wallet: wallet,
                asset: asset,
                chain: .none,
                event: event,
                state: Primitives.BannerState(item.state),
            )
        }
    }
}
