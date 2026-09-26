// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemInfoSheet
import InfoSheet

enum TransactionSheetType: Identifiable {
    case share
    case feeDetails
    case info(GemInfoSheet)

    var id: String {
        switch self {
        case .share: "share"
        case .feeDetails: "feeDetails"
        case let .info(sheet): "info_\(sheet.hashValue)"
        }
    }
}
