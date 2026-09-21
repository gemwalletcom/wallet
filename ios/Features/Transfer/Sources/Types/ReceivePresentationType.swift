// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum ReceivePresentationType: Identifiable {
    case share
    case networkSelector
    case copy

    var id: String {
        switch self {
        case .share: "share"
        case .networkSelector: "network-selector"
        case .copy: "copy"
        }
    }
}
