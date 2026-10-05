// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import InfoSheet
import Primitives

public enum SwapSheetType: Identifiable, Sendable {
    case info(InfoSheetModel)
    case selectAsset(SelectAssetSwapType)
    case swapDetails

    public var id: String {
        switch self {
        case let .info(sheet): "info-\(sheet.id)"
        case let .selectAsset(type): "selectAsset-\(type)"
        case .swapDetails: "details"
        }
    }
}
