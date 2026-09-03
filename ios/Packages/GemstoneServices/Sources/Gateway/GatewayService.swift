// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import NativeProviderService
import Primitives

public actor GatewayService: Sendable {
    let gateway: GemGateway
    private let preferences: any GemPreferencesStore
    private let securePreferences: any GemSecureStore

    public init(
        provider: NativeProvider,
        preferences: any GemPreferencesStore,
        securePreferences: any GemSecureStore,
    ) {
        self.preferences = preferences
        self.securePreferences = securePreferences
        gateway = GemGateway(
            provider: provider,
            preferences: preferences,
            securePreferences: securePreferences,
        )
    }

    public nonisolated func with(provider: NativeProvider) -> GatewayService {
        GatewayService(provider: provider, preferences: preferences, securePreferences: securePreferences)
    }

    public nonisolated func chainSettingsService(nodes: GemNodeService, explorer: GemExplorerService) -> GemChainSettingsService {
        GemChainSettingsService(nodes: nodes, explorer: explorer, gateway: gateway)
    }

    public nonisolated func stakeService(
        staticApi: GemStaticApiClient,
        store: any GemStakeStore,
        addressStore: any GemAddressStore,
        explorer: GemExplorerService,
        preferences: GemPreferencesService,
        session: GemWalletSessionService,
    ) -> GemStakeService {
        GemStakeService(gateway: gateway, staticApi: staticApi, store: store, addressStore: addressStore, explorer: explorer, preferences: preferences, session: session)
    }

    public nonisolated func transactionStateService(
        store: any GemTransactionStateStore,
        assets: GemAssetsService,
        balance: GemBalanceService,
        stake: GemStakeService,
        nft: GemNftService,
    ) -> GemTransactionStateService {
        GemTransactionStateService(gateway: gateway, store: store, assets: assets, balance: balance, stake: stake, nft: nft)
    }

    public nonisolated func balanceService(
        walletStore: any GemWalletStore,
        assetStore: any GemAssetStore,
        store: any GemBalanceStore,
        assets: GemAssetsService,
        price: GemPriceService,
        stream: GemStreamSubscriptionService,
        preferences: GemPreferencesService,
    ) -> GemBalanceService {
        GemBalanceService(gateway: gateway, walletStore: walletStore, assetStore: assetStore, store: store, assets: assets, price: price, stream: stream, preferences: preferences)
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
        assetStore: any GemAssetStore,
        preferences: GemPreferencesService,
        balance: GemBalanceService,
        walletPreferences: GemWalletPreferencesService,
        session: GemWalletSessionService,
    ) -> GemPerpetualService {
        GemPerpetualService(
            gateway: gateway,
            price: price,
            store: store,
            assetStore: assetStore,
            preferences: preferences,
            balance: balance,
            walletPreferences: walletPreferences,
            session: session,
        )
    }

    public nonisolated func confirmService(
        simulation: TransactionSimulationService,
        scanner: GemScanService,
        transactionState: GemTransactionStateService,
        balance: GemBalanceService,
        price: GemPriceService,
        assets: GemAssetsService,
    ) -> GemConfirmService {
        GemConfirmService(gateway: gateway, simulation: simulation, scanner: scanner, transactionState: transactionState, balance: balance, price: price, assets: assets)
    }
}

// MARK: - Account

extension GatewayService {
    func utxos(chain: Primitives.Chain, address: String) async throws -> [Primitives.UTXO] {
        try await gateway.getUtxos(chain: chain.rawValue, address: address).map {
            try Primitives.UTXO($0)
        }
    }
}

// TransactionPreload

// MARK: - State

public extension GatewayService {
    func chainId(chain: Primitives.Chain) async throws -> String {
        try await gateway.getChainId(chain: chain.rawValue)
    }

    func latestBlock(chain: Primitives.Chain) async throws -> BigInt {
        try await gateway.getBlockNumber(chain: chain.rawValue).asBigInt
    }
}

// MARK: - Staking

public extension GatewayService {
    func validators(chain: Primitives.Chain, apy: Double) async throws -> [Primitives.DelegationValidator] {
        try await gateway.getStakingValidators(chain: chain.rawValue, apy: apy)
            .map { try Primitives.DelegationValidator($0) }
    }

    func delegationValidators(chain: Primitives.Chain, address: String) async throws -> [Primitives.DelegationValidator] {
        try await gateway.getStakingDelegationValidators(chain: chain.rawValue, address: address)
            .map { try Primitives.DelegationValidator($0) }
    }

    func delegations(chain: Primitives.Chain, address: String) async throws -> [Primitives.DelegationBase] {
        try await gateway.getStakingDelegations(chain: chain.rawValue, address: address)
            .map { try Primitives.DelegationBase($0) }
    }
}

// MARK: - Perpetual

public extension GatewayService {
    func getPerpetualAccountMode(chain: Primitives.Chain, address: String) async throws -> Primitives.PerpetualAccountMode {
        try await Primitives.PerpetualAccountMode(gateway.getPerpetualAccountMode(chain: chain.rawValue, address: address))
    }

    func getPerpetualCandlesticks(chain: Primitives.Chain, symbol: String, period: Primitives.ChartPeriod) async throws -> [Primitives.ChartCandleStick] {
        try await gateway.getPerpetualCandlesticks(chain: chain.rawValue, symbol: symbol, period: period.rawValue).map {
            try Primitives.ChartCandleStick($0)
        }
    }

    func getPerpetualPortfolio(chain: Primitives.Chain, address: String) async throws -> Primitives.PerpetualPortfolio {
        try await Primitives.PerpetualPortfolio(gateway.getPerpetualPortfolio(chain: chain.rawValue, address: address))
    }
}
