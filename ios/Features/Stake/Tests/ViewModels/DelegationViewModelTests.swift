// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.delegationListRows
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
@testable import Stake
import Testing

struct DelegationViewModelTests {
    @Test
    func itemsFormatEachDelegationWithTheAssetPrice() {
        let delegations: [Delegation] = [
            .mock(base: .mock(state: .active, assetId: .mock(chain: .tron), balance: 1_500_000_000, delegationId: "1")),
            .mock(base: .mock(state: .active, assetId: .mock(chain: .tron), balance: 500_000_000, delegationId: "2")),
        ]

        let models = delegationListRows(delegations: delegations.map { $0.toGem() }, asset: Chain.tron.asset.toGem(), price: 2.0, currency: Currency.usd.toGem()).map(DelegationViewModel.init(row:))

        #expect(models.map(\.balanceText) == ["1,500 TRX", "500 TRX"])
        #expect(models.map(\.fiatValueText) == ["$3,000.00", "$1,000.00"])
    }
}
