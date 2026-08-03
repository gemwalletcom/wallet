// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import Assets
import AssetsService
import BalanceService
import ChainService
import EarnService
import EventPresenterService
import FiatConnect
import FiatService
import Foundation
import Keystore
import NameService
import PerpetualService
import Preferences
import PriceAlertService
import PriceService
import PaymentService
import Primitives
import SigningRequestService
import PrimitivesComponents
import ScanService
import Stake
import StakeService
import Store
import Swap
import SwapService
import SwiftUI
import TransactionStateService
import Transfer
import WalletConnector
import WalletConnectorService
import WalletService
import WalletTab

public struct ViewModelFactory: Sendable {
    let keystore: any Keystore
    let chainServiceFactory: ChainServiceFactory
    let scanService: ScanService
    let swapService: SwapService
    let assetsEnabler: any AssetsEnabler
    let priceUpdater: any PriceUpdater
    let walletService: WalletService
    let stakeService: StakeService
    let earnService: EarnService
    let amountService: AmountService
    let nameService: NameService
    let balanceService: BalanceService
    let priceService: PriceService
    let transactionStateScheduler: TransactionStateScheduler
    let addressNameService: AddressNameService
    let activityService: ActivityService
    let eventPresenterService: EventPresenterService
    let fiatService: FiatService
    let assetsService: AssetsService
    let assetSearchService: AssetSearchService
    let priceAlertService: PriceAlertService
    let walletSearchService: WalletSearchService
    let perpetualService: PerpetualService

    public init(
        keystore: any Keystore,
        chainServiceFactory: ChainServiceFactory,
        scanService: ScanService,
        swapService: SwapService,
        assetsEnabler: any AssetsEnabler,
        priceUpdater: any PriceUpdater,
        walletService: WalletService,
        stakeService: StakeService,
        earnService: EarnService,
        amountService: AmountService,
        nameService: NameService,
        balanceService: BalanceService,
        priceService: PriceService,
        transactionStateScheduler: TransactionStateScheduler,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService,
        fiatService: FiatService,
        assetsService: AssetsService,
        assetSearchService: AssetSearchService,
        priceAlertService: PriceAlertService,
        walletSearchService: WalletSearchService,
        perpetualService: PerpetualService,
    ) {
        self.keystore = keystore
        self.chainServiceFactory = chainServiceFactory
        self.scanService = scanService
        self.swapService = swapService
        self.assetsEnabler = assetsEnabler
        self.priceUpdater = priceUpdater
        self.walletService = walletService
        self.stakeService = stakeService
        self.earnService = earnService
        self.amountService = amountService
        self.nameService = nameService
        self.balanceService = balanceService
        self.priceService = priceService
        self.transactionStateScheduler = transactionStateScheduler
        self.addressNameService = addressNameService
        self.activityService = activityService
        self.eventPresenterService = eventPresenterService
        self.fiatService = fiatService
        self.assetsService = assetsService
        self.assetSearchService = assetSearchService
        self.priceAlertService = priceAlertService
        self.walletSearchService = walletSearchService
        self.perpetualService = perpetualService
    }

    @MainActor
    public func selectAssetScene(
        wallet: Wallet,
        selectType: SelectAssetType,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) -> SelectAssetViewModel {
        SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            searchService: assetSearchService,
            assetsEnabler: assetsEnabler,
            priceAlertService: priceAlertService,
            activityService: activityService,
            selectAssetAction: selectAssetAction,
            chains: chains,
        )
    }

    @MainActor
    public func assetsResultsScene(
        wallet: Wallet,
        request: WalletSearchRequest,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) -> AssetsResultsSceneViewModel {
        AssetsResultsSceneViewModel(
            wallet: wallet,
            assetsEnabler: assetsEnabler,
            balanceService: balanceService,
            preferences: Preferences.standard,
            searchService: walletSearchService,
            perpetualService: perpetualService,
            activityService: activityService,
            request: request,
            title: title,
            onSelectAsset: onSelectAsset,
        )
    }

    @MainActor
    public func confirmTransferScene(
        wallet: Wallet,
        data: TransferData,
        confirmTransferDelegate: StringResultAction? = nil,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction,
    ) -> ConfirmTransferSceneViewModel {
        ConfirmTransferSceneViewModel(
            request: ConfirmTransferRequest(
                wallet: wallet,
                data: data,
                simulation: simulation,
                delegate: confirmTransferDelegate,
            ),
            confirmService: ConfirmServiceFactory.create(
                keystore: keystore,
                chainServiceFactory: chainServiceFactory,
                assetsEnabler: assetsEnabler,
                scanService: scanService,
                balanceService: balanceService,
                assetsService: assetsService,
                priceService: priceService,
                transactionStateScheduler: transactionStateScheduler,
                addressNameService: addressNameService,
                activityService: activityService,
                eventPresenterService: eventPresenterService,
                chain: data.chain,
            ),
            onComplete: onComplete,
        )
    }

    @MainActor
    public func recipientScene(
        wallet: Wallet,
        asset: Asset,
        type: RecipientAssetType,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            walletService: walletService,
            nameService: nameService,
            type: type,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func amountScene(
        input: AmountInput,
        wallet: Wallet,
        onTransferAction: TransferDataAction,
    ) -> AmountSceneViewModel {
        AmountSceneViewModel(
            input: input,
            wallet: wallet,
            service: amountService,
            fiatService: fiatService,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func fiatScene(
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            fiatService: fiatService,
            assetAddress: assetAddress,
            wallet: wallet,
            assetsEnabler: assetsEnabler,
            type: type,
            amount: amount,
        )
    }

    @MainActor
    public func swapScene(
        input: SwapInput,
        onSwap: @escaping (TransferData) -> Void,
    ) -> SwapSceneViewModel {
        SwapSceneViewModel(
            input: input,
            balanceUpdater: balanceService,
            priceUpdater: priceUpdater,
            swapQuotesProvider: SwapQuotesProvider(swapService: swapService),
            swapQuoteDataProvider: SwapQuoteDataProvider(keystore: keystore, swapService: swapService),
            onSwap: onSwap,
        )
    }

    @MainActor
    public func stakeScene(
        wallet: Wallet,
        chain: Chain,
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: StakeChain(rawValue: chain.rawValue)!, // Expected Only StakeChain accepted.
            currencyCode: Preferences.standard.currency,
            stakeService: stakeService,
        )
    }

    @MainActor
    public func earnScene(
        wallet: Wallet,
        asset: Asset,
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            currencyCode: Preferences.standard.currency,
            earnService: earnService,
        )
    }

    @MainActor
    public func delegationScene(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        validators: [DelegationValidator],
        onAmountInputAction: AmountInputAction,
        onTransferAction: TransferDataAction,
    ) -> DelegationSceneViewModel {
        DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(delegation: delegation, asset: asset, formatter: .auto, currencyCode: Preferences.standard.currency),
            asset: asset,
            validators: validators,
            onAmountInputAction: onAmountInputAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func signMessageScene(
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping StringResultAction,
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            keystore: keystore,
            addressNameService: addressNameService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
