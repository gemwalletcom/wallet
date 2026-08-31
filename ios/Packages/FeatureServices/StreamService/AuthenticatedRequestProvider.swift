// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceKeyService
import class Gemstone.GemDeviceRequestSigner
import GemstonePrimitives
import GemstoneServices
import Preferences
import Primitives
import WebSocketClient

public struct AuthenticatedRequestProvider: WebSocketRequestProvider {
    private let deviceKeyService: GemDeviceKeyService

    public init(deviceKeyService: GemDeviceKeyService) {
        self.deviceKeyService = deviceKeyService
    }

    public func makeRequest() throws -> URLRequest {
        let signer = try GemDeviceRequestSigner(privateKey: deviceKeyService.keyPair().privateKey)
        var request = URLRequest(url: Constants.deviceStreamWebSocketURL)
        request.httpMethod = "GET"
        try signer.sign(request: &request)
        return request
    }
}
