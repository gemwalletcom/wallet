// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
@testable import Primitives
import Testing

final class FeeTests {
    @Test
    func testTotalFee() {
        let fee = Fee(fee: BigInt(1), gasPriceType: .regular(gasPrice: BigInt(1)), gasLimit: .zero)
        #expect(fee.totalFee == BigInt(1))
    }

    @Test
    func totalFeeWithTokenCreation() {
        let fee = Fee(
            fee: BigInt(1),
            gasPriceType: .regular(gasPrice: BigInt(1)),
            gasLimit: .zero,
            options: [.tokenAccountCreation: BigInt(10)],
        )
        #expect(fee.totalFee == BigInt(11))
    }
}
