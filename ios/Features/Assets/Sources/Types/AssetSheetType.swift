// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemInfoSheet
import struct Gemstone.GemTransferData
import GemstonePrimitives
import InfoSheet
import Primitives

public enum AssetSheetType: Identifiable, Sendable {
    case info(GemInfoSheet)
    case transfer(GemTransferData)
    case share
    case url(URL)

    public var id: String {
        switch self {
        case let .info(sheet): "info-\(sheet.hashValue)"
        case let .transfer(data): "transfer-\(data.id)"
        case .share: "share"
        case let .url(url): "url-\(url)"
        }
    }
}
