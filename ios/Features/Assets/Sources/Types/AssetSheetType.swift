// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

public enum AssetSheetType: Identifiable, Sendable {
    case info(InfoSheetType)
    case transfer(GemTransferData)
    case share
    case url(URL)

    public var id: String {
        switch self {
        case let .info(type): "info-\(type.id)"
        case let .transfer(data): "transfer-\(data.id)"
        case .share: "share"
        case let .url(url): "url-\(url)"
        }
    }
}
