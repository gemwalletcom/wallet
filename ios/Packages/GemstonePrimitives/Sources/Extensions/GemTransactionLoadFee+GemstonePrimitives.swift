// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemTransactionLoadFee {
    func map() throws -> Fee {
        try Fee(
            fee: BigInt.from(string: fee),
            gasPriceType: gasPriceType.map(),
            gasLimit: BigInt.from(string: gasLimit),
            options: options.map(),
        )
    }
}

public extension Fee {
    func map() -> Gemstone.GemTransactionLoadFee {
        Gemstone.GemTransactionLoadFee(
            fee: fee.description,
            gasPriceType: gasPriceType.map(),
            gasLimit: gasLimit.description,
            options: options.map(),
        )
    }
}
