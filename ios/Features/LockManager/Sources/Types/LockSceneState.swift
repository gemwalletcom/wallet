// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum LockSceneState {
    case unlocking(UnlockAttempt)
    case unlocked
    case locked
    case lockedCanceled
}

extension LockSceneState: Equatable {
    static func == (lhs: Self, rhs: Self) -> Bool {
        switch (lhs, rhs) {
        case (.unlocking, .unlocking),
             (.unlocked, .unlocked),
             (.locked, .locked),
             (.lockedCanceled, .lockedCanceled):
            true
        default:
            false
        }
    }
}
