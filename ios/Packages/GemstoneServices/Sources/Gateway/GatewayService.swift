// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import NativeProviderService
import Primitives

public actor GatewayService: Sendable {
    let gateway: GemGateway
    private let nodes: GemNodeService
    private let preferences: any GemPreferencesStore
    private let securePreferences: any GemSecureStore

    public init(
        provider: NativeProvider,
        nodes: GemNodeService,
        preferences: any GemPreferencesStore,
        securePreferences: any GemSecureStore,
    ) {
        self.nodes = nodes
        self.preferences = preferences
        self.securePreferences = securePreferences
        gateway = GemGateway(
            provider: provider,
            nodes: nodes,
            preferences: preferences,
            securePreferences: securePreferences,
        )
    }

    public nonisolated func with(provider: NativeProvider) -> GatewayService {
        GatewayService(provider: provider, nodes: nodes, preferences: preferences, securePreferences: securePreferences)
    }

    public nonisolated func chainSettingsService(nodes: GemNodeService, explorer: GemExplorerService) -> GemChainSettingsService {
        GemChainSettingsService(nodes: nodes, explorer: explorer, gateway: gateway)
    }

    public nonisolated func stakeService(
        staticApi: GemStaticApiClient,
        store: any GemStakeStore,
        names: GemNameService,
        explorer: GemExplorerService,
        preferences: GemPreferencesService,
        session: GemWalletSessionService,
    ) -> GemStakeService {
        GemStakeService(gateway: gateway, staticApi: staticApi, store: store, names: names, explorer: explorer, preferences: preferences, session: session)
    }

    public nonisolated func transactionStateService(
        store: any GemTransactionStateStore,
        assets: GemAssetsService,
        balance: GemBalanceService,
        stake: GemStakeService,
        nft: GemNftService,
        payments: GemPaymentService,
    ) -> GemTransactionStateService {
        GemTransactionStateService(gateway: gateway, store: store, assets: assets, balance: balance, stake: stake, nft: nft, payments: payments)
    }

    public nonisolated func balanceService(
        store: any GemBalanceStore,
        assets: GemAssetsService,
        session: GemWalletSessionService,
        stream: GemStreamSubscriptionService,
    ) -> GemBalanceService {
        GemBalanceService(gateway: gateway, store: store, assets: assets, session: session, stream: stream)
    }

    public nonisolated func assetsService(
        api: GemApiClient,
        store: any GemAssetStore,
        price: GemPriceService,
        preferences: GemPreferencesService,
        session: GemWalletSessionService,
    ) -> GemAssetsService {
        GemAssetsService(api: api, gateway: gateway, store: store, price: price, preferences: preferences, session: session)
    }

    public nonisolated func perpetualService(
        price: GemPriceService,
        store: any GemPerpetualStore,
        assets: GemAssetsService,
        preferences: GemPreferencesService,
        balance: GemBalanceService,
        walletPreferences: GemWalletPreferencesService,
        session: GemWalletSessionService,
        recentActivity: GemRecentActivityService,
    ) -> GemPerpetualService {
        GemPerpetualService(
            gateway: gateway,
            price: price,
            store: store,
            assets: assets,
            preferences: preferences,
            balance: balance,
            walletPreferences: walletPreferences,
            session: session,
            recentActivity: recentActivity,
        )
    }

    public nonisolated func confirmService(
        simulation: GemSimulationService,
        scanner: GemScanService,
        transactionState: GemTransactionStateService,
        balance: GemBalanceService,
        price: GemPriceService,
        assets: GemAssetsService,
        transactionStatus: any GemTransactionStatusService,
    ) -> GemConfirmService {
        GemConfirmService(gateway: gateway, simulation: simulation, scanner: scanner, transactionState: transactionState, balance: balance, price: price, assets: assets, transactionStatus: transactionStatus)
    }
}
