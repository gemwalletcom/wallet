// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerButton
import enum Gemstone.GemBannerDestination
import struct Gemstone.GemBannerKey

public struct BannerAction: Identifiable, Sendable {
    public let id: String
    public let key: GemBannerKey
    public let type: BannerActionType

    public init(
        key: GemBannerKey,
        type: BannerActionType,
    ) {
        id = key.identifier()
        self.key = key
        self.type = type
    }
}

public enum BannerActionType: Sendable {
    case destination(GemBannerDestination)
    case button(GemBannerButton)
    case closeBanner
}
