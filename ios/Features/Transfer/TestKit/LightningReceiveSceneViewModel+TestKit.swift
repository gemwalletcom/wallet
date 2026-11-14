// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemLightningServiceProtocol
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Transfer

public extension LightningReceiveSceneViewModel {
    static func mock(
        service: any GemLightningServiceProtocol = GemLightningServiceMock(),
        walletId: WalletId = .mock(),
        onComplete: @escaping (ToastMessage) -> Void = { _ in },
    ) -> LightningReceiveSceneViewModel {
        LightningReceiveSceneViewModel(service: service, walletId: walletId, onComplete: onComplete)
    }
}
