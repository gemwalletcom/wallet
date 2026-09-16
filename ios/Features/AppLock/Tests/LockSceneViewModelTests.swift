// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppLock
import Foundation
import enum Gemstone.GemAppLockScreen
import GemstoneServices
import LocalAuthentication
import Primitives
import Testing

@MainActor
struct LockSceneViewModelTests {
    @Test
    func startsLockedOnlyWhenAuthenticationIsRequired() {
        let locked = LockSceneViewModel(service: MockBiometryAuthenticationService(isAuthEnabled: true))
        #expect(locked.viewState.screen == .lock(unlockButton: false))
        #expect(!locked.viewState.isUnlocked)

        let open = LockSceneViewModel(service: MockBiometryAuthenticationService(isAuthEnabled: false))
        #expect(open.viewState.screen == .hidden)
        #expect(open.viewState.isUnlocked)
    }

    @Test
    func activationPromptsOnceAndSuccessUnlocks() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true)
        service.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: service)

        viewModel.handleSceneChange(to: .active)
        viewModel.handleSceneChange(to: .active)
        await Task.yield()
        #expect(service.authenticateCallsCount == 1)
        #expect(viewModel.viewState.screen == .lock(unlockButton: false))

        service.releaseAuthentication()
        await viewModel.attempt?.task.value

        #expect(viewModel.viewState.screen == .hidden)
        #expect(viewModel.viewState.isUnlocked)
    }

    @Test
    func cancelledPromptWaitsForTheUnlockButton() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true)
        service.errorToThrow = BiometryAuthenticationError.cancelledByUser
        let viewModel = LockSceneViewModel(service: service)

        viewModel.handleSceneChange(to: .active)
        await viewModel.attempt?.task.value
        #expect(viewModel.viewState.screen == .lock(unlockButton: true))

        viewModel.handleSceneChange(to: .active)
        await viewModel.attempt?.task.value
        #expect(service.authenticateCallsCount == 1)

        service.errorToThrow = nil
        viewModel.requestUnlock()
        await viewModel.attempt?.task.value
        #expect(service.authenticateCallsCount == 2)
        #expect(viewModel.viewState.isUnlocked)
    }

    @Test
    func backgroundingRepromptsOnActivationAndIgnoresTheStaleResult() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true)
        service.holdAuthentication = true
        let viewModel = LockSceneViewModel(service: service)

        viewModel.handleSceneChange(to: .active)
        await Task.yield()
        let first = viewModel.attempt?.task
        viewModel.handleSceneChange(to: .inactive)
        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)
        await Task.yield()
        #expect(service.authenticateCallsCount == 2)

        service.errorToThrow = BiometryAuthenticationError.cancelledBySystem
        service.releaseNextAuthentication()
        await first?.value
        #expect(viewModel.viewState.screen == .lock(unlockButton: false))

        service.errorToThrow = nil
        service.releaseAuthentication()
        await viewModel.attempt?.task.value
        #expect(viewModel.viewState.isUnlocked)
    }

    @Test
    func ownPromptKeepsContentVisibleButLeavingCoversIt() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true, isPrivacyLockEnabled: true)
        let viewModel = await unlocked(service)

        service.isAuthenticating = true
        viewModel.handleSceneChange(to: .inactive)
        #expect(viewModel.viewState.screen == .hidden)

        service.isAuthenticating = false
        viewModel.handleSceneChange(to: .inactive)
        #expect(viewModel.viewState.screen == .cover)

        viewModel.handleSceneChange(to: .background)
        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.viewState.screen == .hidden)
        #expect(service.authenticateCallsCount == 1)
    }

    @Test
    func immediateLockPeriodRelocksOnReturn() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true, lockPeriod: .immediate)
        let viewModel = await unlocked(service)

        viewModel.handleSceneChange(to: .background)
        try? await Task.sleep(for: .milliseconds(2))
        viewModel.handleSceneChange(to: .active)
        #expect(viewModel.viewState.screen == .lock(unlockButton: false))

        await viewModel.attempt?.task.value
        #expect(service.authenticateCallsCount == 2)
        #expect(viewModel.viewState.isUnlocked)
    }

    @Test
    func turningAuthenticationOffUnlocksWithoutAPrompt() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true)
        service.errorToThrow = BiometryAuthenticationError.cancelledByUser
        let viewModel = LockSceneViewModel(service: service)
        viewModel.handleSceneChange(to: .active)
        await viewModel.attempt?.task.value

        service.requiresAuthentication = false
        viewModel.handleSceneChange(to: .active)

        #expect(viewModel.viewState == .init(screen: .hidden, isUnlocked: true))
        #expect(service.authenticateCallsCount == 1)
    }

    @Test
    func waitUntilUnlockedResumesAfterUnlock() async {
        let service = MockBiometryAuthenticationService(isAuthEnabled: true)
        let viewModel = LockSceneViewModel(service: service)
        let waiter = Task { await viewModel.waitUntilUnlocked() }

        viewModel.handleSceneChange(to: .active)
        await waiter.value

        #expect(viewModel.viewState.isUnlocked)
    }

    private func unlocked(_ service: MockBiometryAuthenticationService) async -> LockSceneViewModel {
        let viewModel = LockSceneViewModel(service: service)
        viewModel.handleSceneChange(to: .active)
        await viewModel.attempt?.task.value
        return viewModel
    }
}

// MARK: - Mock

@MainActor
final class MockBiometryAuthenticationService: BiometryAuthenticatable {
    var requiresAuthentication: Bool
    var isPrivacyLockEnabled: Bool
    var isAuthenticating = false
    var lockPeriod: LockPeriod
    let availableAuthentication: KeystoreAuthentication = .biometrics

    var errorToThrow: (any Error)?
    var holdAuthentication = false
    private(set) var authenticateCallsCount = 0
    private var holdContinuations: [CheckedContinuation<Void, Never>] = []

    init(isAuthEnabled: Bool, lockPeriod: LockPeriod = .oneMinute, isPrivacyLockEnabled: Bool = false) {
        requiresAuthentication = isAuthEnabled
        self.lockPeriod = lockPeriod
        self.isPrivacyLockEnabled = isPrivacyLockEnabled
    }

    func authenticate(context _: LAContext) async throws {
        authenticateCallsCount += 1
        if holdAuthentication {
            await withCheckedContinuation { holdContinuations.append($0) }
        }
        if let error = errorToThrow {
            throw error
        }
    }

    func enableAuthentication(_ enable: Bool, context _: LAContext) async throws {
        requiresAuthentication = enable
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

    func togglePrivacyLock(enabled: Bool) throws {
        isPrivacyLockEnabled = enabled
    }
}
