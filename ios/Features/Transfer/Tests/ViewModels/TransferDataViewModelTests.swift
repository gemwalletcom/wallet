// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import enum Gemstone.TransactionInputType
import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
import struct Gemstone.GemTransferData
@testable import Transfer

struct TransferDataViewModelTests {
    @Test
    func depositTitle() {
        #expect(TransferDataViewModel.mock(type: .deposit(.mock())).title == "Deposit")
    }

    @Test
    func genericSendTitle() {
        let type = TransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .send))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func genericSignTitle() {
        let type = TransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .sign))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }
}

private extension TransferDataViewModel {
    static func mock(
        type: TransactionInputType = .transfer(.mock()),
    ) -> TransferDataViewModel {
        TransferDataViewModel(
            data: GemTransferData.mock(type: type),
        )
    }
}
