// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum GemListRowPresentationType: Identifiable, Equatable {
    case url(URL)

    var id: String {
        switch self {
        case let .url(url): "url-\(url)"
        }
    }
}
