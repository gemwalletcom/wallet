// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemPriceAlertServiceProtocol
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemSearchServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemFiatServiceProtocol
import GemstoneServices
import Assets
import FiatConnect
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import protocol Gemstone.GemSwapServiceProtocol
import class Gemstone.GemConfirmService
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
    let swapService: any GemSwapServiceProtocol
    let assetsEnabler: any AssetsEnabler
    let priceUpdater: any PriceUpdater
    let walletSessionService: any WalletSessionManageable
    let stakeService: any GemStakeServiceProtocol
    let explorerService: any GemExplorerServiceProtocol
    let preferencesService: any GemPreferencesServiceProtocol
    let amountService: AmountService
    let nameService: any GemNameServiceProtocol
    let balanceService: any GemBalanceServiceProtocol
    let balanceStore: BalanceStore
    let addressStore: AddressStore
    let priceService: any GemPriceServiceProtocol
    let priceStore: PriceStore
    let transactionStateScheduler: TransactionStateScheduler
    let gemNameService: any GemNameServiceProtocol
    let recentActivityStore: RecentActivityStore
    let toastPresenter: ToastPresenter
    let fiatService: any GemFiatServiceProtocol
    let assetsService: any GemAssetsServiceProtocol
    let assetStore: AssetStore
    let priceAlertService: any GemPriceAlertServiceProtocol
    let searchService: any GemSearchServiceProtocol
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
            searchService: searchService,
            assetsEnabler: assetsEnabler,
            priceAlertService: priceAlertService,
            recentActivityStore: recentActivityStore,
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
            searchService: searchService,
            perpetualService: perpetualService,
            recentActivityStore: recentActivityStore,
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
                gemConfirmService: gemConfirmService,
                preferencesService: preferencesService,
                balanceStore: balanceStore,
                assetStore: assetStore,
                assetsService: assetsService,
                priceStore: priceStore,
                transactionStateScheduler: transactionStateScheduler,
                nameService: gemNameService,
                addressStore: addressStore,
                recentActivityStore: recentActivityStore,
                toastPresenter: toastPresenter,
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
            swapService: swapService,
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
