// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSignMessageService
import GemstoneServicesTestKit
import WalletConnector
import WalletConnectorService

public extension AuthenticationSceneViewModel {
    @MainActor
    static func mock(request: WCAuthenticationRequest) -> AuthenticationSceneViewModel {
        AuthenticationSceneViewModel(
            service: GemSignMessageService.mock(),
            request: request,
            confirmTransferDelegate: { _ in },
        )
    }
}
