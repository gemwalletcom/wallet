// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemConfirmError
import struct Gemstone.GemSimulationBalanceChange
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.GemSimulationValue
import enum Gemstone.GemSubmitResult
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
import TransferTestKit

@MainActor
struct ConfirmSubmissionTests {
    @Test
    func confirmReportsEveryHashAndTracksSentTransactions() async throws {
        let reported = ReportedValues()

        let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
        try await ConfirmTransferSceneViewModel.mock(
            request: request,
            execute: .success(.sent(hashes: ["hash-1", "hash-2"], warning: nil)),
        ).submit(request: request)

        #expect(reported.values == ["hash-1", "hash-2"])
    }

    @Test
    func confirmReportsSignedDataWithoutTracking() async throws {
        let reported = ReportedValues()

        let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
        try await ConfirmTransferSceneViewModel.mock(request: request, execute: .success(.signed(data: ["signed"], warning: nil))).submit(request: request)

        #expect(reported.values == ["signed"])
    }

    @Test
    func partialBroadcastReportsBroadcastHashesAndRethrows() async throws {
        let reported = ReportedValues()

        await #expect(throws: GemConfirmError.self) {
            let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
            try await ConfirmTransferSceneViewModel.mock(
                request: request,
                execute: .failure(GemConfirmError.Broadcast(hashes: ["hash-1"], msg: "second leg failed")),
            ).submit(request: request)
        }

        #expect(reported.values == ["hash-1"])
    }

    @Test
    func simulationStateMapsTheHeader() async {
        let usdt = Asset.mockEthereumUSDT()
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(header: GemSimulationValue(asset: usdt.toGem(), value: .exact(value: 1_000_000))),
        )))
        await model.load()

        let state = model.state.simulation

        #expect(state.headerData == GemSimulationValue(asset: usdt.toGem(), value: .exact(value: BigUInt(1_000_000))))
        #expect(state.payload.primaryFields.isEmpty)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsAnUnlimitedHeader() async {
        let usdt = Asset.mockEthereumUSDT()
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(header: GemSimulationValue(asset: usdt.toGem(), value: .unlimited)),
        )))
        await model.load()

        #expect(model.state.simulation.headerData == GemSimulationValue(asset: usdt.toGem(), value: .unlimited))
    }

    @Test
    func simulationStateKeepsPrimaryAndSecondaryFieldsApart() async {
        let primary = GemSimulationPayloadRow(title: .contract, value: .text(text: "0x1"))
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(primaryFields: [primary]),
        )))
        await model.load()

        let state = model.state.simulation

        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.primaryFields.first?.title == .contract)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsBalanceChanges() async {
        let usdt = Asset.mockEthereumUSDT()
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(balanceChanges: [GemSimulationBalanceChange(asset: usdt.toGem(), value: "-25", sign: .outgoing, tone: .negative)]),
        )))
        await model.load()

        #expect(model.state.simulation.balanceChanges == [GemSimulationBalanceChange(asset: usdt.toGem(), value: "-25", sign: .outgoing, tone: .negative)])
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
