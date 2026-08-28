// Copyright (c). Gem Wallet. All rights reserved.

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

extension ConfirmService {
    static func mock(
        transaction: Result<GemConfirmData, Error> = .success(.mock()),
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
