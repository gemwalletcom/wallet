// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualMetadata {
    func map() -> PerpetualMetadata {
        PerpetualMetadata(isPinned: isPinned)
    }
}
