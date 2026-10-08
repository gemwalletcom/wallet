// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import GemstoneServicesTestKit
import LocalAuthentication
import os
import Primitives
@testable import Settings

extension LockSceneViewModel {
    var isUnlocking: Bool {
        if case .unlocking = state {
            true
        } else {
            false
        }
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
        #expect(viewModel.isLocked)
    }

    @Test
    func initializationWhenAuthDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)
    }

    @Test
    func unlockSuccess() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)
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
        #expect(viewModel.isLocked)
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
    func passcodeOffExplainsWithoutPrompt() {
        let mockService = BiometryAuthenticationMock(isPasscodeSet: false)
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.onScenePhase(.active)

        #expect(viewModel.state == .passcodeOff)
        #expect(viewModel.isLocked)
        #expect(!viewModel.isUnlockButtonVisible)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func passcodeTurnedBackOnStartsUnlockOnActivation() async {
        let mockService = BiometryAuthenticationMock(isPasscodeSet: false)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.onScenePhase(.active)

        mockService.isPasscodeSet = true
        viewModel.onScenePhase(.active)
        #expect(viewModel.isUnlocking)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 1)
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
        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.background)

        mockService.authenticateError = BiometryAuthenticationError.cancelledBySystem
        mockService.releaseAuthentication()
        await first?.value
        #expect(viewModel.state == .locked)

        mockService.authenticateError = nil
        viewModel.onScenePhase(.active)
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
        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.background)
        viewModel.onScenePhase(.active)

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
        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.active)
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

        viewModel.onScenePhase(.active)
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

        viewModel.onScenePhase(.active)

        #expect(viewModel.state == .lockedCanceled)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func activationLocksAndStartsUnlockWhenGracePeriodExpired() async {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)

        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.background)
        viewModel.onScenePhase(.active)

        #expect(viewModel.isUnlocking)
        #expect(viewModel.isLocked)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func activationKeepsUnlockedWithinGracePeriod() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now

        viewModel.onScenePhase(.background)
        viewModel.onScenePhase(.active)

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func timeInTheAppAfterAShortTripOutDoesNotCountTowardTheLockPeriod() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now - .seconds(20)

        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.active)

        #expect(viewModel.state == .unlocked)
        #expect(viewModel.backgroundedAt == nil)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func gracePeriodExtendedDuringBackgrounding() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now

        viewModel.onScenePhase(.background)

        let elapsed = viewModel.backgroundedAt.map { (ContinuousClock.now - $0).milliseconds } ?? .max
        #expect(elapsed < 1000, "backgrounding inside the grace period restarts the countdown")
        #expect(viewModel.shouldLock == false)
    }

    @Test
    func togglingAuthOffResetsViewModel() async throws {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.onScenePhase(.inactive)

        try await mockService.enableAuthentication(false, reason: "unit")
        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)
    }

    @Test
    func changingLockPeriodDoesNotTriggerImmediateLock() throws {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = nil

        try mockService.update(period: .fiveMinutes)

        #expect(viewModel.state == .unlocked)
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
    func returningAfterTheLockPeriodCoversTheAppBeforeItIsActive() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)

        viewModel.onScenePhase(.inactive)

        #expect(viewModel.state == .locked)
        #expect(viewModel.isLocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func inactiveWithinTheLockPeriodKeepsTheAppUnlocked() {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked

        viewModel.onScenePhase(.inactive)

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)
    }

    @Test
    func onScenePhaseWhenAutoLockDisabled() {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        viewModel.onScenePhase(.inactive)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)

        viewModel.onScenePhase(.active)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isLocked)
        #expect(mockService.authenticateCallsCount == 0)

        viewModel.onScenePhase(.inactive)
        #expect(viewModel.obscureTask == nil)
        viewModel.onScenePhase(.background)
        #expect(!viewModel.isCovered)
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

        viewModel.onScenePhase(.background)
        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.background)

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

    @Test
    func leavingTheAppCoversItAfterAMoment() async {
        let viewModel = LockSceneViewModel(service: BiometryAuthenticationMock())
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        #expect(!viewModel.isCovered)

        await viewModel.obscureTask?.value
        #expect(viewModel.isCovered)
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func aBriefInterruptionDoesNotCover() async {
        let viewModel = LockSceneViewModel(service: BiometryAuthenticationMock())
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        let obscureTask = viewModel.obscureTask
        viewModel.onScenePhase(.active)
        await obscureTask?.value

        #expect(!viewModel.isCovered)
    }

    @Test
    func aSystemPromptKeepsTheCoverDownUntilTheAppIsActive() async {
        let mockService = BiometryAuthenticationMock()
        mockService.presentedSystemPrompt = true
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        await viewModel.obscureTask?.value

        #expect(!viewModel.isCovered)
    }

    @Test
    func turningTheLockOffWhileInactiveNeverCovers() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        mockService.requiresAuthentication = false
        await viewModel.obscureTask?.value

        #expect(!viewModel.isCovered)
    }

    @Test
    func returningAndLeavingAgainCoversAfterAMoment() async {
        let mockService = BiometryAuthenticationMock()
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.inactive)
        await viewModel.obscureTask?.value

        #expect(viewModel.isCovered)
    }

    @Test
    func goingToTheBackgroundCoversAtOnceEvenDuringASystemPrompt() {
        let mockService = BiometryAuthenticationMock()
        mockService.presentedSystemPrompt = true
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)

        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.background)

        #expect(viewModel.isCovered)
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func returningWithinTheLockPeriodUncoversOnceActive() {
        let viewModel = LockSceneViewModel(service: BiometryAuthenticationMock(lockPeriod: .oneMinute))
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.background)

        viewModel.onScenePhase(.inactive)
        #expect(viewModel.isCovered)

        viewModel.onScenePhase(.active)
        #expect(!viewModel.isCovered)
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func returningAfterTheLockPeriodStaysCoveredUntilUnlocked() async {
        let mockService = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.background)
        viewModel.backgroundedAt = ContinuousClock.now - .seconds(3600)

        viewModel.onScenePhase(.inactive)
        viewModel.onScenePhase(.active)
        #expect(viewModel.isUnlocking)
        #expect(viewModel.isCovered)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isCovered)
    }

    @Test
    func theCoverIsObservedWhenAuthenticationIsTurnedOnLater() async {
        let mockService = BiometryAuthenticationMock(requiresAuthentication: false, availableAuthentication: .none)
        let viewModel = LockSceneViewModel(service: mockService)
        let changed = OSAllocatedUnfairLock(initialState: false)
        withObservationTracking {
            _ = viewModel.isCovered
        } onChange: {
            changed.withLock { $0 = true }
        }

        mockService.requiresAuthentication = true
        viewModel.onScenePhase(.active)
        viewModel.onScenePhase(.inactive)
        await viewModel.obscureTask?.value

        #expect(changed.withLock { $0 })
        #expect(viewModel.isCovered)
    }
}
