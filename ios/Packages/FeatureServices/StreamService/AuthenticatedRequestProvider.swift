// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceRequestSigner
import GemstonePrimitives
import GemstoneServices
import Preferences
import Primitives
import WebSocketClient

public struct AuthenticatedRequestProvider: WebSocketRequestProvider {
    private let securePreferences: SecurePreferences

    public init(securePreferences: SecurePreferences) {
        self.securePreferences = securePreferences
    }

    public func makeRequest() throws -> URLRequest {
        let keyPair = try securePreferences.getOrCreateDeviceKeyPair()
        let signer = try GemDeviceRequestSigner(privateKey: keyPair.privateKey)
        var request = URLRequest(url: Constants.deviceStreamWebSocketURL)
        request.httpMethod = "GET"
        try signer.sign(request: &request)
        return request
    }
}
