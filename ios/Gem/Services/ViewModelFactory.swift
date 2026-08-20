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
import Payments
import PaymentService
import PerpetualService
import Preferences
import PriceAlertService
import PriceService
import Primitives
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
import WalletSessionService
import WalletTab

public struct ViewModelFactory: Sendable {
    let keystore: any Keystore
    let chainServiceFactory: ChainServiceFactory
    let scanService: ScanService
    let swapService: SwapService
    let paymentService: PaymentService
    let assetsEnabler: any AssetsEnabler
    let priceUpdater: any PriceUpdater
    let walletSessionService: any WalletSessionManageable
    let stakeService: StakeService
    let earnService: EarnService
    let amountService: AmountService
    let nameService: any NameServiceable
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
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
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
                paymentService: paymentService,
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
        recipient: RecipientData? = .none,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            walletSessionService: walletSessionService,
            nameService: nameService,
            type: type,
            recipient: recipient,
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
    public func paymentScene(
        wallet: Wallet,
        link: PaymentLink,
        quotes: PaymentQuotes,
        onTransferAction: TransferDataAction,
        onComplete: VoidAction,
    ) -> PaymentSceneViewModel {
        PaymentSceneViewModel(
            wallet: wallet,
            link: link,
            quotes: quotes,
            paymentService: paymentService,
            balanceService: balanceService,
            onTransferAction: onTransferAction,
            onComplete: onComplete,
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
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            keystore: keystore,
            addressNameService: addressNameService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
