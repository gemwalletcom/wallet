// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemFeeAsset
import Primitives

struct ConfirmTransferData {
    let preload: ConfirmTransferPreload
    let simulation: ConfirmSimulationState
    let feeAssets: [GemFeeAsset]
    let addressName: AddressName?
}
