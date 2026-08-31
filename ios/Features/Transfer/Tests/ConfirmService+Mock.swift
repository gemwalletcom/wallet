// Copyright (c). Gem Wallet. All rights reserved.

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

extension ConfirmService {
    static func mock(
        transaction: Result<GemConfirmData, Error> = .success(.mock(input: TransferData.mock().confirmInput(from: .mock()))),
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
    ) -> ConfirmService {
        ConfirmService(
            metadataProvider: TransferMetadataProviderMock(metadataResult: .success(.mock())),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProviderMock(result: transaction),
                feeAssetProvider: FeeAssetProviderMock(),
                feeService: GemFeeService(),
            transferService: GemTransferService(),
            ),
            simulationService: ConfirmSimulationService(
                nameService: GemNameServiceMock(),
                assetsService: GemAssetsServiceMock(),
                simulationFormatter: GemSimulationFormatter(),
            ),
            gemConfirmService: gemConfirmService,
            signer: GemTransactionSignerMock(),
            preferencesService: GemPreferencesServiceMock(),
            transactionStateService: transactionStateService,
            recentAssetsService: RecentAssetsService(store: .mock()),
            toastPresenter: ToastPresenter(),
            keystore: KeystoreMock(),
            explorerService: GemExplorerServiceMock(),
            nameService: GemNameServiceMock(),
            feeService: GemFeeService(),
            perpetualService: GemPerpetualServiceMock(),
        )
    }
}
