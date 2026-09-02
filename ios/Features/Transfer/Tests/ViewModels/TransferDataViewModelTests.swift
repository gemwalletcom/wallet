// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import enum Gemstone.GemTransactionInputType
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
        let type = GemTransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .send))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func genericSignTitle() {
        let type = GemTransactionInputType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .sign))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }
}

private extension TransferDataViewModel {
    static func mock(
        type: GemTransactionInputType = .transfer(.mock()),
    ) -> TransferDataViewModel {
        TransferDataViewModel(
            data: GemTransferData.mock(type: type),
        )
    }
}
