// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPriceService
import protocol Gemstone.GemPriceServiceProtocol
import Foundation
import GemstoneServices
import GemstoneServicesTestKit
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamSubscriptionService {
    static func mock(
        priceService: any GemPriceServiceProtocol = GemPriceService.mock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock(),
    ) -> StreamSubscriptionService {
        StreamSubscriptionService(
            priceService: priceService,
            walletSessionService: walletSessionService,
            webSocket: webSocket,
        )
    }
}
