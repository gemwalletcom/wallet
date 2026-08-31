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
import class Gemstone.GemTransferService
import protocol Gemstone.GemConfirmServiceProtocol
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemAmountService
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
        recentAssetsService: any RecentAssetsServiceable,
        toastPresenter: ToastPresenter,
        feeService: GemFeeService,
        transferService: GemTransferService,
        amountService: GemAmountService,
        simulationFormatter: GemSimulationFormatter,
        perpetualService: any GemPerpetualServiceProtocol,
    ) -> ConfirmService {
        return ConfirmService(
            gemConfirmService: gemConfirmService,
            signer: KeystoreTransactionSigner(keystore: keystore),
            preferencesService: preferencesService,
            transactionStateService: transactionStateService,
            recentAssetsService: recentAssetsService,
            toastPresenter: toastPresenter,
            keystore: keystore,
            explorerService: explorerService,
            nameService: nameService,
            assetsService: assetsService,
            feeService: feeService,
            transferService: transferService,
            perpetualService: perpetualService,
        )
    }
}
