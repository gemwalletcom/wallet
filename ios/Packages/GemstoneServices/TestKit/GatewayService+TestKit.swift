// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import GemstonePrimitivesTestKit
import NativeProviderService
import Primitives
import PrimitivesTestKit

public extension GatewayService {
    static func mock() -> GatewayService {
        GatewayService(
            provider: NativeProvider(session: .offline, nodeProvider: NodeURLProviderMock()),
            preferences: GemPreferencesStoreMock(),
            securePreferences: GemSecureStoreMock(),
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
