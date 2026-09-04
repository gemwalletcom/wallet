// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContent
import protocol Gemstone.GemBannerServiceProtocol
import Primitives

public extension GemBannerServiceProtocol {
    func content(for banner: Banner) -> GemBannerContent {
        bannerContent(event: banner.event.json(), asset: banner.asset?.map())
    }
}
