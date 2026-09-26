// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemConfirmError
import enum Gemstone.GemListRow
import struct Gemstone.GemSimulationPayloadRow
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
            execute: .success(.sent(hashes: ["hash-1", "hash-2"], message: nil)),
        ).submit(request: request)

        #expect(reported.values == ["hash-1", "hash-2"])
    }

    @Test
    func confirmReportsSignedDataWithoutTracking() async throws {
        let reported = ReportedValues()

        let request = ConfirmTransferRequest.mock(delegate: { reported.append(try? $0.get()) })
        try await ConfirmTransferSceneViewModel.mock(request: request, execute: .success(.signed(data: ["signed"], message: nil))).submit(request: request)

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
    func simulationStateKeepsPrimaryAndSecondaryFieldsApart() async {
        let primary = GemSimulationPayloadRow(title: .contract, value: .text(text: "0x1"))
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(simulation: .mock(primaryFields: [primary])),
        )))
        await model.load()

        #expect(model.primaryPayloadFields.count == 1)
        #expect(model.primaryPayloadFields.first?.title == .contract)
        #expect(model.secondaryPayloadFields.isEmpty)
    }

    @Test
    func simulationStateMapsBalanceChanges() async {
        let change = GemListRow.assetChange(
            name: "Tether",
            icon: .mock(),
            amount: .mock(value: 1, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .signed, tone: .plain, rounding: .toNearest),
        )
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(simulation: .mock(balanceChanges: [change])),
        )))
        await model.load()

        #expect(model.viewState.sections.contains(.balanceChanges(rows: [change])))
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
