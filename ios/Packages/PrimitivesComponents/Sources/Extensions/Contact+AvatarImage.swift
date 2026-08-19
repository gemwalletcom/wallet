// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

public extension Contact {
    var avatarImage: AssetImage {
        AssetImage(
            type: .text(String(name.prefix(2))),
            imageURL: imageUrl.map { ImageSource($0).url },
        )
    }
}
