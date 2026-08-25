// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import AssetsService
import BalanceService
import ChainService
import EventPresenterService
import ExplorerService
import Foundation
import protocol Gemstone.TransactionSimulationServiceProtocol
import Keystore
import PriceService
import Primitives
import ScanService
import Signer
import TransactionStateService

public enum ConfirmServiceFactory {
    public static func create(
        keystore: any Keystore,
        chainServiceFactory: any ChainServiceFactorable,
        assetsEnabler: any AssetsEnabler,
        scanService: ScanService,
        balanceService: BalanceService,
        assetsService: AssetsService,
        priceService: PriceService,
        transactionStateScheduler: TransactionStateScheduler,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService,
        transactionSimulationService: any TransactionSimulationServiceProtocol,
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
                    chainService: chainService,
                    scanService: scanService,
                ),
                feeAssetProvider: FeeAssetProvider(assetStore: assetsService.assetStore),
            ),
            simulationService: ConfirmSimulationService(
                addressNameService: addressNameService,
                assetsService: assetsService,
            ),
            transactionSimulationService: transactionSimulationService,
            transferExecutor: TransferExecutor(
                signer: TransactionSigner(keystore: keystore),
                chainService: chainService,
                assetsEnabler: assetsEnabler,
                transactionStateScheduler: transactionStateScheduler,
            ),
            activityService: activityService,
            eventPresenterService: eventPresenterService,
            keystore: keystore,
            chainService: chainService,
            explorerService: ExplorerService.standard,
            addressNameService: addressNameService,
        )
    }
}
