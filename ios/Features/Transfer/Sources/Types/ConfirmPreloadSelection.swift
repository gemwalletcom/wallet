// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmFeeSelection
import Primitives

struct ConfirmPreloadSelection: Equatable {
    let fee: GemConfirmFeeSelection
    let feeAsset: FeeAssetSelection
}
