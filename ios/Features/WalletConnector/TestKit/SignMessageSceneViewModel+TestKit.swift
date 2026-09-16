// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSignMessageService
import GemstoneServicesTestKit
import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension SignMessageSceneViewModel {
    @MainActor
    static func mock(payload: SignMessagePayload = .mock()) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            service: GemSignMessageService.mock(),
            payload: payload,
            confirmTransferDelegate: { _ in },
        )
    }
}
