// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSignMessageService
import struct Gemstone.GemWalletConnectMessageRequest
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import WalletConnector
import WalletConnectorService

public extension SignMessageSceneViewModel {
    @MainActor
    static func mock(request: GemWalletConnectMessageRequest = .mock()) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            service: GemSignMessageService.mock(),
            request: request,
            confirmTransferDelegate: { _ in },
        )
    }
}
