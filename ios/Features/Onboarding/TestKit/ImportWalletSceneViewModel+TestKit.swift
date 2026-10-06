// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemWalletService
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import Onboarding
import Primitives
import PrimitivesTestKit

extension ImportWalletSceneViewModel {
    static func mock(
        service: GemWalletService = .mock(),
        nameService: any GemNameServiceProtocol = GemNameServiceMock(nameRecord: .mock()),
        onComplete: VoidAction = nil,
    ) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(
            service: service,
            nameService: nameService,
            type: .chain(.ethereum),
            onComplete: onComplete,
        )
    }
}
