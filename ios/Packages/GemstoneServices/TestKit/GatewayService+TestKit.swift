// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import GemstonePrimitivesTestKit
import NativeProviderService
import Primitives

public extension GatewayService {
    static func mock() -> GatewayService {
        GatewayService(
            provider: NativeProvider(session: .offline, url: Constants.apiURL),
            preferences: GemPreferencesStoreMock(),
            securePreferences: GemPreferencesStoreMock(),
        )
    }
}

private extension URLSession {
    static let offline: URLSession = {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [OfflineURLProtocol.self]
        return URLSession(configuration: configuration)
    }()
}

private final class OfflineURLProtocol: URLProtocol {
    override class func canInit(with _: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        client?.urlProtocol(self, didFailWithError: URLError(.notConnectedToInternet))
    }

    override func stopLoading() {}
}
