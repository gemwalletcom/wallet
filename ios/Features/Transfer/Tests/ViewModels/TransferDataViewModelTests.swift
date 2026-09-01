// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import enum Gemstone.GemTransactionInputType
import BigInt
import Localization
@testable import Primitives
import PrimitivesTestKit
import class Gemstone.GemTransferService
import Testing
@testable import Transfer

struct TransferDataViewModelTests {
    @Test
    func depositTitle() {
        #expect(TransferDataViewModel.mock(type: .deposit(.mock())).title == "Deposit")
    }

    @Test
    func genericSendTitle() {
        let type = GemTransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .send))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func genericSignTitle() {
        let type = GemTransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .sign))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func availableValueForUnfreeze() throws {
        let balance = Balance(available: 1000, frozen: 500, locked: 300)

        #expect(try TransferData.mock(type: .stake(.mock(), .unfreeze(.bandwidth))).availableValue(balance: balance, transferService: GemTransferService()) == BigInt(500))
        #expect(try TransferData.mock(type: .stake(.mock(), .unfreeze(.energy))).availableValue(balance: balance, transferService: GemTransferService()) == BigInt(300))
    }
}

private extension TransferDataViewModel {
    static func mock(
        type: GemTransactionInputType = .transfer(.mock()),
    ) -> TransferDataViewModel {
        TransferDataViewModel(
            data: TransferData.mock(type: type),
        )
    }
}
