// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension NFTData {
    static func mock(
        collection: NFTCollection = .mock(),
        assets: [NFTAsset] = [.mock()],
    ) -> NFTData {
        NFTData(
            collection: collection,
            assets: assets,
        )
    }
}
