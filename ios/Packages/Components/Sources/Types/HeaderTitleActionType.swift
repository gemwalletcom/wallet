// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum HeaderTitleActionType: Sendable {
    case privacyToggle
    case action(@MainActor @Sendable () -> Void)
    case none
}
