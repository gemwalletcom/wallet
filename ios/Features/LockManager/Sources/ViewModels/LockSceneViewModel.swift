// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
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

    var lastUnlockTime: Date = .init(timeIntervalSince1970: 0)
    var state: LockSceneState

    private var showPlaceholderPreview: Bool = false

    public init(
        service: any BiometryAuthenticatable = BiometryAuthenticationService(),
    ) {
        self.service = service
        state = service.isAuthenticationEnabled ? .locked : .unlocked
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
        service.isAuthenticationEnabled
    }

    var isLocked: Bool {
        state != .unlocked && isAutoLockEnabled
    }

    var isUnlocking: Bool {
        if case .unlocking = state { true } else { false }
    }

    var isUnlockButtonVisible: Bool {
        state == .locked || state == .lockedCanceled
    }

    var shouldLock: Bool {
        Date() > lastUnlockTime && isAutoLockEnabled
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
            return state == .lockedCanceled || shouldLock
        }
    }
}

// MARK: - Business Logic

extension LockSceneViewModel {
    func handleSceneChange(to phase: ScenePhase) {
        switch phase {
        case .background:
            if case let .unlocking(attempt) = state {
                attempt.context.invalidate()
            }
            if state == .unlocked, !shouldLock {
                lastUnlockTime = Date().addingTimeInterval(TimeInterval(lockPeriod.value))
            }
        case .active:
            showPlaceholderPreview = false
            if state == .unlocked, shouldLock {
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
        lastUnlockTime = .distantFuture
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
        do {
            try await service.authenticate(context: context, reason: Self.reason)
            resetLockState()
        } catch let error as BiometryAuthenticationError {
            state = error == .cancelledBySystem ? .locked : .lockedCanceled
        } catch {
            state = .lockedCanceled
        }
    }
}
