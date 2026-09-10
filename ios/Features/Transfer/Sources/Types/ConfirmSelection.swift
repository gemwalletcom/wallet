// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmFeeSelection
import Primitives

struct ConfirmSelection: Equatable {
    let fee: GemConfirmFeeSelection
    let feeAsset: FeeAssetSelection
    let asset: AssetId?
}
