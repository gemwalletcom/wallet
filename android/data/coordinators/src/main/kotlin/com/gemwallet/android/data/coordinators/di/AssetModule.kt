package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.coordinators.asset.GetActiveAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetLinksImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetMarketImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetTokenInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetChainAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetWalletSummaryImpl
import com.gemwallet.android.data.coordinators.asset.WalletAssetsCoordinator
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletHomeService
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AssetModule {
    @Provides
    @Singleton
    fun provideGetActiveAssetsInfo(getWalletAssets: GetWalletAssets, userConfig: UserConfig): GetActiveAssetsInfo = GetActiveAssetsInfoImpl(getWalletAssets, userConfig)

    @Provides
    @Singleton
    fun provideGetAssetTokenInfo(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetAssetTokenInfo = GetAssetTokenInfoImpl(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideGetChainAssetInfo(getAssetTokenInfo: GetAssetTokenInfo): GetChainAssetInfo = GetChainAssetInfoImpl(getAssetTokenInfo)

    @Provides
    @Singleton
    fun provideGetAssetInfo(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetAssetInfo = GetAssetInfoImpl(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideGetAssetLinks(assetStore: GemstoneAssetStore): GetAssetLinks = GetAssetLinksImpl(assetStore)

    @Provides
    @Singleton
    fun provideGetAssetMarket(assetStore: GemstoneAssetStore): GetAssetMarket = GetAssetMarketImpl(assetStore)

    @Provides
    @Singleton
    fun provideGetWalletSummary(
        getSession: GetSession,
        assetStore: GemstoneAssetStore,
        getPerpetualBalance: GetPerpetualBalance,
        bannerStore: GemstoneBannerStore,
        userConfig: UserConfig,
        walletHomeService: GemWalletHomeServiceInterface,
    ): GetWalletSummary = GetWalletSummaryImpl(
        getSession = getSession,
        assetStore = assetStore,
        getPerpetualBalance = getPerpetualBalance,
        bannerStore = bannerStore,
        userConfig = userConfig,
        walletHomeService = walletHomeService,
    )

    @Provides
    @Singleton
    fun provideGemAssetDiscoveryService(
        apiClient: GemDeviceApiClient,
        balanceService: GemBalanceService,
        transactionsService: GemTransactionsService,
        nftService: GemNftService,
        walletSessionService: GemWalletSessionService,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemAssetDiscoveryService = GemAssetDiscoveryService(
        apiClient,
        balanceService,
        transactionsService,
        nftService,
        walletSessionService,
        walletPreferencesService,
    )

    @Provides
    fun provideGemWalletHomeService(
        balanceService: GemBalanceService,
        discoveryService: GemAssetDiscoveryService,
        bannerService: GemBannerService,
        walletPreferencesService: GemWalletPreferencesService,
        preferencesService: GemPreferencesService,
        walletSessionService: GemWalletSessionService,
    ): GemWalletHomeServiceInterface = GemWalletHomeService(
        balanceService,
        discoveryService,
        bannerService,
        walletPreferencesService,
        preferencesService,
        walletSessionService,
    )

    @Provides
    @Singleton
    fun provideGetWalletAssets(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetWalletAssets = WalletAssetsCoordinator(assetStore, getCurrentWalletId)
}
