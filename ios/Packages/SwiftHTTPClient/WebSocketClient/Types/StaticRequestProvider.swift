// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct StaticRequestProvider: WebSocketRequestProvider {
    private let request: URLRequest

    init(url: URL) {
        request = URLRequest(url: url)
    }

    func makeRequest() -> URLRequest {
        request
    }
}
