// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives

public extension GasPriceType {
    static func mockEip1559() -> GasPriceType {
        .eip1559(gasPrice: BigInt(5000), priorityFee: BigInt(10000))
    }

    static func mockSolana() -> GasPriceType {
        .solana(gasPrice: BigInt(5000), priorityFee: BigInt(10000), unitPrice: BigInt(200))
    }
}
