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
        let stream = try GemDeviceRequestSigner(privateKey: deviceKeyService.keyPair().privateKey).deviceStreamRequest()
        guard let url = URL(string: stream.url) else {
            throw AnyError("invalid device stream url: \(stream.url)")
        }
        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue(stream.authorization, forHTTPHeaderField: "Authorization")
        return request
    }
}
