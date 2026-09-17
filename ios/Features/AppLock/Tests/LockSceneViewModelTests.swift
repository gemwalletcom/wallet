// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Foundation
import GemstoneServices
import GemstoneServicesTestKit
import LocalAuthentication
@testable import AppLock

extension LockSceneViewModel {
    var isUnlocking: Bool {
        if case .unlocking = state { true } else { false }
    }
}
import Testing

@MainActor
struct LockSceneViewModelTests {
    @Test
    func initializationWhenAuthEnabled() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        #expect(viewModel.state == .locked)
        #expect(viewModel.isPrivacyLockVisible)
    }

    @Test
    func initializationWhenAuthDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isPrivacyLockVisible)
    }

    @Test
    func unlockSuccess() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .unlocked)
        #expect(viewModel.shouldShowLockScreen == false)
        #expect(viewModel.backgroundedAt == nil)
        #expect(mockService.authenticateCallsCount == 1)
    }

    @Test
    func userCancelledUnlockShowsUnlockButton() async {
        let mockService = BiometryAuthenticationMock()
        mockService.authenticateError = BiometryAuthenticationError.cancelledByUser
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
        #expect(viewModel.shouldShowLockScreen)
    }

    @Test
    func systemCancelledUnlockWithoutBackgroundingShowsUnlockButton() async {
        let mockService = BiometryAuthenticationMock()
        mockService.authenticateError = BiometryAuthenticationError.cancelledBySystem
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func lockedStateDoesNotShowUnlockButton() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        #expect(viewModel.state == .locked)
        #expect(!viewModel.isUnlockButtonVisible)
    }

    @Test
    func failedUnlockShowsUnlockButton() async {
        let mockService = BiometryAuthenticationMock()
        mockService.authenticateError = BiometryAuthenticationError.authenticationFailed
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func biometryUnavailableShowsUnlockButton() async {
        let mockService = BiometryAuthenticationMock()
        mockService.authenticateError = BiometryAuthenticationError.biometryUnavailable
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func unexpectedErrorShowsUnlockButton() async {
        let mockService = BiometryAuthenticationMock()
        mockService.authenticateError = NSError(domain: "TestError", code: 999, userInfo: nil)
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func startUnlockJoinsAttemptInFlight() async {
        let mockService = BiometryAuthenticationMock()
        mockService.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: mockService)

        let first = viewModel.startUnlock()
        #expect(viewModel.isUnlocking)
        let second = viewModel.startUnlock()

        mockService.releaseAuthentication()
        await first?.value
        await second?.value

        #expect(mockService.authenticateCallsCount == 1)
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func backgroundInterruptionRepromptsOnNextActivation() async {
        let mockService = BiometryAuthenticationMock()
        mockService.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: mockService)

        let first = viewModel.startUnlock()
        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)

        mockService.authenticateError = BiometryAuthenticationError.cancelledBySystem
        mockService.releaseAuthentication()
        await first?.value
        #expect(viewModel.state == .locked)

        mockService.authenticateError = nil
        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.isUnlocking)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 2)
    }

    @Test
    func staleAttemptIsReplacedOnActivationAndLateResultIgnored() async {
        let mockService = BiometryAuthenticationMock()
        mockService.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: mockService)

        let first = viewModel.startUnlock()
        await Task.yield()
        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.isUnlocking)
        let second = viewModel.startUnlock()
        await Task.yield()
        #expect(mockService.authenticateCallsCount == 2)

        mockService.releaseNextAuthentication()
        await first?.value
        #expect(viewModel.isUnlocking, "a stale result must not apply after the attempt was replaced")

        mockService.releaseAuthentication()
        await second?.value
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func inactiveBlipDoesNotInterruptAttemptInFlight() async {
        let mockService = BiometryAuthenticationMock()
        mockService.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: mockService)

        let task = viewModel.startUnlock()
        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.isUnlocking)

        mockService.releaseAuthentication()
        await task?.value

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 1)
    }

    @Test
    func activationStartsUnlockWhenLocked() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.isUnlocking)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 1)
    }

    @Test
    func activationDoesNotRetryAfterUserCancel() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .lockedCanceled

        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .lockedCanceled)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func activationLocksAndStartsUnlockWhenGracePeriodExpired() async {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)

        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.isUnlocking)
        #expect(viewModel.shouldShowLockScreen)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func activationKeepsUnlockedWithinGracePeriod() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func gracePeriodExtendedDuringBackgrounding() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now

        viewModel.handleSceneChange(to: .background)

        let elapsed = viewModel.backgroundedAt.map { (ContinuousClock.now - $0).milliseconds } ?? .max
        #expect(elapsed < 1000, "backgrounding inside the grace period restarts the countdown")
        #expect(viewModel.shouldLock == false)
    }

    @Test
    func togglingAuthOffResetsViewModel() async throws {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.handleSceneChange(to: .inactive) // showPlaceholderPreview true

        try await mockService.enableAuthentication(false, reason: "unit")
        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
    }

    @Test
    func changingLockPeriodDoesNotTriggerImmediateLock() throws {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = nil

        try mockService.update(period: .fiveMinutes)

        #expect(viewModel.lockPeriod == .fiveMinutes)
        #expect(viewModel.backgroundedAt == nil)
    }

    @Test
    func isLockedProperty() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.state = .unlocked
        #expect(!viewModel.isLocked)

        viewModel.state = .locked
        #expect(viewModel.isLocked)

        viewModel.state = .unlocking(UnlockAttempt(context: LAContext(), task: Task {}))
        #expect(viewModel.isLocked)
    }

    @Test
    func isLockedPropertyWhenAuthDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.state = .unlocked
        #expect(!viewModel.isLocked)

        viewModel.state = .locked
        #expect(!viewModel.isLocked)
    }

    @Test
    func shouldShowLockScreen() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.state = .unlocked
        #expect(!viewModel.shouldShowLockScreen)

        viewModel.state = .locked
        #expect(viewModel.shouldShowLockScreen)

        viewModel.state = .unlocked
        viewModel.handleSceneChange(to: .inactive)
        #expect(viewModel.shouldShowLockScreen)
    }

    @Test
    func handleSceneChangeWhenAutoLockDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked

        viewModel.handleSceneChange(to: .background)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)

        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
    }

    @Test
    func startUnlockWhenAutoLockDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        let task = viewModel.startUnlock()

        #expect(task == nil)
        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func shouldLockWhenAutoLockEnabled() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)
        #expect(viewModel.shouldLock)

        viewModel.backgroundedAt = ContinuousClock.now
        #expect(!viewModel.shouldLock)
    }

    @Test
    func shouldLockWhenAutoLockDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)
        #expect(!viewModel.shouldLock)

        viewModel.backgroundedAt = ContinuousClock.now
        #expect(!viewModel.shouldLock)
    }

    @Test
    func rapidSceneChanges() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)
        viewModel.handleSceneChange(to: .background)

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldLock)
    }

    @Test
    func resetLockState() {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
        #expect(!viewModel.isLocked)
        #expect(viewModel.backgroundedAt == nil)
    }

    @Test
    func waitUntilUnlockedReturnsWhenAlreadyUnlocked() async {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.waitUntilUnlocked()

        #expect(viewModel.state == .unlocked)
    }

    @Test
    func waitUntilUnlockedResumesAfterUnlock() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        let waiter = Task { await viewModel.waitUntilUnlocked() }

        viewModel.startUnlock()
        await waiter.value

        #expect(viewModel.state == .unlocked)
    }
}
