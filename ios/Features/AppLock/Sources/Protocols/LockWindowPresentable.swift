// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@MainActor
public protocol LockWindowPresentable: Observable {
    var lockModel: LockSceneViewModel { get }
    var overlayWindow: UIWindow? { get }

    var showLockScreen: Bool { get }
    var isPrivacyLockVisible: Bool { get }

    func setPhase(phase: ScenePhase)
    func setColorScheme(_ colorScheme: ColorScheme)
    func toggleLock(show: Bool)
    func togglePrivacyLock(visible: Bool)
}
