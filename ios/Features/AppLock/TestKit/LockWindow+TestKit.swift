// Copyright (c). Gem Wallet. All rights reserved.

import AppLock
import GemstoneServices
import GemstoneServicesTestKit
import UIKit

public extension LockWindow {
    @MainActor
    static func mock(service: any BiometryAuthenticatable = BiometryAuthenticationMock()) -> LockWindow {
        LockWindow(
            lockModel: LockSceneViewModel(service: service),
            sceneWindow: { UIWindow(frame: .zero) },
        )
    }
}
