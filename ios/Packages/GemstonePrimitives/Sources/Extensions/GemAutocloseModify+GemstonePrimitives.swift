// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAutocloseModify
import Primitives

public extension GemAutocloseModify {
    func modifyTypes() -> [PerpetualModifyPositionType] {
        do {
            return try build().map { try PerpetualModifyPositionType($0) }
        } catch {
            preconditionFailure("Undecodable modify position type: \(error)")
        }
    }
}
