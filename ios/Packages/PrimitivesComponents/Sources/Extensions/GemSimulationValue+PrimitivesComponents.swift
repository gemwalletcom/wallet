// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSimulationValue
import GemstonePrimitives

public extension GemSimulationValue {
    func map() -> AssetValueHeaderData {
        AssetValueHeaderData(asset: asset.map(), value: value.map())
    }
}
