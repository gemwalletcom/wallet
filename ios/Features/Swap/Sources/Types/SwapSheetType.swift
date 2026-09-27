// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemInfoSheet
import GemstonePrimitives
import InfoSheet
import Primitives

public enum SwapSheetType: Identifiable, Equatable, Sendable {
    case info(GemInfoSheet)
    case selectAsset(SelectAssetSwapType)
    case swapDetails

    public var id: String {
        switch self {
        case let .info(sheet): "info-\(sheet.hashValue)"
        case let .selectAsset(type): "selectAsset-\(type)"
        case .swapDetails: "details"
        }
    }
}
