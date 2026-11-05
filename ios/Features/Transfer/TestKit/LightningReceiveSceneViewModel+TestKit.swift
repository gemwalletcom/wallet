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
        biometry: any BiometryAuthenticatable = BiometryAuthenticationMock(),
        wallet: Wallet = .mock(accounts: [.mock(chain: .lightning)]),
        onComplete: @escaping (ToastMessage) -> Void = { _ in },
    ) -> LightningReceiveSceneViewModel {
        LightningReceiveSceneViewModel(service: service, biometry: biometry, wallet: wallet, onComplete: onComplete)
    }
}
