// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WebSocketClient

public struct WebSocketRequestProviderMock: WebSocketRequestProvider {
    private let request: URLRequest

    public init(url: URL) {
        request = URLRequest(url: url)
    }

    public func makeRequest() -> URLRequest {
        request
    }
}
