// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct LockWindowViewModifier: ViewModifier {
    @Environment(\.scenePhase) var scenePhase
    @Environment(\.colorScheme) var colorScheme
    private let lockWindow: LockWindow

    init(lockWindow: LockWindow) {
        self.lockWindow = lockWindow
    }

    func body(content: Content) -> some View {
        content
            .onChange(of: scenePhase, initial: true) { _, newPhase in
                lockWindow.lockModel.onScenePhase(newPhase)
            }
            .onChange(of: colorScheme, initial: true) { _, newColorScheme in
                lockWindow.setColorScheme(newColorScheme)
            }
            .onChange(of: lockWindow.isPrivacyLockVisible) { _, visible in
                lockWindow.togglePrivacyLock(visible: visible)
            }
            .onChange(of: lockWindow.showLockScreen, initial: true) { _, showLockScreen in
                lockWindow.toggleLock(show: showLockScreen)
            }
    }
}

public extension View {
    func lockWindow(_ lockWindow: LockWindow) -> some View {
        modifier(LockWindowViewModifier(lockWindow: lockWindow))
    }
}
