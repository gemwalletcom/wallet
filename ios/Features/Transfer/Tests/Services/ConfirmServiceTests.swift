// Copyright (c). Gem Wallet. All rights reserved.

import ActivityServiceTestKit
import AddressNameServiceTestKit
import AssetsServiceTestKit
import BalanceServiceTestKit
import ChainServiceTestKit
import EventPresenterServiceTestKit
import Foundation
import GemAPITestKit
import KeystoreTestKit
import PriceServiceTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import ScanServiceTestKit
import Store
import StoreTestKit
import Testing
import TransactionStateServiceTestKit
@testable import Transfer

struct ConfirmServiceTests {
    @Test
    func simulationStateUsesTransferApprovalValue() {
        let service = ConfirmSimulationServiceFactory.create(
            addressNameService: .mock(addressStore: .mock()),
            assetsService: .mock(),
        )

        let state = service.makeState(
            data: TransferData.mock(type: .tokenApprove(.mockEthereumUSDT(), ApprovalData(token: "", spender: "", value: "1000000", isUnlimited: false))),
            simulation: SimulationResult.mock(payload: [
                SimulationPayloadField.standard(kind: .value, value: "1000000", fieldType: .text, display: .primary),
            ]),
        )

        #expect(state.headerData == AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(1_000_000)))
        #expect(state.primaryFields.isEmpty)
        #expect(state.secondaryFields.isEmpty)
    }

    @Test
    func genericApprovalHeaderUsesCachedAsset() async throws {
        let assetStore = AssetStore.mock()
        try assetStore.add(assets: [.mock(asset: .mockEthereumUSDT())])

        let service = ConfirmSimulationServiceFactory.create(
            addressNameService: .mock(addressStore: .mock()),
            assetsService: .mock(assetStore: assetStore),
        )

        let state = await service.updateState(
            data: TransferData.mock(type: .generic(asset: .mockBNB(), metadata: .mock(), extra: .mock())),
            simulation: SimulationResult.mock(header: SimulationHeader(assetId: Asset.mockEthereumUSDT().id, value: "0", isUnlimited: true)),
        )

        #expect(state.headerData == AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .unlimited))
    }

    @Test
    func simulationStateUsesGenericCachedHeaderAndHidesValueField() throws {
        let assetStore = AssetStore.mock()
        try assetStore.add(assets: [.mock(asset: .mockEthereumUSDT())])

        let service = ConfirmSimulationServiceFactory.create(
            addressNameService: .mock(addressStore: .mock()),
            assetsService: .mock(assetStore: assetStore),
        )

        let state = service.makeState(
            data: TransferData.mock(type: .generic(asset: .mockBNB(), metadata: .mock(), extra: .mock())),
            simulation: SimulationResult.mock(
                payload: [
                    SimulationPayloadField.standard(kind: .contract, value: "0x123", fieldType: .address, display: .primary),
                    SimulationPayloadField.standard(kind: .value, value: "1", fieldType: .text, display: .primary),
                ],
                header: SimulationHeader(assetId: Asset.mockEthereumUSDT().id, value: "0", isUnlimited: true),
            ),
        )

        #expect(state.headerData == AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .unlimited))
        #expect(state.primaryFields.count == 1)
        #expect(state.primaryFields.first?.kind == .contract)
        #expect(state.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateIgnoresAddressNameLookupFailure() async throws {
        let service = ConfirmSimulationServiceFactory.create(
            addressNameService: .mock(
                addressStore: .mock(),
                apiService: GemAPIAddressNamesServiceMock(error: NSError(domain: "test", code: 404)),
            ),
            assetsService: .mock(),
        )

        let state = await service.updateState(
            data: TransferData.mock(type: .generic(asset: .mockBNB(), metadata: .mock(), extra: .mock())),
            simulation: SimulationResult.mock(payload: [
                SimulationPayloadField.standard(kind: .contract, value: "0x123", fieldType: .address, display: .primary),
            ]),
        )

        #expect(state.primaryFields.count == 1)
        #expect(state.secondaryFields.isEmpty)
        #expect(state.payloadAddressNames.isEmpty)
    }
}
