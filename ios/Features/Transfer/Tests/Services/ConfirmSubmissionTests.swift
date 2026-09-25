// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAssetConfigService
import enum Gemstone.GemConfirmError
import struct Gemstone.GemSimulationBalanceChange
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
            simulation: .mock(primaryFields: [primary]),
        )))
        await model.load()

        let state = model.state.simulation

        #expect(state.primaryFields.count == 1)
        #expect(state.primaryFields.first?.title == .contract)
        #expect(state.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsBalanceChanges() async {
        let usdt = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let model = ConfirmTransferSceneViewModel.mock(load: .success(.mock(
            simulation: .mock(balanceChanges: [GemSimulationBalanceChange(asset: usdt.toGem(), icon: GemAssetConfigService.shared.assetIcon(assetId: usdt.id.identifier), amount: .mock())]),
        )))
        await model.load()

        #expect(model.state.simulation.balanceChanges == [GemSimulationBalanceChange(asset: usdt.toGem(), icon: GemAssetConfigService.shared.assetIcon(assetId: usdt.id.identifier), amount: .mock())])
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
