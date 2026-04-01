// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension URLSessionWebSocketTask.Message {
    var data: Data? {
        switch self {
        case let .string(text): text.data(using: .utf8)
        case let .data(data): data
        @unknown default: nil
        }
    }
}
