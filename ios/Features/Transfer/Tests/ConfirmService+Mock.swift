// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
import Store
import StoreTestKit
@testable import Transfer
import TransferTestKit

extension ConfirmService {
    static func mock(
        transaction: Result<TransferTransactionData, Error> = .success(TransferTransactionData(allRates: [], transactionData: .mock())),
        gemConfirmService: GemConfirmServiceMock = GemConfirmServiceMock(),
        transactionStateTracker: TransactionStateTracker = .mock(),
    ) -> ConfirmService {
        ConfirmService(
            metadataProvider: TransferMetadataProviderMock(metadataResult: .success(.mock())),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProviderMock(result: transaction),
                feeAssetProvider: FeeAssetProviderMock(),
            ),
            simulationService: ConfirmSimulationService(nameService: GemNameServiceMock(), assetsService: GemAssetsServiceMock(), assetStore: .mock()),
            gemConfirmService: gemConfirmService,
            signer: GemTransactionSignerMock(),
            preferencesService: GemPreferencesServiceMock(),
            transactionStateTracker: transactionStateTracker,
            recentActivityStore: .mock(),
            toastPresenter: ToastPresenter(),
            keystore: KeystoreMock(),
            explorerService: GemExplorerServiceMock(),
            addressStore: .mock(),
        )
    }
}
