// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAvatar
import Primitives

public extension GemAvatar {
    var assetImage: AssetImage {
        AssetImage(type: .text(initials), imageURL: imageUrl.map { ImageSource($0).url })
    }
}
