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
        let data = ConfirmTransferData(metadata: .mock(), input: .mock(), simulation: .mock(warnings: [warning]))

        let loaded = ConfirmTransferState.loaded(data)

        #expect(loaded.transaction.value != nil)
        #expect(loaded.metadata != nil)
        #expect(loaded.simulation.warnings.count == 1)
    }
}
