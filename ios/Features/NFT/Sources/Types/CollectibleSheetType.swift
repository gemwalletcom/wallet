// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemInfoSheet

enum CollectibleSheetType: Identifiable {
    case report
    case info(GemInfoSheet)

    var id: String {
        switch self {
        case .report: "report"
        case let .info(sheet): "info-\(sheet.hashValue)"
        }
    }
}
