// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import AssetsService
import BalanceService
import ChainService
import EventPresenterService
import ExplorerService
import Foundation
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
        assetsService _: AssetsService,
        priceService: PriceService,
        transactionStateService: TransactionStateService,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService,
        chain: Chain,
    ) -> ConfirmService {
        let chainService = chainServiceFactory.service(for: chain)

        return ConfirmService(
            explorerService: ExplorerService.standard,
            metadataProvider: TransferMetadataProvider(
                balanceService: balanceService,
                priceService: priceService,
            ),
            transferTransactionProvider: TransferTransactionProvider(
                chainService: chainService,
                scanService: scanService,
            ),
            transferExecutor: TransferExecutor(
                signer: TransactionSigner(keystore: keystore),
                chainService: chainService,
                assetsEnabler: assetsEnabler,
                balanceService: balanceService,
                transactionStateService: transactionStateService,
            ),
            keystore: keystore,
            chainService: chainService,
            addressNameService: addressNameService,
            activityService: activityService,
            eventPresenterService: eventPresenterService,
        )
    }
}

public enum ConfirmSimulationServiceFactory {
    public static func create(
        addressNameService: AddressNameService,
        assetsService: AssetsService,
    ) -> ConfirmSimulationService {
        ConfirmSimulationService(
            addressNameService: addressNameService,
            assetsService: assetsService,
        )
    }
}
