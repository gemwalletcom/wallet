// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import GemstoneServicesTestKit
@testable import Settings
import SettingsTestKit
import SwiftUI
import Testing

@MainActor
struct LockWindowTests {
    @Test
    func initialization() {
        let manager = LockWindow.mock()
        #expect(manager.overlayWindow == nil)
    }

    @Test
    func showLockScreenCreatesWindow() {
        let manager = LockWindow.mock()
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.isHidden == false)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissWhileLockedDoesNotRemoveWindow() {
        let manager = LockWindow.mock()
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissAfterUnlockHidesWindow() {
        let manager = LockWindow.mock()
        manager.toggleLock(show: true)

        manager.lockModel.state = .unlocked
        manager.lockModel.backgroundedAt = nil
        manager.toggleLock(show: false)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 0)
        #expect(manager.overlayWindow?.isHidden == true)
    }

    @Test
    func interruptedUnlockKeepsLockVisible() async {
        let service = BiometryAuthenticationMock()
        service.authenticateError = BiometryAuthenticationError.cancelledBySystem
        let manager = LockWindow.mock(service: service)
        manager.toggleLock(show: true)

        await manager.lockModel.startUnlock()?.value

        #expect(manager.lockModel.state == .lockedCanceled)
        #expect(manager.lockModel.isUnlockButtonVisible)
        #expect(manager.showLockScreen)
        #expect(manager.overlayWindow?.isHidden == false)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func overlayWindowIsReused() {
        let manager = LockWindow.mock()
        manager.toggleLock(show: true)
        let first = manager.overlayWindow

        manager.toggleLock(show: true)
        #expect(first === manager.overlayWindow)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchDisabled() {
        let manager = LockWindow.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: false))
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchEnabled() {
        let manager = LockWindow.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: true))
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func secondPresentKeepsOverlayIfConditionsUnchanged() {
        let manager = LockWindow.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: false))
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func noOverlayWhenAuthenticationDisabled() {
        let manager = LockWindow.mock(service: BiometryAuthenticationMock(requiresAuthentication: false, isPrivacyLockEnabled: true))

        #expect(manager.isPrivacyLockVisible == false)
        #expect(manager.overlayWindow == nil)
    }
}
