// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives

struct SimulationAssetChange: Equatable {
    let assetId: AssetId
    let value: BigInt
    let decimals: Int32
    let name: String?
    let symbol: String?
}
