// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

public extension SimpleAccount {
    static func mock(
        name: String? = "Alice",
        assetImage: AssetImage? = nil,
    ) -> SimpleAccount {
        SimpleAccount(
            name: name,
            chain: .ethereum,
            address: "0x123456789101112",
            assetImage: assetImage,
        )
    }
}
