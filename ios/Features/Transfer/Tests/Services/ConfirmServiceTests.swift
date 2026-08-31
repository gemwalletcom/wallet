// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemSimulationBalanceChange
import enum Gemstone.GemConfirmError
import enum Gemstone.GemExecuteResult
import GemstoneServicesTestKit
import GemstoneServices
import BigInt
import Foundation
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import class Gemstone.GemSimulationFormatter
@testable import Transfer

struct ConfirmServiceTests {
    @Test
    func confirmReportsEveryHashAndTracksSentTransactions() async throws {
        let tracked = Primitives.Transaction.mock()
        let gemConfirmService = GemConfirmServiceMock(execute: .success(.sent(hashes: ["hash-1", "hash-2"], transactions: [tracked.json()])))
        let reported = ReportedValues()

        try await ConfirmService.mock(gemConfirmService: gemConfirmService).confirm(
            request: .mock(wallet: .mock(accounts: [Account.mock(chain: .ethereum)]), delegate: { reported.append(try? $0.get()) }),
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

        try await ConfirmService.mock(gemConfirmService: gemConfirmService).confirm(
            request: .mock(wallet: .mock(accounts: [Account.mock(chain: .ethereum)]), delegate: { reported.append(try? $0.get()) }),
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
            try await ConfirmService.mock(gemConfirmService: gemConfirmService).confirm(
                request: .mock(wallet: .mock(accounts: [Account.mock(chain: .ethereum)]), delegate: { reported.append(try? $0.get()) }),
                confirmData: .mock(),
                amount: .mock(),
                simulation: nil,
            )
        }

        #expect(reported.values == ["hash-1"])
    }

    @Test
    func simulationStateMapsTheResolvedHeader() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmService.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                payloadFields: [],
                header: GemSimulationValue(asset: usdt.json(), value: .exact(value: "1000000")),
                balanceChanges: [],
            ),
        ))

        let state = service.simulationState(request: .mock())

        #expect(state.headerData == AssetValueHeaderData(asset: usdt, value: .exact(1_000_000)))
        #expect(state.payload.primaryFields.isEmpty)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsAnUnlimitedHeader() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmService.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                payloadFields: [],
                header: GemSimulationValue(asset: usdt.json(), value: .unlimited),
                balanceChanges: [],
            ),
        ))

        #expect(service.simulationState(request: .mock()).headerData == AssetValueHeaderData(asset: usdt, value: .unlimited))
    }

    @Test
    func simulationStateSplitsPayloadFieldsByDisplay() {
        let primary = SimulationPayloadField.standard(kind: .contract, value: "0x1", fieldType: .text, display: .primary)
        let service = ConfirmService.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(payloadFields: [primary.json()], header: nil, balanceChanges: []),
        ))

        let state = service.simulationState(request: .mock())

        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.primaryFields.first?.kind == .contract)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsResolvedBalanceChanges() {
        let usdt = Asset.mockEthereumUSDT()
        let service = ConfirmService.mock(gemConfirmService: GemConfirmServiceMock(
            simulation: GemConfirmSimulation(
                payloadFields: [],
                header: nil,
                balanceChanges: [GemSimulationBalanceChange(asset: usdt.json(), value: "-25")],
            ),
        ))

        #expect(service.simulationState(request: .mock()).balanceChanges == [SimulationAssetChange(asset: usdt, value: -25)])
    }

    @Test
    func simulationStateIgnoresAddressNameLookupFailure() async throws {
        let field = SimulationPayloadField.standard(kind: .contract, value: "0x123", fieldType: .address, display: .primary)
        let service = ConfirmService.mock(
            gemConfirmService: GemConfirmServiceMock(
                metadata: .success(GemConfirmMetadata(
                    assetBalance: .mock(assetId: Asset.mock().id.identifier),
                    feeAssetBalance: .mock(assetId: Asset.mock().id.identifier),
                    prices: [],
                )),
                preload: .success(.mock()),
                simulation: GemConfirmSimulation(payloadFields: [field.json()], header: nil, balanceChanges: []),
            ),
            nameService: GemNameServiceMock(error: NSError(domain: "test", code: 404)),
        )

        let request = ConfirmTransferRequest.mock(wallet: .mock(accounts: [.mock(chain: TransferData.mock().chain)]))
        let state = try await service.load(request: request, selection: FeeSelection.preset(.normal), feeAssetSelection: FeeAssetSelection.automatic).simulation

        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.secondaryFields.isEmpty)
        #expect(state.payload.addressNames.isEmpty)
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
