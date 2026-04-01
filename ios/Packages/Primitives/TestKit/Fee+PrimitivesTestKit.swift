// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public extension Fee {
    static func mock(
        fee: BigInt = BigInt(21000),
        gasPriceType: GasPriceType = .regular(gasPrice: BigInt(1_000_000_000)),
        gasLimit: BigInt = BigInt(21000),
        options: FeeOptionMap = [:],
    ) -> Fee {
        Fee(
            fee: fee,
            gasPriceType: gasPriceType,
            gasLimit: gasLimit,
            options: options,
        )
    }
}
