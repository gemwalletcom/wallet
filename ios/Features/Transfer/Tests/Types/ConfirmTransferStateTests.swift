// Copyright (c). Gem Wallet. All rights reserved.

@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmTransferStateTests {
    private let warning = SimulationWarning(severity: .warning, warning: .externallyOwnedSpender, message: nil)

    @Test
    func loadingKeepsSimulationAndMetadata() {
        var state = ConfirmTransferState.mock(
            transaction: .data(.mock()),
            metadata: .mock(),
            simulation: .mock(warnings: [warning]),
        )

        state.transaction = .loading

        #expect(state.transaction.isLoading)
        #expect(state.metadata != nil)
        #expect(state.simulation.warnings.count == 1)
    }

    @Test
    func loadedCarriesBundle() {
        let data = ConfirmTransferData(metadata: .mock(), input: .mock(), simulation: .mock(warnings: [warning]))

        let loaded = ConfirmTransferState.loaded(data)

        #expect(loaded.transaction.value != nil)
        #expect(loaded.metadata != nil)
        #expect(loaded.simulation.warnings.count == 1)
    }

    @Test
    func failedKeepsSimulationAndMetadata() {
        var state = ConfirmTransferState.mock(
            transaction: .loading,
            metadata: .mock(),
            simulation: .mock(warnings: [warning]),
        )

        state.transaction = .error(AnyError("boom"))

        #expect(state.transaction.isError)
        #expect(state.metadata != nil)
        #expect(state.simulation.warnings.count == 1)
    }
}
