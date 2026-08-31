// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
import class Gemstone.GemAmountService
import class Gemstone.GemApplicationMetadataService
import class Gemstone.GemAssetConfigService
import class Gemstone.GemAssetsService
import class Gemstone.GemAvatarService
import class Gemstone.GemBalanceService
import class Gemstone.GemChainService
import class Gemstone.GemConfirmSceneService
import class Gemstone.GemConfirmService
import class Gemstone.GemContactService
import class Gemstone.GemDeeplinkService
import class Gemstone.GemExplorerService
import class Gemstone.GemFeeService
import class Gemstone.GemFiatService
import class Gemstone.GemManageContactService
import class Gemstone.GemNameService
import class Gemstone.GemOnboardingService
import class Gemstone.GemPaymentService
import class Gemstone.GemPerpetualService
import class Gemstone.GemPreferencesService
import class Gemstone.GemPriceAlertService
import class Gemstone.GemPriceService
import class Gemstone.GemSearchService
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemStakeService
import class Gemstone.GemStreamSubscriptionService
import class Gemstone.GemSwapQuoteService
import class Gemstone.GemSwapService
import class Gemstone.GemTransactionStateService
import class Gemstone.GemTransferService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import Assets
import Contacts
import FiatConnect
import Foundation
import GemstoneServices
import ManageWallets
import Onboarding
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
    // Core services
    let addressService: GemAddressService
    let applicationMetadataService: GemApplicationMetadataService
    let assetConfig: GemAssetConfigService
    let assetsService: GemAssetsService
    let avatarService: GemAvatarService
    let balanceService: GemBalanceService
    let chainService: GemChainService
    let confirmService: GemConfirmService
    let contactService: GemContactService
    let deeplinkService: GemDeeplinkService
    let explorerService: GemExplorerService
    let feeService: GemFeeService
    let fiatService: GemFiatService
    let nameService: GemNameService
    let onboardingService: GemOnboardingService
    let paymentService: GemPaymentService
    let perpetualService: GemPerpetualService
    let preferencesService: GemPreferencesService
    let priceAlertService: GemPriceAlertService
    let priceService: GemPriceService
    let searchService: GemSearchService
    let simulationFormatter: GemSimulationFormatter
    let stakeService: GemStakeService
    let streamSubscriptionService: GemStreamSubscriptionService
    let swapQuoteService: GemSwapQuoteService
    let swapService: GemSwapService
    let transactionStateService: GemTransactionStateService
    let transferService: GemTransferService
    let walletService: GemWalletService
    let walletSessionService: GemWalletSessionService

    // Platform services Core cannot own
    let keystore: any Keystore
    let observablePreferences: ObservablePreferences
    let recentAssetsService: RecentAssetsService
    let amountService: AmountService
    let toastPresenter: ToastPresenter

    // Stores
    let addressStore: AddressStore
    let assetStore: AssetStore

    @MainActor
    public func walletsScene(
        navigationPath: Binding<NavigationPath>,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            session: walletSessionService,
            preferences: observablePreferences,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }

    @MainActor
    public func walletDetailScene(navigationPath: Binding<NavigationPath>, wallet: Wallet) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: navigationPath,
            wallet: wallet,
            walletService: walletService,
            keystore: keystore,
            preferences: observablePreferences,
            explorerService: explorerService,
        )
    }

    @MainActor
    public func walletImageScene(wallet: Wallet) -> WalletImageViewModel {
        WalletImageViewModel(wallet: wallet, avatarService: avatarService)
    }

    @MainActor
    public func createWalletScene(onComplete: VoidAction) -> CreateWalletModel {
        CreateWalletModel(service: onboardingService, preferences: observablePreferences, onComplete: onComplete)
    }

    @MainActor
    public func importWalletScene(onComplete: VoidAction) -> ImportWalletViewModel {
        ImportWalletViewModel(service: onboardingService, preferences: observablePreferences, onComplete: onComplete)
    }

    @MainActor
    public func contactsScene(mode: ContactsViewModel.Mode = .list) -> ContactsViewModel {
        ContactsViewModel(service: manageContactService(), mode: mode)
    }

    @MainActor
    public func manageContactScene(mode: ManageContactViewModel.Mode) -> ManageContactViewModel {
        ManageContactViewModel(service: manageContactService(), mode: mode)
    }

    private func manageContactService() -> GemManageContactService {
        GemManageContactService(
            contacts: contactService,
            names: nameService,
            addresses: addressService,
            chains: chainService,
        )
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
            searchService: searchService,
            balanceService: balanceService,
            priceAlertService: priceAlertService,
            recentAssetsService: recentAssetsService,
            preferencesService: preferencesService,
            assetConfig: assetConfig,
            chainService: chainService,
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
            balanceService: balanceService,
            preferencesService: preferencesService,
            searchService: searchService,
            perpetualService: perpetualService,
            recentAssetsService: recentAssetsService,
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
            service: confirmSceneService(),
            signer: KeystoreTransactionSigner(keystore: keystore),
            keystore: keystore,
            recentAssetsService: recentAssetsService,
            toastPresenter: toastPresenter,
            preferencesService: preferencesService,
            onComplete: onComplete,
        )
    }

    private func confirmSceneService() -> GemConfirmSceneService {
        GemConfirmSceneService(
            confirm: confirmService,
            explorer: explorerService,
            names: nameService,
            assetConfig: assetConfig,
            transfer: transferService,
            fee: feeService,
            swapQuote: swapQuoteService,
            applicationMetadata: applicationMetadataService,
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
            addressService: addressService,
            paymentService: paymentService,
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
            preferencesService: preferencesService,
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
            balanceService: balanceService,
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
            preferencesService: preferencesService,
            input: input,
            balanceService: balanceService,
            priceUpdater: streamSubscriptionService,
            swapService: swapService,
            swapQuoteService: swapQuoteService,
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
            currencyCode: preferencesService.currencyCode,
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
            currencyCode: preferencesService.currencyCode,
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
            model: DelegationViewModel(explorerService: explorerService, stakeService: stakeService, delegation: delegation, asset: asset, formatter: .auto, currencyCode: preferencesService.currencyCode),
            asset: asset,
            stakeService: stakeService,
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
            nameService: nameService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
            applicationMetadataService: applicationMetadataService,
        )
    }
}
