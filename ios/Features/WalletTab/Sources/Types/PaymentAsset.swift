// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

enum PaymentAsset {
    case unsupported
    case single(AssetData)
    case choice([AssetData])
}
