// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

public extension Contact {
    var avatarImage: AssetImage {
        AssetImage(type: avatar.map(AssetImageType.emoji) ?? .text(String(name.prefix(2))))
    }
}
