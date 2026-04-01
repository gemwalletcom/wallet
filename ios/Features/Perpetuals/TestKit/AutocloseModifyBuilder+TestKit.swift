// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Perpetuals
import Primitives

extension AutocloseModifyBuilder {
    static func mock(
        direction: PerpetualDirection = .long,
    ) -> AutocloseModifyBuilder {
        AutocloseModifyBuilder(direction: direction)
    }
}
