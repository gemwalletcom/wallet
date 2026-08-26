// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import AssetsService
import BalanceService
import ChainService
import ExplorerService
import Foundation
import class Gemstone.GemConfirmService
import Keystore
import PriceService
import Primitives
import PrimitivesComponents
import Signer
import TransactionStateService

public enum ConfirmServiceFactory {
    public static func create(
        keystore: any Keystore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsEnabler: any AssetsEnabler,
        gemConfirmService: GemConfirmService,
        balanceService: BalanceService,
        assetsService: AssetsService,
        priceService: PriceService,
        transactionStateScheduler: TransactionStateScheduler,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        toastPresenter: ToastPresenter,
        chain: Chain,
    ) -> ConfirmService {
        let chainService = chainServiceFactory.service(for: chain)

        return ConfirmService(
            metadataProvider: TransferMetadataProvider(
                balanceService: balanceService,
                priceService: priceService,
            ),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProvider(
                    confirmService: gemConfirmService,
                ),
                feeAssetProvider: FeeAssetProvider(assetStore: assetsService.assetStore),
            ),
            simulationService: ConfirmSimulationService(
                addressNameService: addressNameService,
                assetsService: assetsService,
            ),
            transferExecutor: TransferExecutor(
                signer: TransactionSigner(keystore: keystore),
                confirmService: gemConfirmService,
                assetsEnabler: assetsEnabler,
                transactionStateScheduler: transactionStateScheduler,
            ),
            activityService: activityService,
            toastPresenter: toastPresenter,
            keystore: keystore,
            chainService: chainService,
            explorerService: ExplorerService.standard,
            addressNameService: addressNameService,
        )
    }
}
