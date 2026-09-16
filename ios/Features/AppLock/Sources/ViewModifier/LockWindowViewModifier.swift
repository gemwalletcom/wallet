// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct LockWindowViewModifier: ViewModifier {
    @Environment(\.scenePhase) var scenePhase
    @Environment(\.colorScheme) var colorScheme
    private let lockWindow: any LockWindowPresentable

    init(lockWindow: any LockWindowPresentable) {
        self.lockWindow = lockWindow
    }

    func body(content: Content) -> some View {
        content
            .onChange(of: scenePhase, initial: true) { _, newPhase in
                lockWindow.setPhase(newPhase)
            }
            .onChange(of: colorScheme, initial: true) { _, newColorScheme in
                lockWindow.setColorScheme(newColorScheme)
            }
            .onChange(of: lockWindow.screen, initial: true) { _, screen in
                lockWindow.present(screen)
            }
    }
}

public extension View {
    func lockWindow(_ lockWindow: any LockWindowPresentable) -> some View {
        modifier(LockWindowViewModifier(lockWindow: lockWindow))
    }
}
