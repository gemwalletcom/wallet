// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemValueHeader
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct ConfirmHeaderItemTests {
    @Test
    func theHeaderItemCarriesCoresHeaderAndWhetherItOnlyReservesItsPlace() {
        let confirmation = GemConfirmationMock()
        confirmation.headerValue = .mock(header: .value(header: .mock()), isReserved: true)
        let model = ConfirmTransferSceneViewModel.mock(confirmation: confirmation)

        guard case let .header(header) = model.itemModel(for: .header) else {
            Issue.record("Expected the header item")
            return
        }
        #expect(header == confirmation.headerValue)
    }
}
