// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum RewardsSheetType: Identifiable, Sendable {
    case walletSelector
    case share
    case createCode
    case activateCode(code: String)
    case url(URL)

    var id: String {
        switch self {
        case .walletSelector: "walletSelector"
        case .share: "share"
        case .createCode: "createCode"
        case .activateCode: "activateCode"
        case let .url(url): "url-\(url)"
        }
    }
}
