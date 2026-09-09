// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemStreamServiceProtocol
import GemstonePrimitivesTestKit
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamObserverService {
    static func mock(
        service: any GemStreamServiceProtocol = GemStreamServiceMock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock(),
    ) -> StreamObserverService {
        StreamObserverService(
            service: service,
            webSocket: webSocket,
        )
    }
}
