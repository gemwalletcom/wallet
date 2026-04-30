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

    func testUnlockClearsStateAndSetsTimerToFuture() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()
        #expect(viewModel.state == .unlocked)
        #expect(viewModel.shouldShowLockScreen == false)
        #expect(viewModel.lastUnlockTime == .distantFuture)
        #expect(mockService.didCallAuthenticate)
    }

    @Test
    func cancelledUnlockStaysOnPlaceholder() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.cancelled
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .lockedCanceled)
        #expect(viewModel.shouldShowLockScreen)
    }

    @Test
    func unlockSuccess() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.shouldAuthenticateSucceed = true
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .unlocked)
        #expect(viewModel.lastUnlockTime == Date.distantFuture)
    }

    @Test
    func unlockFailure() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.authenticationFailed
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .locked)
        #expect(viewModel.lastUnlockTime != Date.distantFuture)
    }

    @Test
    func inactiveThenActiveNoGracePeriodLocksImmediately() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date(timeIntervalSince1970: 0) // long ago → shouldLock = true

        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .locked)
        #expect(viewModel.shouldShowLockScreen)
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
        viewModel.state = .locked
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

    func testIsLockedPropertyWhenAuthEnabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.state = .unlocked
        #expect(!viewModel.isLocked)

        viewModel.state = .locked
        #expect(viewModel.isLocked)

        viewModel.state = .unlocking
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

        viewModel.state = .unlocking
        #expect(!viewModel.isLocked)
    }

    @Test
    func testShouldShowLockScreen() {
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
    func shouldLockWhenAutoLockDisabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.lastUnlockTime = Date(timeIntervalSince1970: 0)

        #expect(!viewModel.shouldLock)
    }

    @Test
    func unlockWhenAutoLockDisabled() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .unlocked)
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
        viewModel.lastUnlockTime = Date().addingTimeInterval(1000) // Future date

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)
        viewModel.handleSceneChange(to: .background)

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldLock)
    }

    @Test
    func unlockWithUnexpectedError() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let unexpectedError = NSError(domain: "TestError", code: 999, userInfo: nil)
        mockService.errorToThrow = unexpectedError
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .locked)
    }

    @Test
    func stateTransitionsWithAutoLockDisabled() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()
        #expect(viewModel.state == .unlocked)

        viewModel.handleSceneChange(to: .background)
        #expect(viewModel.state == .unlocked)
    }

    @Test
    func isAutoLockEnabledProperty() {
        let mockServiceEnabled = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModelEnabled = LockSceneViewModel(service: mockServiceEnabled)
        #expect(viewModelEnabled.isAutoLockEnabled)

        let mockServiceDisabled = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModelDisabled = LockSceneViewModel(service: mockServiceDisabled)
        #expect(!viewModelDisabled.isAutoLockEnabled)
    }

    @Test
    func shouldLockPropertyWhenAutoLockEnabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.lastUnlockTime = Date().addingTimeInterval(-1000) // Past date
        #expect(viewModel.shouldLock)

        viewModel.lastUnlockTime = Date().addingTimeInterval(1000) // Future date
        #expect(!viewModel.shouldLock)
    }

    @Test
    func shouldLockPropertyWhenAutoLockDisabled() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: false,
            availableAuth: .none,
        )
        let viewModel = LockSceneViewModel(service: mockService)

        viewModel.lastUnlockTime = Date().addingTimeInterval(-1000) // Past date
        #expect(!viewModel.shouldLock)

        viewModel.lastUnlockTime = Date().addingTimeInterval(1000) // Future date
        #expect(!viewModel.shouldLock)
    }

    @Test
    func handleSceneChangeToActiveWhenUnlockedAndShouldLock() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date(timeIntervalSince1970: 0) // Past date

        // Simulate going to background
        viewModel.handleSceneChange(to: .background)
        // Now simulate becoming active
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .locked)
        #expect(viewModel.shouldShowLockScreen)
    }

    @Test
    func handleSceneChangeWhenStateIsNotUnlocked() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        let initialStates: [LockSceneState] = [.locked, .unlocking, .lockedCanceled]

        for state in initialStates {
            viewModel.state = state
            viewModel.handleSceneChange(to: .background)
            #expect(viewModel.state == state)

            viewModel.handleSceneChange(to: .active)
            #expect(viewModel.state == state)
        }
    }

    @Test
    func unlockWithBiometryUnavailableError() async {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        mockService.errorToThrow = BiometryAuthenticationError.biometryUnavailable
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        await viewModel.unlock()

        #expect(viewModel.state == .locked)
    }

    @Test
    func testResetLockState() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .locked

        // Perform reset
        viewModel.resetLockState()

        #expect(viewModel.state == .unlocked)
        #expect(!viewModel.shouldShowLockScreen)
        #expect(!viewModel.isLocked)
    }

    @Test
    func inactiveToActiveTransitionWithoutBackgroundLocksWhenExpired() {
        let mockService = MockBiometryAuthenticationService(
            isAuthEnabled: true,
            availableAuth: .biometrics,
            lockPeriod: .oneMinute,
        )
        let viewModel = LockSceneViewModel(service: mockService)
        viewModel.state = .unlocked
        viewModel.lastUnlockTime = Date(timeIntervalSince1970: 0)

        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.state == .locked)
        #expect(viewModel.shouldShowLockScreen)
    }
}

// MARK: - Mock

// TODO: - probably move to Keystore TestKip
class MockBiometryAuthenticationService: BiometryAuthenticatable, @unchecked Sendable {
    var lockPeriod: LockPeriod

    var isAuthenticationEnabled: Bool
    var isPrivacyLockEnabled: Bool
    var availableAuthentication: KeystoreAuthentication

    var shouldAuthenticateSucceed: Bool = true
    var errorToThrow: (any Error)?
    var didCallAuthenticate: Bool = false

    init(isAuthEnabled: Bool,
         availableAuth: KeystoreAuthentication,
         lockPeriod: LockPeriod = .default,
         isPrivacyLockEnabled: Bool = false)
    {
        isAuthenticationEnabled = isAuthEnabled
        availableAuthentication = availableAuth
        self.lockPeriod = lockPeriod
        self.isPrivacyLockEnabled = isPrivacyLockEnabled
        self.lockPeriod = lockPeriod
    }

    func enableAuthentication(_ enable: Bool, context _: LAContext, reason _: String) async throws {
        isAuthenticationEnabled = enable
        if !enable {
            isPrivacyLockEnabled = false
            lockPeriod = .default
        }
    }

    func authenticate(context _: LAContext, reason _: String) async throws {
        didCallAuthenticate = true
        if let error = errorToThrow {
            throw error
        }
        if !shouldAuthenticateSucceed {
            throw BiometryAuthenticationError.cancelled
        }
    }

    func update(period: LockPeriod) throws {
        lockPeriod = period
    }

    func togglePrivacyLock(enbaled: Bool) throws {
        isPrivacyLockEnabled = enbaled
    }
}
