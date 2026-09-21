// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppLock
import AppLockTestKit
import GemstoneServices
import GemstoneServicesTestKit
import SwiftUI
import Testing

@MainActor
struct LockWindowTests {
    @Test
    func initialization() {
        let manager = LockWindowMock.mock()
        #expect(manager.overlayWindow == nil)
    }

    @Test
    func showLockScreenCreatesWindow() {
        let manager = LockWindowMock.mock()
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.isHidden == false)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissWhileLockedDoesNotRemoveWindow() {
        let manager = LockWindowMock.mock()
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func dismissAfterUnlockHidesWindow() {
        let manager = LockWindowMock.mock()
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
        let manager = LockWindowMock.mock(service: service)
        manager.toggleLock(show: true)

        await manager.lockModel.startUnlock()?.value

        #expect(manager.lockModel.state == .lockedCanceled)
        #expect(manager.lockModel.isUnlockButtonVisible)
        #expect(manager.showLockScreen)
        #expect(manager.overlayWindow?.isHidden == false)
        #expect(manager.overlayWindow?.alpha == 1)
    }

    @Test
    func setPhaseInactiveShowsPlaceholder() {
        let manager = LockWindowMock.mock()
        manager.setPhase(phase: .inactive)
        #expect(manager.showLockScreen)
    }

    @Test
    func setPhaseActiveStartsUnlock() async {
        let manager = LockWindowMock.mock()
        manager.setPhase(phase: .active)

        #expect(manager.lockModel.isUnlocking)
        #expect(manager.showLockScreen)

        await manager.lockModel.startUnlock()?.value
        #expect(manager.lockModel.state == .unlocked)
    }

    @Test
    func backgroundSchedulesAutoLock() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(lockPeriod: .oneMinute))
        manager.lockModel.state = .unlocked
        manager.lockModel.backgroundedAt = nil

        manager.setPhase(phase: .background)

        let elapsed = manager.lockModel.backgroundedAt.map { (ContinuousClock.now - $0).milliseconds } ?? .max

        #expect(elapsed < 1000, "backgrounding stamps the countdown start")
    }

    @Test
    func autoLockDisabledResetsState() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(requiresAuthentication: false))
        manager.lockModel.state = .locked
        manager.setPhase(phase: .active)

        #expect(manager.lockModel.state == .unlocked)
        #expect(!manager.showLockScreen)
    }

    @Test
    func overlayWindowIsReused() {
        let manager = LockWindowMock.mock()
        manager.toggleLock(show: true)
        let first = manager.overlayWindow

        manager.toggleLock(show: true)
        #expect(first === manager.overlayWindow)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchDisabled() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: false))
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func overlayVisibleWhenPrivacySwitchEnabled() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: true))
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func secondPresentKeepsOverlayIfConditionsUnchanged() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(isPrivacyLockEnabled: false))
        manager.toggleLock(show: true)
        manager.toggleLock(show: false)
        manager.toggleLock(show: true)

        #expect(manager.overlayWindow != nil)
        #expect(manager.overlayWindow?.alpha == 1)
        #expect(manager.isPrivacyLockVisible)
    }

    @Test
    func noOverlayWhenAuthenticationDisabled() {
        let manager = LockWindowMock.mock(service: BiometryAuthenticationMock(requiresAuthentication: false, isPrivacyLockEnabled: true))

        #expect(manager.isPrivacyLockVisible == false)
        #expect(manager.overlayWindow == nil)
    }
}
