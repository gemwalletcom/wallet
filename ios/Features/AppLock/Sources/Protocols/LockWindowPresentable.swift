// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAppLockScreen
import SwiftUI

@MainActor
public protocol LockWindowPresentable: Observable {
    var lockModel: LockSceneViewModel { get }
    var screen: GemAppLockScreen { get }

    func setPhase(_ phase: ScenePhase)
    func setColorScheme(_ colorScheme: ColorScheme)
    func present(_ screen: GemAppLockScreen)
}
