// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.SolanaTokenProgramId {
    func map() -> Primitives.SolanaTokenProgramId {
        switch self {
        case .token: .token
        case .token2022: .token2022
        }
    }
}

public extension Primitives.SolanaTokenProgramId {
    func map() -> Gemstone.SolanaTokenProgramId {
        switch self {
        case .token: .token
        case .token2022: .token2022
        }
    }
}
