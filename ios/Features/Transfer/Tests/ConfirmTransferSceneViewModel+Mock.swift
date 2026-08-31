// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemAmountService
import class Gemstone.GemTransferService
import protocol Gemstone.GemTransactionStateServiceProtocol
import struct Gemstone.GemConfirmData
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import Store
import StoreTestKit
@testable import Transfer
import TransferTestKit
import class Gemstone.GemFeeService
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemAssetConfigService
import class Gemstone.GemSwapQuoteService
import class Gemstone.GemApplicationMetadataService

@MainActor
extension ConfirmTransferSceneViewModel {
    static func mock(
        request: ConfirmTransferRequest? = nil,
        wallet: Wallet? = nil,
        data: TransferData = .mock(),
        simulation: SimulationResult? = nil,
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        nameService: any GemNameServiceProtocol = GemNameServiceMock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
        onComplete: VoidAction = nil,
    ) -> ConfirmTransferSceneViewModel {
        ConfirmTransferSceneViewModel(
            request: request ?? ConfirmTransferRequest(
                wallet: wallet ?? .mock(accounts: [.mock(chain: data.chain)]),
                data: data,
                simulation: simulation,
            ),
            gemConfirmService: gemConfirmService,
            signer: GemTransactionSignerMock(),
            keystore: KeystoreMock(),
            explorerService: GemExplorerServiceMock(),
            nameService: nameService,
            assetsService: GemAssetsServiceMock(),
            transactionStateService: transactionStateService,
            recentAssetsService: RecentAssetsService(store: .mock()),
            toastPresenter: ToastPresenter(),
            perpetualService: GemPerpetualServiceMock(),
            preferencesService: GemPreferencesServiceMock(),
            transferService: GemTransferService(),
            onComplete: onComplete,
            assetConfig: GemAssetConfigService(),
            feeService: GemFeeService(),
            swapQuoteService: GemSwapQuoteService(),
            applicationMetadataService: GemApplicationMetadataService(),
        )
    }
}
