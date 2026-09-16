// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAppLockSession
import struct Gemstone.GemAppLockViewState
import func Gemstone.newAppLockSession
import GemstoneServices
import LocalAuthentication
import Localization
import Observation
import Style
import SwiftUI

@MainActor
@Observable
public final class LockSceneViewModel {
    private let service: any BiometryAuthenticatable
    private let launchedAt = ContinuousClock.now

    private(set) var session: GemAppLockSession
    private(set) var attempt: UnlockAttempt?

    public init(service: any BiometryAuthenticatable) {
        self.service = service
        session = newAppLockSession(settings: service.lockSettings)
    }

    var viewState: GemAppLockViewState {
        session.viewState(nowMilliseconds: nowMilliseconds)
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
}

// MARK: - Business Logic

extension LockSceneViewModel {
    func handleSceneChange(to phase: ScenePhase) {
        session = session.onSettingsChanged(settings: service.lockSettings)
        switch phase {
        case .active:
            session = session.onActive(nowMilliseconds: nowMilliseconds, hasPendingRequest: false)
        case .inactive:
            session = session.onInactive(ownPromptVisible: service.isAuthenticating)
        case .background:
            attempt?.context.invalidate()
            session = session.onBackground(nowMilliseconds: nowMilliseconds)
        @unknown default:
            break
        }
        startUnlockAttempt()
    }

    func requestUnlock() {
        session = session.onUnlockRequested()
        startUnlockAttempt()
    }

    public func waitUntilUnlocked() async {
        while !viewState.isUnlocked {
            await withCheckedContinuation { continuation in
                withObservationTracking {
                    _ = session
                } onChange: {
                    continuation.resume()
                }
            }
        }
    }
}

// MARK: - Private

extension LockSceneViewModel {
    private var nowMilliseconds: Int64 {
        (ContinuousClock.now - launchedAt).milliseconds
    }

    private func startUnlockAttempt() {
        guard case let .unlocking(number, _) = session.phase, attempt?.number != number else { return }
        let context = LAContext()
        attempt = UnlockAttempt(
            number: number,
            context: context,
            task: Task { await authenticate(attempt: number, context: context) },
        )
    }

    private func authenticate(attempt: UInt32, context: LAContext) async {
        do {
            try await service.authenticate(context: context)
            session = session.onUnlocked(attempt: attempt)
        } catch let error as BiometryAuthenticationError {
            session = session.onUnlockFailed(attempt: attempt, outcome: error.promptOutcome)
        } catch {
            session = session.onUnlockFailed(attempt: attempt, outcome: .failed)
        }
    }
}

// MARK: - Previews

extension LockSceneViewModel {
    static var preview: LockSceneViewModel {
        LockSceneViewModel(service: BiometryAuthenticationService(keystorePassword: LocalKeystorePassword(), reason: Localized.Settings.Security.authentication))
    }
}
