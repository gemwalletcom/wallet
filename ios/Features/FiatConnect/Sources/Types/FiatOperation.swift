// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import GemAPI
import Primitives
import Validators

protocol FiatOperation: Sendable {
    var defaultAmount: Int { get }
    var emptyAmountTitle: String { get }

    func fetch(amount: Double) async throws -> [FiatQuote]

    func validators(
        availableBalance: BigInt,
        selectedQuote: FiatQuote?,
    ) -> [any TextValidator]
}
