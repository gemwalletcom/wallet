// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import GemstoneServices
import BigInt
import Foundation
import GemstonePrimitivesTestKit
import GemAPITestKit
import KeystoreTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import Transfer

struct ConfirmServiceTests {
    @Test
    func simulationStateUsesTransferApprovalValue() {
        let service = ConfirmSimulationService(
            nameService: GemNameServiceMock(),
            assetsService: GemAssetsServiceMock(),
            assetStore: .mock(),
        )

        let state = service.makeState(
            data: TransferData.mock(type: .tokenApprove(.mockEthereumUSDT(), ApprovalData(token: "", spender: "", value: "1000000", isUnlimited: false))),
            simulation: SimulationResult.mock(payload: [
                SimulationPayloadField.standard(kind: .value, value: "1000000", fieldType: .text, display: .primary),
            ]),
        )

        #expect(state.headerData == AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(1_000_000)))
        #expect(state.payload.primaryFields.isEmpty)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func genericApprovalHeaderUsesCachedAsset() async throws {
        let assetStore = AssetStore.mock()
        try assetStore.add(assets: [.mock(asset: .mockEthereumUSDT())])

        let service = ConfirmSimulationService(
            nameService: GemNameServiceMock(),
            assetsService: GemAssetsServiceMock(),
            assetStore: assetStore,
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

        let service = ConfirmSimulationService(
            nameService: GemNameServiceMock(),
            assetsService: GemAssetsServiceMock(),
            assetStore: assetStore,
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
        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.primaryFields.first?.kind == .contract)
        #expect(state.payload.secondaryFields.isEmpty)
    }

    @Test
    func simulationStateMapsResolvedBalanceChanges() throws {
        let solana = Asset.mock(id: .mockSolana(), name: "Solana", symbol: "SOL", decimals: 9, type: .native)
        let usdc = Asset.mock(id: .mockSolanaUSDC(), name: "USD Coin", symbol: "USDC", decimals: 6, type: .spl)
        let unknownAssetId = AssetId(chain: .solana, tokenId: "MissingMint111111111111111111111111111111111")
        let assetStore = AssetStore.mock()
        try assetStore.add(assets: [.mock(asset: solana), .mock(asset: usdc)])

        let service = ConfirmSimulationService(
            nameService: GemNameServiceMock(),
            assetsService: GemAssetsServiceMock(),
            assetStore: assetStore,
        )

        let state = service.makeState(
            data: TransferData.mock(type: .transfer(solana)),
            simulation: SimulationResult.mock(balanceChanges: [
                SimulationBalanceChange(assetId: solana.id, value: "-100005000", decimals: 9, name: "Solana", symbol: "SOL"),
                SimulationBalanceChange(assetId: usdc.id, value: "750000", decimals: 6, name: "USD Coin", symbol: "USDC"),
                SimulationBalanceChange(assetId: unknownAssetId, value: "-42", decimals: 2, name: nil, symbol: nil),
            ]),
        )

        #expect(state.balanceChanges == [
            SimulationAssetChange(asset: solana, value: -100_005_000),
            SimulationAssetChange(asset: usdc, value: 750_000),
        ])
    }

    @Test
    func simulationStatePrefetchesBalanceChangeAsset() async {
        let dust = Asset.mock(
            id: AssetId(chain: .ton, tokenId: "EQBlqsm144Dq6SjbPI4jjZvA1hqTIP3CvHovbIfW_t-SCALE"),
            name: "DeDust",
            symbol: "DUST",
            decimals: 9,
            type: .jetton,
        )
        let simulation = SimulationResult.mock(balanceChanges: [
            SimulationBalanceChange(assetId: dust.id, value: "2244508455", decimals: 0, name: nil, symbol: nil),
        ])
        let expected = [
            SimulationAssetChange(asset: dust, value: 2_244_508_455),
        ]

        let assetStore = AssetStore.mock()
        let fetchedState = await ConfirmSimulationService(
            nameService: GemNameServiceMock(),
            assetsService: GemAssetsServiceMock(
                assetsResult: [.mock(asset: dust)],
                store: GemstoneAssetStore(assetStore: assetStore, balanceStore: .mock()),
            ),
            assetStore: assetStore,
        ).updateState(
            data: TransferData.mock(type: .transfer(.mock())),
            simulation: simulation,
        )

        #expect(fetchedState.balanceChanges == expected)
    }

    @Test
    func simulationStateIgnoresAddressNameLookupFailure() async {
        let service = ConfirmSimulationService(
            nameService: GemNameServiceMock(error: NSError(domain: "test", code: 404)),
            assetsService: GemAssetsServiceMock(),
            assetStore: .mock(),
        )

        let state = await service.updateState(
            data: TransferData.mock(type: .generic(asset: .mockBNB(), metadata: .mock(), extra: .mock())),
            simulation: SimulationResult.mock(payload: [
                SimulationPayloadField.standard(kind: .contract, value: "0x123", fieldType: .address, display: .primary),
            ]),
        )

        #expect(state.payload.primaryFields.count == 1)
        #expect(state.payload.secondaryFields.isEmpty)
        #expect(state.payload.addressNames.isEmpty)
    }
}
