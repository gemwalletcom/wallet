// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

enum FeeAssetSelection: Equatable {
    case automatic
    case selected(AssetId)
}

extension FeeAssetSelection {
    var selectedAssetId: AssetId? {
        switch self {
        case .automatic: nil
        case let .selected(assetId): assetId
        }
    }
}
