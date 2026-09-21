// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum GemListRowPresentationType: Identifiable, Equatable, Sendable {
    case copy
    case url(URL)

    var id: String {
        switch self {
        case .copy: "copy"
        case let .url(url): "url-\(url)"
        }
    }
}
