// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension [AssetData] {
    func assets(ids: [AssetId]) -> [AssetData] {
        let byId = Dictionary(map { ($0.asset.id, $0) }, uniquingKeysWith: { first, _ in first })
        return ids.compactMap { byId[$0] }
    }
}
