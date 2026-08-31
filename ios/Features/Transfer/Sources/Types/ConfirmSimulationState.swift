// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let result: SimulationResult?
    let warnings: [SimulationWarning]
    let payload: SimulationPayloadModel
    let headerData: AssetValueHeaderData?
    let balanceChanges: [SimulationAssetChange]
}
