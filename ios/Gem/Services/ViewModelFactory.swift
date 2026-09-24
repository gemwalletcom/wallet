// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAmountService
import class Gemstone.GemApiClient
import class Gemstone.GemAppUpdateService
import class Gemstone.GemAssetDiscoveryService
import class Gemstone.GemAssetsService
import class Gemstone.GemAvatarService
import class Gemstone.GemBalanceService
import class Gemstone.GemBannerService
import class Gemstone.GemChainService
import class Gemstone.GemConfirmService
import class Gemstone.GemContactEditorService
import class Gemstone.GemContactService
import class Gemstone.GemDeeplinkService
import class Gemstone.GemDeveloperService
import class Gemstone.GemDeviceService
import class Gemstone.GemExplorerService
import class Gemstone.GemFiatService
import class Gemstone.GemNameService
import class Gemstone.GemNftService
import class Gemstone.GemNodeService
import protocol Gemstone.GemNotificationPermissions
import class Gemstone.GemNotificationService
import class Gemstone.GemPaymentService
import class Gemstone.GemPerpetualService
import class Gemstone.GemPortfolioService
import class Gemstone.GemPreferencesService
import class Gemstone.GemPriceAlertService
import class Gemstone.GemPriceService
import class Gemstone.GemRecentActivityService
import class Gemstone.GemRewardsService
import class Gemstone.GemSearchService
import class Gemstone.GemServiceStatus
import class Gemstone.GemSignMessageService
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemStakeService
import class Gemstone.GemStreamSubscriptionService
import protocol Gemstone.GemSupportServiceProtocol
import class Gemstone.GemSwapService
import class Gemstone.GemTransactionsService
import class Gemstone.GemWalletConnectService
import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

public struct ViewModelFactory: Sendable {
    let apiClient: GemApiClient
    let assetDiscoveryService: GemAssetDiscoveryService
    let assetsService: GemAssetsService
    let avatarService: GemAvatarService
    let bannerService: GemBannerService
    let balanceService: GemBalanceService
    let confirmService: GemConfirmService
    let contactService: GemContactService
    let contactEditorService: GemContactEditorService
    let deeplinkService: GemDeeplinkService
    let explorerService: GemExplorerService
    let fiatService: GemFiatService
    let gatewayService: GatewayService
    let hyperliquidObserverService: any PerpetualObservable
    let nameService: GemNameService
    let nftService: GemNftService
    let nodeService: GemNodeService
    let paymentService: GemPaymentService
    let perpetualService: GemPerpetualService
    let portfolioService: GemPortfolioService
    let preferencesService: GemPreferencesService
    let priceAlertService: GemPriceAlertService
    let priceService: GemPriceService
    let rewardsService: GemRewardsService
    let searchService: GemSearchService
    let simulationFormatter: GemSimulationFormatter
    let stakeService: GemStakeService
    let streamSubscriptionService: GemStreamSubscriptionService
    let swapService: GemSwapService
    let transactionsService: GemTransactionsService
    let walletService: GemWalletService
    let walletSessionService: GemWalletSessionService
    let walletConnectService: GemWalletConnectService
    let serviceStatusService: GemServiceStatus
    let appUpdateService: GemAppUpdateService
    let inAppNotificationService: GemNotificationService

    let biometryService: any BiometryAuthenticatable
    let keystore: any Keystore
    let observablePreferences: ObservablePreferences
    let recentAssetsService: GemRecentActivityService
    let amountService: GemAmountService
    let toastPresenter: ToastPresenter
    let walletPreferencesService: GemWalletPreferencesService
    let signMessageService: GemSignMessageService
    let devicePlatform: GemstoneDevicePlatform
    let developerService: GemDeveloperService
    let deviceService: GemDeviceService
    let notificationPermissions: any GemNotificationPermissions
    let stores: Stores
    let supportService: any GemSupportServiceProtocol
    let supportTyping: ObservableSupportTyping

    func currentWallets() -> [Wallet] {
        (try? stores.walletStore.getWallets()) ?? []
    }

    func currentWallet(in wallets: [Wallet]) -> Wallet? {
        walletSessionService.currentWalletId.flatMap { walletId in wallets.first { $0.id == walletId } }
    }
}
