// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.PerpetualConfirmData
import struct Gemstone.PerpetualModifyConfirmData
import struct Gemstone.PerpetualReduceData
import enum Gemstone.PerpetualType

public extension PerpetualType {
    static func mockOpen(
        data: PerpetualConfirmData = .mock(),
    ) -> PerpetualType {
        .open(data: data)
    }

    static func mockClose(
        data: PerpetualConfirmData = .mock(),
    ) -> PerpetualType {
        .close(data: data)
    }

    static func mockIncrease(
        data: PerpetualConfirmData = .mock(),
    ) -> PerpetualType {
        .increase(data: data)
    }

    static func mockReduce(
        data: PerpetualReduceData = .mock(),
    ) -> PerpetualType {
        .reduce(data: data)
    }

    static func mockModify(
        data: PerpetualModifyConfirmData = .mock(),
    ) -> PerpetualType {
        .modify(data: data)
    }
}
