// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import Store
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstoneServices
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemPerpetualServiceProtocol
import Foundation
import protocol Gemstone.GemConfirmServiceProtocol
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemFeeService
import Preferences
import Primitives
import PrimitivesComponents

public enum ConfirmServiceFactory {
    public static func create(
        explorerService: any GemExplorerServiceProtocol,
        keystore: any Keystore,
        gemConfirmService: any GemConfirmServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        assetStore: AssetStore,
        assetsService: any GemAssetsServiceProtocol,
        transactionStateService: any GemTransactionStateServiceProtocol,
        nameService: any GemNameServiceProtocol,
        recentActivityStore: RecentActivityStore,
        toastPresenter: ToastPresenter,
        feeService: GemFeeService,
        simulationFormatter: GemSimulationFormatter,
        perpetualService: any GemPerpetualServiceProtocol,
    ) -> ConfirmService {
        return ConfirmService(
            metadataProvider: TransferMetadataProvider(confirmService: gemConfirmService),
            inputProvider: ConfirmTransferInputProvider(
                transferTransactionProvider: TransferTransactionProvider(
                    confirmService: gemConfirmService,
                ),
                feeAssetProvider: FeeAssetProvider(assetStore: assetStore),
                feeService: feeService,
            ),
            simulationService: ConfirmSimulationService(
                nameService: nameService,
                assetsService: assetsService,
                simulationFormatter: simulationFormatter,
            ),
            gemConfirmService: gemConfirmService,
            signer: KeystoreTransactionSigner(keystore: keystore),
            preferencesService: preferencesService,
            transactionStateService: transactionStateService,
            recentActivityStore: recentActivityStore,
            toastPresenter: toastPresenter,
            keystore: keystore,
            explorerService: explorerService,
            nameService: nameService,
            feeService: feeService,
            perpetualService: perpetualService,
        )
    }
}
