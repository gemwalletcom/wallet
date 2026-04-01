// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Primitives
import Validators

struct FiatSellValidator: ValueValidator {
    typealias Formatted = BigInt

    private let quote: FiatQuote?
    private let availableBalance: BigInt
    private let asset: Asset
    private let formatter = BigNumberFormatter.standard

    init(
        quote: FiatQuote?,
        availableBalance: BigInt,
        asset: Asset,
    ) {
        self.quote = quote
        self.availableBalance = availableBalance
        self.asset = asset
    }

    func validate(_: BigInt) throws {
        guard let quote else { return }

        let amount = try formatter.number(from: String(quote.cryptoAmount), decimals: asset.decimals.asInt)

        guard amount <= availableBalance else {
            throw TransferAmountCalculatorError.insufficientBalance(asset)
        }
    }

    var id: String { "FiatSellValidator" }
}
