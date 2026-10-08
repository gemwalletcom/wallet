// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSecurityService
import GemstoneServices
import LocalAuthentication
import Localization
import Observation
import Primitives
import PrimitivesComponents
import SwiftUI

@MainActor
@Observable
public class LockSceneViewModel {
    private static let reason: String = Localized.Settings.Security.authentication
    private static let obscureDelay: Duration = .milliseconds(150)

    private let service: any BiometryAuthenticatable

    var leftAt: ContinuousClock.Instant?
    var state: LockSceneState
    private var isObscured = false
    private(set) var obscureTask: Task<Void, Never>?

    public init(
        service: any BiometryAuthenticatable,
    ) {
        self.service = service
        state = service.requiresAuthentication ? .locked : .unlocked
    }

    var unlockTitle: String {
        Localized.Lock.unlock
    }

    var unlockImage: String? {
        service.availableAuthentication.systemImage
    }

    var isAutoLockEnabled: Bool {
        service.requiresAuthentication
    }

    var isLocked: Bool {
        state != .unlocked && isAutoLockEnabled
    }

    var isCovered: Bool {
        isLocked || isObscured
    }

    var isUnlockButtonVisible: Bool {
        state == .lockedCanceled
    }

    var isPasscodeOff: Bool {
        state == .passcodeOff
    }

    var passcodeOffTitle: String {
        Localized.Lock.passcodeOffTitle
    }

    var passcodeOffDescription: String {
        Localized.Lock.passcodeOffDescription
    }

    var shouldLock: Bool {
        guard let leftAt else { return false }
        return service.shouldRelock(elapsedMilliseconds: (ContinuousClock.now - leftAt).milliseconds)
    }
}

// MARK: - Business Logic

extension LockSceneViewModel {
    func onScenePhase(_ phase: ScenePhase) {
        guard isAutoLockEnabled else {
            resetLockState()
            return
        }
        switch phase {
        case .background:
            setObscured(true)
            if case let .unlocking(attempt) = state, !attempt.isInvalidated {
                state = .unlocking(attempt.invalidated())
            }
            startLockPeriod(at: ContinuousClock.now)
        case .active:
            setObscured(false)
            lockIfExpired()
            leftAt = nil
            if case let .unlocking(attempt) = state, attempt.isInvalidated {
                state = .locked
            }
            if state == .locked || state == .passcodeOff {
                startUnlock()
            }
        case .inactive:
            lockIfExpired()
            obscureAfterDelay()
        @unknown default:
            break
        }
    }

    @discardableResult
    func startUnlock() -> Task<Void, Never>? {
        switch state {
        case let .unlocking(attempt):
            return attempt.task
        case .unlocked:
            return nil
        case .locked, .lockedCanceled, .passcodeOff:
            guard isAutoLockEnabled else {
                resetLockState()
                return nil
            }
            guard service.isPasscodeSet else {
                state = .passcodeOff
                return nil
            }
            let context = LAContext()
            let task = Task {
                await authenticate(context: context)
            }
            state = .unlocking(UnlockAttempt(context: context, task: task))
            return task
        }
    }

    func resetLockState() {
        setObscured(false)
        leftAt = nil
        state = .unlocked
    }

    public func waitUntilUnlocked() async {
        while state != .unlocked {
            await withCheckedContinuation { continuation in
                withObservationTracking {
                    _ = state
                } onChange: {
                    continuation.resume()
                }
            }
        }
    }
}

// MARK: - Private

extension LockSceneViewModel {
    private func lockIfExpired() {
        if state == .unlocked, shouldLock {
            state = .locked
        }
    }

    private func setObscured(_ obscured: Bool) {
        obscureTask?.cancel()
        obscureTask = nil
        isObscured = obscured
    }

    private func obscureAfterDelay() {
        obscureTask?.cancel()
        let inactiveAt = ContinuousClock.now
        obscureTask = Task {
            try? await Task.sleep(for: Self.obscureDelay)
            guard !Task.isCancelled, isAutoLockEnabled, !service.hasPresentedSystemPrompt(since: inactiveAt) else { return }
            isObscured = true
            startLockPeriod(at: inactiveAt)
        }
    }

    private func startLockPeriod(at instant: ContinuousClock.Instant) {
        if state == .unlocked, leftAt == nil {
            leftAt = instant
        }
    }

    private func authenticate(context: LAContext) async {
        let newState = await getAuthenticationState(context: context)
        guard case let .unlocking(attempt) = state, attempt.context === context else { return }

        switch newState {
        case .unlocked:
            resetLockState()
        case .locked where !attempt.isInvalidated:
            state = .lockedCanceled
        default:
            state = newState
        }
    }

    private func getAuthenticationState(context: LAContext) async -> LockSceneState {
        do {
            try await service.authenticate(context: context, reason: Self.reason)
            return .unlocked
        } catch let error as BiometryAuthenticationError {
            return error == .cancelledBySystem ? .locked : .lockedCanceled
        } catch {
            return .lockedCanceled
        }
    }
}
