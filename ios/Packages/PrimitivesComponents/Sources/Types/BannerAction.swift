// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerDestination
import Primitives

public struct BannerAction: Identifiable, Sendable {
    public let banner: Banner
    public let type: BannerActionType

    public var id: String { banner.id }

    public init(
        banner: Banner,
        type: BannerActionType,
    ) {
        self.banner = banner
        self.type = type
    }
}

public enum BannerActionType: Sendable {
    case destination(GemBannerDestination)
    case button(BannerButton)
    case closeBanner
}

public enum BannerButton: String, Sendable {
    case buy
    case receive
}
