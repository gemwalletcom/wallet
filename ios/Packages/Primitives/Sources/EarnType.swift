// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum EarnType: Codable, Equatable, Hashable, Sendable {
    case deposit(DelegationValidator)
    case withdraw(Delegation)
}
