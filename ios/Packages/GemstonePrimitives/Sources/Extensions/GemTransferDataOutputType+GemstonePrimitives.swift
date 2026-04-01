// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TransferDataOutputType {
    func map() -> Primitives.TransferDataOutputType {
        switch self {
        case .encodedTransaction: .encodedTransaction
        case .signature: .signature
        }
    }
}

public extension Primitives.TransferDataOutputType {
    func map() -> Gemstone.TransferDataOutputType {
        switch self {
        case .encodedTransaction: .encodedTransaction
        case .signature: .signature
        }
    }
}
