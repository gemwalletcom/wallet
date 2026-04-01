// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import GemAPI
import Localization
import Primitives
import Validators

struct BuyOperation: FiatOperation {
    private let service: any GemAPIFiatService
    private let asset: Asset
    private let currencyFormatter: CurrencyFormatter
    private let walletId: WalletId

    private let config = FiatOperationConfig(
        defaultAmount: 50,
        minimumAmount: 5,
        maximumAmount: 10000,
    )

    var defaultAmount: Int { config.defaultAmount }

    var emptyAmountTitle: String {
        Localized.Input.enterAmountTo(Localized.Wallet.buy)
    }

    init(
        service: any GemAPIFiatService,
        asset: Asset,
        currencyFormatter: CurrencyFormatter,
        walletId: WalletId,
    ) {
        self.service = service
        self.asset = asset
        self.currencyFormatter = currencyFormatter
        self.walletId = walletId
    }

    func fetch(amount: Double) async throws -> [FiatQuote] {
        let request = FiatQuoteRequest(amount: amount, currency: currencyFormatter.currencyCode)
        return try await service.getQuotes(walletId: walletId.id, type: .buy, assetId: asset.id, request: request)
    }

    func validators(
        availableBalance _: BigInt,
        selectedQuote _: FiatQuote?,
    ) -> [any TextValidator] {
        let rangeValidator = FiatRangeValidator(
            range: BigInt(config.minimumAmount) ... BigInt(config.maximumAmount),
            minimumValueText: currencyFormatter.string(Double(config.minimumAmount)),
            maximumValueText: currencyFormatter.string(Double(config.maximumAmount)),
        )
        return [.assetAmount(decimals: 0, validators: [rangeValidator])]
    }
}
