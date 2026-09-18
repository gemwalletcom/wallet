// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSignMessageService
import struct Gemstone.GemWalletConnectMessageRequest
import GemstoneServicesTestKit
import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit

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
