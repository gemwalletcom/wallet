// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
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
        gemConfirmService: GemConfirmService,
        preferencesService: any GemPreferencesServiceProtocol,
        balanceStore: BalanceStore,
        assetStore: AssetStore,
        assetsService: any GemAssetsServiceProtocol,
        priceStore: PriceStore,
        transactionStateTracker: TransactionStateTracker,
        nameService: any GemNameServiceProtocol,
        addressStore: AddressStore,
        recentActivityStore: RecentActivityStore,
        toastPresenter: ToastPresenter,
    ) -> ConfirmService {
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
            gemConfirmService: gemConfirmService,
            signer: KeystoreTransactionSigner(keystore: keystore),
            preferencesService: preferencesService,
            transactionStateTracker: transactionStateTracker,
            recentActivityStore: recentActivityStore,
            toastPresenter: toastPresenter,
            keystore: keystore,
            explorerService: explorerService,
            addressStore: addressStore,
        )
    }
}
