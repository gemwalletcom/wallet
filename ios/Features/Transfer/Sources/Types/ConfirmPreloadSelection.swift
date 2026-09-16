// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemConfirmLoadOptions
import Primitives

struct ConfirmPreloadSelection: Equatable {
    let fee: GemConfirmFeeSelection
    let feeAsset: FeeAssetSelection
    let asset: AssetId?

    var loadOptions: GemConfirmLoadOptions {
        GemConfirmLoadOptions(feeSelection: fee, feeAssetId: feeAsset.selectedAssetId?.identifier, assetId: asset?.identifier)
    }
}
