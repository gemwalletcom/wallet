// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation
import Primitives

public extension OnstartAsyncService {
    static func mock(runners: [any AsyncRunnable] = []) -> OnstartAsyncService {
        OnstartAsyncService(runners: runners)
    }
}
