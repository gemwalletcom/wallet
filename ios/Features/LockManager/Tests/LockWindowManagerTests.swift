// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
@testable import LockManager
import SwiftUI
import Testing

@MainActor
struct LockWindowManagerTests {
    @Test
    func initialization() {
        let manager = LockWindowManagerMock.mock()
        #expect(manager.overlayWindow == nil)
    }

    @Test
    func showLockScreenCreatesWindow() {
        let manager = LockWindowManagerMock.mock()
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.isHidden == false)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissWhileLockedDoesNotRemoveWindow() {
        let manager = LockWindowManagerMock.mock()
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissAfterUnlockRemovesWindow() {
        let manager = LockWindowManagerMock.mock()
        manager.toggleLock(show: true)

        manager.lockModel.state = .unlocked
        manager.lockModel.lastUnlockTime = .distantFuture
        manager.toggleLock(show: false)

        #expect(manager.overlayWindow == nil)
    }

    @Test
    func setPhaseInactiveShowsPlaceholder() {
        let manager = LockWindowManagerMock.mock()
        manager.setPhase(phase: .inactive)
        #expect(manager.showLockScreen)
    }

    @Test
    func setPhaseActiveAutoLocks() {
        let manager = LockWindowManagerMock.mock()
        manager.setPhase(phase: .active)
        #expect(manager.lockModel.state == .locked)
    }

    @Test
    func backgroundSchedulesAutoLock() {
        let manager = LockWindowManagerMock.mock(lockPeriod: .oneMinute)
        manager.lockModel.state = .unlocked
        manager.lockModel.lastUnlockTime = .distantFuture

        manager.setPhase(phase: .background)

        let expected = Date().addingTimeInterval(TimeInterval(manager.lockModel.lockPeriod.value))

        #expect(abs(manager.lockModel.lastUnlockTime.timeIntervalSince(expected)) < 1)
    }

    @Test
    func autoLockDisabledResetsState() {
        let manager = LockWindowManagerMock.mock(isAuthEnabled: false)
        manager.lockModel.state = .locked
        manager.setPhase(phase: .active)

        #expect(manager.lockModel.state == .unlocked)
        #expect(!manager.showLockScreen)
    }

    @Test
    func overlayWindowIsReused() {
        let manager = LockWindowManagerMock.mock()
        manager.toggleLock(show: true)
        let first = manager.overlayWindow

        manager.toggleLock(show: true)
        #expect(first === manager.overlayWindow)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchDisabled() {
        let manager = LockWindowManagerMock.mock(isPrivacyLockEnabled: false)
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchEnabled() {
        let manager = LockWindowManagerMock.mock(isPrivacyLockEnabled: true)
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func secondPresentKeepsOverlayIfConditionsUnchanged() {
        let manager = LockWindowManagerMock.mock(isPrivacyLockEnabled: false)
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func noOverlayWhenAuthenticationDisabled() {
        let manager = LockWindowManagerMock.mock(isAuthEnabled: false,
                                                 isPrivacyLockEnabled: true)

        #expect(manager.isPrivacyLockVisible == false)
        #expect(manager.overlayWindow == nil)
    }
}
