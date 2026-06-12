// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import LocalAuthentication
@testable import LockManager
import Testing

@MainActor
struct LockSceneViewModelTests {
    @Test
    func initializationWhenAuthEnabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        #expect(viewModel.state == .locked)
        #expect(viewModel.isPrivacyLockVisible)
    }

    @Test
    func initializationWhenAuthDisabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.isPrivacyLockVisible)
    }

    @Test
    func unlockSuccess() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .unlocked)
        #expect(viewModel.shouldShowLockScreen == false)
        #expect(viewModel.lastUnlockTime == .distantFuture)
        #expect(mockService.authenticateCallsCount == 1)
    }

    @Test
    func userCancelledUnlockShowsUnlockButton() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.cancelledByUser
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
        #expect(viewModel.shouldShowLockScreen)
    }

    @Test
    func systemCancelledUnlockWithoutBackgroundingShowsUnlockButton() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.cancelledBySystem
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func lockedStateDoesNotShowUnlockButton() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        #expect(viewModel.state == .locked)
        #expect(!viewModel.isUnlockButtonVisible)
    }

    @Test
    func failedUnlockShowsUnlockButton() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.authenticationFailed
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func biometryUnavailableShowsUnlockButton() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.biometryUnavailable
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func unexpectedErrorShowsUnlockButton() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = NSError(domain: "TestError", code: 999, userInfo: nil)
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.isUnlockButtonVisible)
    }

    @Test
    func startUnlockJoinsAttemptInFlight() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: mockService)

        let first = viewModel.startUnlock()
        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)

        mockService.errorToThrow = BiometryAuthenticationError.cancelledBySystem
        mockService.releaseAuthentication()
        await first?.value
        #expect(viewModel.state == .locked)

        mockService.errorToThrow = nil
        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.isUnlocking)

        await viewModel.startUnlock()?.value
        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 2)
    }

    @Test
    func staleAttemptIsReplacedOnActivationAndLateResultIgnored() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.isUnlocking)

        await viewModel.startUnlock()?.value

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 1)
    }

    @Test
    func activationDoesNotRetryAfterUserCancel() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .lockedCanceled

        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .lockedCanceled)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func activationLocksAndStartsUnlockWhenGracePeriodExpired() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date(timeIntervalSince1970: 0)

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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date().addingTimeInterval(1000)

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func gracePeriodExtendedDuringBackgrounding() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = .now + 0.5 // inside grace period

        viewModel.handleSceneChange(to: .background)

        // lastUnlockTime must now be ≈ now + 60 s
        let delta = Int(viewModel.lastUnlockTime.timeIntervalSinceNow.rounded())
        #expect(delta == LockPeriod.oneMinute.value)
    }

    @Test
    func togglingAuthOffResetsViewModel() async throws {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.handleSceneChange(to: .inactive) // showPlaceholderPreview true

        try await mockService.enableAuthentication(false, reason: "unit")
        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
    }

    @Test
    func changingLockPeriodDoesNotTriggerImmediateLock() throws {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = .distantFuture

        try mockService.update(period: .fiveMinutes)

        #expect(viewModel.lockPeriod == .fiveMinutes)
        #expect(viewModel.lastUnlockTime == .distantFuture)
    }

    @Test
    func isLockedProperty() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.state = .unlocked
        #expect(!viewModel.isLocked)

        viewModel.state = .locked
        #expect(!viewModel.isLocked)
    }

    @Test
    func shouldShowLockScreen() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
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
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        let task = viewModel.startUnlock()

        #expect(task == nil)
        #expect(viewModel.state == .unlocked)
        #expect(mockService.authenticateCallsCount == 0)
    }

    @Test
    func shouldLockWhenAutoLockEnabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.lastUnlockTime = Date().addingTimeInterval(-1000)
        #expect(viewModel.shouldLock)

        viewModel.lastUnlockTime = Date().addingTimeInterval(1000)
        #expect(!viewModel.shouldLock)
    }

    @Test
    func shouldLockWhenAutoLockDisabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.lastUnlockTime = Date().addingTimeInterval(-1000)
        #expect(!viewModel.shouldLock)

        viewModel.lastUnlockTime = Date().addingTimeInterval(1000)
        #expect(!viewModel.shouldLock)
    }

    @Test
    func rapidSceneChanges() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date().addingTimeInterval(1000)

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)
        viewModel.handleSceneChange(to: .background)

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldLock)
    }

    @Test
    func resetLockState() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
        #expect(!viewModel.isLocked)
        #expect(viewModel.lastUnlockTime == .distantFuture)
    }

    @Test
    func waitUntilUnlockedReturnsWhenAlreadyUnlocked() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        await viewModel.waitUntilUnlocked()

        #expect(viewModel.state == .unlocked)
    }

    @Test
    func waitUntilUnlockedResumesAfterUnlock() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        let waiter = Task { await viewModel.waitUntilUnlocked() }

        viewModel.startUnlock()
        await waiter.value

        #expect(viewModel.state == .unlocked)
    }
}

// MARK: - Mock

// TODO: - probably move to Keystore TestKip
class MockBiometryAuthenticationService: BiometryAuthenticatable, @unchecked Sendable {
    var lockPeriod: LockPeriod

    var requiresAuthentication: Bool
    var isPrivacyLockEnabled: Bool
    var availableAuthentication: KeystoreAuthentication

    var shouldAuthenticateSucceed: Bool = true
    var errorToThrow: (any Error)?
    var holdAuthentication: Bool = false
    private(set) var authenticateCallsCount: Int = 0

    private var holdContinuations: [CheckedContinuation<Void, Never>] = []

    init(isAuthEnabled: Bool,
         availableAuth: KeystoreAuthentication,
         lockPeriod: LockPeriod = .default,
         isPrivacyLockEnabled: Bool = false)
    {
        requiresAuthentication = isAuthEnabled
        availableAuthentication = availableAuth
        self.lockPeriod = lockPeriod
        self.isPrivacyLockEnabled = isPrivacyLockEnabled
    }

    @MainActor
    func enableAuthentication(_ enable: Bool, context _: LAContext, reason _: String) async throws {
        requiresAuthentication = enable
        if !enable {
            isPrivacyLockEnabled = false
            lockPeriod = .default
        }
    }

    @MainActor
    func authenticate(context _: LAContext, reason _: String) async throws {
        authenticateCallsCount += 1
        if holdAuthentication {
            await withCheckedContinuation { holdContinuations.append($0) }
        }
        if let error = errorToThrow {
            throw error
        }
        if !shouldAuthenticateSucceed {
            throw BiometryAuthenticationError.cancelledByUser
        }
    }

    func releaseAuthentication() {
        holdAuthentication = false
        holdContinuations.forEach { $0.resume() }
        holdContinuations.removeAll()
    }

    func releaseNextAuthentication() {
        guard holdContinuations.isNotEmpty else { return }
        holdContinuations.removeFirst().resume()
    }

    func update(period: LockPeriod) throws {
        lockPeriod = period
    }

    func togglePrivacyLock(enbaled: Bool) throws {
        isPrivacyLockEnabled = enbaled
    }
}
