// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemFiatServiceProtocol
import GemstoneServices
import Assets
import FiatConnect
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import class Gemstone.GemConfirmService
import Keystore
import Preferences
import Primitives
import PrimitivesComponents
import Stake
import Store
import Swap
import SwiftUI
import Transfer
import WalletConnector
import WalletConnectorService
import WalletTab

public struct ViewModelFactory: Sendable {
    let keystore: any Keystore
    let chainServiceFactory: ChainServiceFactory
    let gemConfirmService: GemConfirmService
    let swapService: SwapService
    let assetsEnabler: any AssetsEnabler
    let priceUpdater: any PriceUpdater
    let walletSessionService: any WalletSessionManageable
    let stakeService: any GemStakeServiceProtocol
    let explorerService: any GemExplorerServiceProtocol
    let amountService: AmountService
    let nameService: any NameServiceable
    let balanceService: any GemBalanceServiceProtocol
    let balanceStore: BalanceStore
    let addressStore: AddressStore
    let priceService: any GemPriceServiceProtocol
    let priceStore: PriceStore
    let transactionStateScheduler: TransactionStateScheduler
    let gemNameService: any GemNameServiceProtocol
    let activityService: ActivityService
    let toastPresenter: ToastPresenter
    let fiatService: any GemFiatServiceProtocol
    let assetsService: any GemAssetsServiceProtocol
    let assetStore: AssetStore
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
                explorerService: explorerService,
                keystore: keystore,
                chainServiceFactory: chainServiceFactory,
                assetsEnabler: assetsEnabler,
                gemConfirmService: gemConfirmService,
                balanceStore: balanceStore,
                assetStore: assetStore,
                assetsService: assetsService,
                priceStore: priceStore,
                transactionStateScheduler: transactionStateScheduler,
                nameService: gemNameService,
                addressStore: addressStore,
                activityService: activityService,
                toastPresenter: toastPresenter,
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
            balanceService: balanceService,
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
            explorerService: explorerService,
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
            stakeService: stakeService,
            explorerService: explorerService,
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
            model: DelegationViewModel(explorerService: explorerService, delegation: delegation, asset: asset, formatter: .auto, currencyCode: Preferences.standard.currency),
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
            explorerService: explorerService,
            keystore: keystore,
            nameService: gemNameService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
