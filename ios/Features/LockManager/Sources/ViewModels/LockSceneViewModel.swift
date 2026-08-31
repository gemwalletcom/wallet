// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import GemstoneServices
import LocalAuthentication
import Localization
import Observation
import Style
import SwiftUI

@MainActor
@Observable
public class LockSceneViewModel {
    private static let reason: String = Localized.Settings.Security.authentication

    private let service: any BiometryAuthenticatable

    var backgroundedAt: ContinuousClock.Instant?
    var state: LockSceneState

    private var showPlaceholderPreview: Bool = false

    public init(
        service: any BiometryAuthenticatable = BiometryAuthenticationService(),
    ) {
        self.service = service
        state = service.requiresAuthentication ? .locked : .unlocked
    }

    var unlockTitle: String {
        Localized.Lock.unlock
    }

    var unlockImage: String? {
        switch service.availableAuthentication {
        case .biometrics: SystemImage.faceid
        case .passcode: SystemImage.lock
        case .none: .none
        }
    }

    var isAutoLockEnabled: Bool {
        service.requiresAuthentication
    }

    var isLocked: Bool {
        state != .unlocked && isAutoLockEnabled
    }

    var isUnlocking: Bool {
        if case .unlocking = state { true } else { false }
    }

    var isUnlockButtonVisible: Bool {
        state == .lockedCanceled
    }

    var shouldLock: Bool {
        guard let backgroundedAt else { return false }
        return service.shouldRelock(elapsedMilliseconds: (ContinuousClock.now - backgroundedAt).milliseconds)
    }

    var shouldShowLockScreen: Bool {
        isLocked || showPlaceholderPreview
    }

    var lockPeriod: LockPeriod {
        service.lockPeriod
    }

    var isPrivacyLockEnabled: Bool {
        service.isPrivacyLockEnabled
    }

    var privacyLockAlpha: CGFloat {
        isPrivacyLockVisible ? 1 : 0
    }

    var isPrivacyLockVisible: Bool {
        guard isAutoLockEnabled else { return false }

        if isPrivacyLockEnabled {
            return state != .unlocked || showPlaceholderPreview
        } else {
            return state == .locked || state == .lockedCanceled || shouldLock
        }
    }
}

// MARK: - Business Logic

extension LockSceneViewModel {
    func handleSceneChange(to phase: ScenePhase) {
        switch phase {
        case .background:
            if case let .unlocking(attempt) = state, !attempt.isInvalidated {
                state = .unlocking(attempt.invalidated())
            }
            if state == .unlocked, !shouldLock {
                backgroundedAt = ContinuousClock.now
            }
        case .active:
            showPlaceholderPreview = false
            if state == .unlocked, shouldLock {
                state = .locked
            }
            if case let .unlocking(attempt) = state, attempt.isInvalidated {
                state = .locked
            }
            if state == .locked {
                startUnlock()
            }
        case .inactive:
            showPlaceholderPreview = true
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
        case .locked, .lockedCanceled:
            guard isAutoLockEnabled else {
                resetLockState()
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
        showPlaceholderPreview = false
        backgroundedAt = nil
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
