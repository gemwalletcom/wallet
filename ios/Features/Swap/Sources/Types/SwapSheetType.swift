// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSwapSide
import GemstonePrimitives
import InfoSheet
import Primitives

public enum SwapSheetType: Identifiable, Sendable {
    case info(InfoSheetModel)
    case selectAsset(GemSwapSide)
    case swapDetails

    public var id: String {
        switch self {
        case let .info(sheet): "info-\(sheet.id)"
        case let .selectAsset(side): "selectAsset-\(side)"
        case .swapDetails: "details"
        }
    }
}
