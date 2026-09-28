// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAvatar
@testable import Primitives
import PrimitivesComponents
import Testing

struct GemAvatarTests {
    @Test
    func aContactShowsItsPictureOrTheInitialsCoreWrote() {
        #expect(GemAvatar(imageUrl: "avatar.png", initials: "AD").assetImage.imageURL == ImageSource("avatar.png").url)
        #expect(GemAvatar(imageUrl: nil, initials: "AD").assetImage.imageURL == nil)
        #expect(GemAvatar(imageUrl: nil, initials: "AD").assetImage.type == .text("AD"))
    }
}
