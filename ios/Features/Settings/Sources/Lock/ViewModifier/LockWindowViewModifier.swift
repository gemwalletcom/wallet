// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

private struct LockWindowViewModifier: ViewModifier {
    @Environment(\.colorScheme) var colorScheme
    private let lockWindow: LockWindow

    init(lockWindow: LockWindow) {
        self.lockWindow = lockWindow
    }

    func body(content: Content) -> some View {
        content
            .taskOnce(lockWindow.observeScene)
            .onChange(of: colorScheme, initial: true) { _, newColorScheme in
                lockWindow.setColorScheme(newColorScheme)
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
