// Copyright (c). Gem Wallet. All rights reserved.

import Store
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import GemstoneServices
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import class Gemstone.GemConfirmService
import Preferences
import Primitives
import PrimitivesComponents

public enum ConfirmServiceFactory {
    public static func create(
        explorerService: any GemExplorerServiceProtocol,
        keystore: any Keystore,
        chainServiceFactory: any ChainServiceFactorable,
        gemConfirmService: GemConfirmService,
        balanceStore: BalanceStore,
        assetStore: AssetStore,
        assetsService: any GemAssetsServiceProtocol,
        priceStore: PriceStore,
        transactionStateScheduler: TransactionStateScheduler,
        nameService: any GemNameServiceProtocol,
        addressStore: AddressStore,
        recentActivityStore: RecentActivityStore,
        toastPresenter: ToastPresenter,
        chain: Chain,
    ) -> ConfirmService {
        let chainService = chainServiceFactory.service(for: chain)

        return ConfirmService(
            metadataProvider: TransferMetadataProvider(
                balanceStore: balanceStore,
                priceStore: priceStore,
            ),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProvider(
                    confirmService: gemConfirmService,
                ),
                feeAssetProvider: FeeAssetProvider(assetStore: assetStore),
            ),
            simulationService: ConfirmSimulationService(
                nameService: nameService,
                assetsService: assetsService,
                assetStore: assetStore,
            ),
            transferExecutor: TransferExecutor(
                signer: TransactionSigner(keystore: keystore),
                confirmService: gemConfirmService,
                preferences: .standard,
                transactionStateScheduler: transactionStateScheduler,
            ),
            recentActivityStore: recentActivityStore,
            toastPresenter: toastPresenter,
            keystore: keystore,
            chainService: chainService,
            explorerService: explorerService,
            addressStore: addressStore,
        )
    }
}
