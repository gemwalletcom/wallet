// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TransferDataOutputAction {
    func map() -> Primitives.TransferDataOutputAction {
        switch self {
        case .sign: .sign
        case .send: .send
        }
    }
}

public extension Primitives.TransferDataOutputAction {
    func map() -> Gemstone.TransferDataOutputAction {
        switch self {
        case .sign: .sign
        case .send: .send
        }
    }
}
