// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import protocol Gemstone.GemFiatQuoteServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import Validators

struct FiatAmountValidator: ValueValidator {
    typealias Formatted = BigInt

    private let service: any GemFiatQuoteServiceProtocol
    private let type: FiatQuoteType
    private let asset: Asset
    private let quote: FiatQuote?
    private let availableBalance: BigInt
    private let currencyFormatter: CurrencyFormatter

    init(
        service: any GemFiatQuoteServiceProtocol,
        type: FiatQuoteType,
        asset: Asset,
        quote: FiatQuote?,
        availableBalance: BigInt,
        currencyFormatter: CurrencyFormatter,
    ) {
        self.service = service
        self.type = type
        self.asset = asset
        self.quote = quote
        self.availableBalance = availableBalance
        self.currencyFormatter = currencyFormatter
    }

    func validate(_ value: BigInt) throws {
        switch service.amountCheck(type: type, amount: Double(value), quote: quote, available: availableBalance) {
        case let .belowMinimum(minimum):
            throw AnyError(Localized.Transfer.minimumAmount(currencyFormatter.string(Double(minimum))))
        case let .aboveMaximum(maximum):
            throw AnyError(Localized.Transfer.maximumAmount(currencyFormatter.string(Double(maximum))))
        case let .insufficientBalance(required, available):
            throw TransferAmountCalculatorError.insufficientBalance(
                asset,
                requirement: BalanceRequirement(required: try BigInt.from(string: required), available: try BigInt.from(string: available)),
            )
        case .valid:
            break
        }
    }

    var id: String {
        "FiatAmountValidator"
    }
}
