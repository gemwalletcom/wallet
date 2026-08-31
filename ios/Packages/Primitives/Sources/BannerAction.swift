// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct BannerAction: Identifiable, Sendable {
    public let banner: Banner
    public let type: BannerActionType
    public let url: URL?

    public var id: String { banner.id }

    public init(
        banner: Banner,
        type: BannerActionType,
        url: URL?,
    ) {
        self.banner = banner
        self.type = type
        self.url = url
    }
}

public enum BannerActionType: Sendable {
    case event(BannerEvent)
    case button(BannerButton)
    case closeBanner
}

public enum BannerButton: String, Sendable {
    case buy
    case receive
}
