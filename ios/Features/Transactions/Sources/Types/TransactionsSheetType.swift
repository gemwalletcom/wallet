// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSelectAssetType
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public enum TransactionsSheetType: Identifiable {
    case filter
    case selectAsset(GemSelectAssetType)

    public var id: String {
        switch self {
        case .filter: "filter"
        case let .selectAsset(type): "selectAsset-\(type)"
        }
    }
}
