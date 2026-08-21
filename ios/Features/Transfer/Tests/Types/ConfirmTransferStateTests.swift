// Copyright (c). Gem Wallet. All rights reserved.

@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {
    private let warning = SimulationWarning(severity: .warning, warning: .externallyOwnedSpender, message: nil)

    @Test
    func loadedCarriesBundle() {
        let data = ConfirmTransferData(
            preload: ConfirmTransferPreload(
                metadata: .mock(),
                input: .mock(),
                feeRates: [FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 1))],
            ),
            simulation: .mock(warnings: [warning]),
            feeAssets: [.mock(asset: .mockTempoUSDC())],
        )

        let loaded = ConfirmTransferState.loaded(data)

        #expect(loaded.transaction.value != nil)
        #expect(loaded.metadata != nil)
        #expect(loaded.feeRates.count == 1)
        #expect(loaded.feeAssets.count == 1)
        #expect(loaded.simulation.warnings.count == 1)
    }
}
