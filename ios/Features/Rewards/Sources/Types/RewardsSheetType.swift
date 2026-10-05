// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import InfoSheet

enum RewardsSheetType: Identifiable {
    case info(InfoSheetModel)
    case walletSelector
    case share
    case createCode
    case activateCode(code: String)
    case url(URL)

    var id: String {
        switch self {
        case .info: "info"
        case .walletSelector: "walletSelector"
        case .share: "share"
        case .createCode: "createCode"
        case .activateCode: "activateCode"
        case let .url(url): "url-\(url)"
        }
    }
}
