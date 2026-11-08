// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSecureStore
import func Gemstone.walletConnectRelay
import Primitives
import WebSocketClient

public struct WalletConnectRequestProvider: WebSocketRequestProvider {
    private let store: any GemSecureStore

    public init(store: any GemSecureStore) {
        self.store = store
    }

    public func makeRequest() throws -> URLRequest {
        let relay = try walletConnectRelay(store: store, platform: .ios)
        guard let url = URL(string: relay.url) else {
            throw AnyError("invalid WalletConnect relay url")
        }
        var request = URLRequest(url: url)
        request.setValue(relay.origin, forHTTPHeaderField: "Origin")
        return request
    }
}
