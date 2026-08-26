// Copyright (c). Gem Wallet. All rights reserved.

import Store
import protocol Gemstone.GemNameServiceProtocol
import ActivityService
import GemstoneServices
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import class Gemstone.GemConfirmService
import Keystore
import Primitives
import PrimitivesComponents

public enum ConfirmServiceFactory {
    public static func create(
        explorerService: any GemExplorerServiceProtocol,
        keystore: any Keystore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsEnabler: any AssetsEnabler,
        gemConfirmService: GemConfirmService,
        balanceStore: BalanceStore,
        assetsService: AssetsService,
        priceService: PriceService,
        transactionStateScheduler: TransactionStateScheduler,
        nameService: any GemNameServiceProtocol,
        addressStore: AddressStore,
        activityService: ActivityService,
        toastPresenter: ToastPresenter,
        chain: Chain,
    ) -> ConfirmService {
        let chainService = chainServiceFactory.service(for: chain)

        return ConfirmService(
            metadataProvider: TransferMetadataProvider(
                balanceStore: balanceStore,
                priceService: priceService,
            ),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProvider(
                    confirmService: gemConfirmService,
                ),
                feeAssetProvider: FeeAssetProvider(assetStore: assetsService.assetStore),
            ),
            simulationService: ConfirmSimulationService(
                nameService: nameService,
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
            explorerService: explorerService,
            addressStore: addressStore,
        )
    }
}
