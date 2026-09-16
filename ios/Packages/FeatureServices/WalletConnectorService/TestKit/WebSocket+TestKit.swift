// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import WalletConnectorService

extension WebSocket {
    static func mock() -> WebSocket {
        WebSocket(request: URLRequest(url: URL(string: "wss://example.com")!))
    }
}
