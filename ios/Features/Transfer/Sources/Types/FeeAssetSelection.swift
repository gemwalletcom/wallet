// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

enum FeeAssetSelection: Equatable {
    case automatic
    case selected(AssetId)
}
