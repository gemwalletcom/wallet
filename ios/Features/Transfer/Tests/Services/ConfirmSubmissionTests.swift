// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmFeeSelection
import Foundation
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemConfirmSimulation
import enum Gemstone.GemExecuteResult
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import Transfer

@MainActor
struct ConfirmSubmissionTests {
    @Test
    func confirmReportsEveryHashAndTracksSentTransactions() async throws {
        let tracked = Primitives.Transaction.mock()
        let gemConfirmService = GemConfirmServiceMock(execute: .success(.sent(hashes: ["hash-1", "hash-2"], transactions: [tracked.json()])))
        let reported = ReportedValues()

        let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
        try await ConfirmTransferSceneViewModel.mock(request: request, gemConfirmService: gemConfirmService).submit(
            request: request,
            confirmData: .mock(),
            amount: .mock(),
            simulation: nil,
        )

        #expect(reported.values == ["hash-1", "hash-2"])
        #expect(gemConfirmService.executedInputs.count == 1)
    }

    @Test
    func confirmReportsSignedDataWithoutTracking() async throws {
        let gemConfirmService = GemConfirmServiceMock(execute: .success(.signed(data: ["signed"])))
        let reported = ReportedValues()

        let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
        try await ConfirmTransferSceneViewModel.mock(request: request, gemConfirmService: gemConfirmService).submit(
            request: request,
            confirmData: .mock(),
            amount: .mock(),
            simulation: nil,
        )

        #expect(reported.values == ["signed"])
    }

    @Test
    func partialBroadcastReportsBroadcastHashesAndRethrows() async throws {
        let gemConfirmService = GemConfirmServiceMock(execute: .failure(GemConfirmError.Broadcast(hashes: ["hash-1"], msg: "second leg failed")))
        let reported = ReportedValues()

        await #expect(throws: GemConfirmError.self) {
            let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
            try await ConfirmTransferSceneViewModel.mock(request: request, gemConfirmService: gemConfirmService).submit(
                request: request,
                confirmData: .mock(),
                amount: .mock(),
                simulation: nil,
            )
        }

        #expect(reported.values == ["hash-1"])
    }

    @Test
    func simulationStateMapsTheHeader() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmTransferSceneViewModel.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                primaryFields: [],
                secondaryFields: [],
                header: GemSimulationValue(asset: usdt.map(), value: .exact(value: 1_000_000)),
                balanceChanges: [],
                hasCriticalWarning: false,
            ),
        ))

        let state = service.state.simulation

        #expect(state.headerData == AssetValueHeaderData(asset: usdt, value: .exact(1_000_000)))
        #expect(state.payload.primaryFields.isEmpty)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsAnUnlimitedHeader() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmTransferSceneViewModel.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                primaryFields: [],
                secondaryFields: [],
                header: GemSimulationValue(asset: usdt.map(), value: .unlimited),
                balanceChanges: [],
                hasCriticalWarning: false,
            ),
        ))

        #expect(service.state.simulation.headerData == AssetValueHeaderData(asset: usdt, value: .unlimited))
    }

    @Test
    func simulationStateKeepsPrimaryAndSecondaryFieldsApart() {
        let primary = SimulationPayloadField.standard(kind: .contract, value: "0x1", fieldType: .text, display: .primary)
        let service = ConfirmTransferSceneViewModel.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(primaryFields: [primary.map()], secondaryFields: [], header: nil, balanceChanges: [], hasCriticalWarning: false),
        ))

        let state = service.state.simulation

        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.primaryFields.first?.kind == .contract)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsBalanceChanges() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmTransferSceneViewModel.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                primaryFields: [],
                secondaryFields: [],
                header: nil,
                balanceChanges: [GemSimulationBalanceChange(asset: usdt.map(), value: "-25")],
                hasCriticalWarning: false,
            ),
        ))

        #expect(service.state.simulation.balanceChanges == [SimulationAssetChange(asset: usdt, value: -25)])
    }

}

private final class ReportedValues: @unchecked Sendable {
    private let lock = NSLock()
    private var storage: [String] = []

    var values: [String] { lock.withLock { storage } }

    func append(_ value: String?) {
        guard let value else { return }
        lock.withLock { storage.append(value) }
    }
}
