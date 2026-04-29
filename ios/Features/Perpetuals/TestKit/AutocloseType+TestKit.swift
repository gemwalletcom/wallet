// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Perpetuals
import Primitives
import PrimitivesTestKit

extension AutocloseType {
    static func mockModify(
        position: PerpetualPositionData = .mock(),
    ) -> AutocloseType {
        .modify(position, onTransferAction: { _ in })
    }

    static func mockOpen(
        data: AutocloseOpenData = .mock(),
    ) -> AutocloseType {
        .open(data, onComplete: { _, _ in })
    }
}
