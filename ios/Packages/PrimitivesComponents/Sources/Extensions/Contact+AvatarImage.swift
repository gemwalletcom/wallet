// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

public extension Contact {
    func avatarImage(initials: String) -> AssetImage {
        AssetImage(
            type: .text(initials),
            imageURL: imageUrl.map { ImageSource($0).url },
        )
    }
}
