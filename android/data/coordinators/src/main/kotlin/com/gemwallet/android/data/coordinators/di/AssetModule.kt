package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetAssetById
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.application.assets.cases.GetSearchLists
import uniffi.gemstone.GemBannerService
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.banner.cases.HasMultiSign
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.data.coordinators.asset.EnableAssetImpl
import com.gemwallet.android.data.coordinators.asset.GetActiveAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetByIdImpl
import uniffi.gemstone.BalanceCalculator
import com.gemwallet.android.data.coordinators.asset.GetAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetLinksImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetMarketImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetTokenInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetChainAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetHideBalancesStateImpl
import com.gemwallet.android.data.coordinators.asset.GetSearchListsImpl
import com.gemwallet.android.data.coordinators.asset.GetWalletSummaryImpl
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemWalletHomeService
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.coordinators.asset.WalletAssetsCoordinator
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.coordinators.asset.GetWidgetAssetsImpl
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.application.assets.cases.GetWidgetAssets
import com.gemwallet.android.application.session.cases.GetCurrentWallet

@InstallIn(SingletonComponent::class)
@Module
object AssetModule {
    @Provides
    @Singleton
    fun provideGetActiveAssetsInfo(getWalletAssets: GetWalletAssets): GetActiveAssetsInfo =
        GetActiveAssetsInfoImpl(getWalletAssets)

    @Provides
    @Singleton
    fun provideGetSearchLists(searchService: AssetsSearchService): GetSearchLists =
        GetSearchListsImpl(searchService)

    @Provides
    @Singleton
    fun provideGetAssetTokenInfo(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetAssetTokenInfo =
        GetAssetTokenInfoImpl(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideGetChainAssetInfo(getAssetTokenInfo: GetAssetTokenInfo): GetChainAssetInfo =
        GetChainAssetInfoImpl(getAssetTokenInfo)

    @Provides
    @Singleton
    fun provideGetAssetById(assetStore: GemstoneAssetStore): GetAssetById = GetAssetByIdImpl(assetStore)

    @Provides
    @Singleton
    fun provideGetAssetInfo(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetAssetInfo =
        GetAssetInfoImpl(assetStore, getCurrentWalletId)

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
        getWalletAssets: GetWalletAssets,
        getPerpetualBalance: GetPerpetualBalance,
        hasMultiSign: HasMultiSign,
        userConfig: UserConfig,
        balanceCalculator: BalanceCalculator,
    ): GetWalletSummary = GetWalletSummaryImpl(
        getSession = getSession,
        getWalletAssets = getWalletAssets,
        getPerpetualBalance = getPerpetualBalance,
        hasMultiSign = hasMultiSign,
        userConfig = userConfig,
        balanceCalculator = balanceCalculator,
    )



    @Provides
    @Singleton
    fun provideEnableAsset(
        balanceService: GemBalanceService,
    ): EnableAsset = EnableAssetImpl(balanceService)

    @Provides
    @Singleton
    fun provideGemAssetDiscoveryService(
        apiClient: GemDeviceApiClient,
        balanceService: GemBalanceService,
        transactionsService: GemTransactionsService,
        nftService: GemNftService,
        walletStore: GemstoneWalletStore,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemAssetDiscoveryService = GemAssetDiscoveryService(
        apiClient,
        balanceService,
        transactionsService,
        nftService,
        walletStore,
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
    fun provideGetWidgetAssets(searchTokensCase: SearchTokens, getWalletAssets: GetWalletAssets): GetWidgetAssets =
        GetWidgetAssetsImpl(searchTokensCase, getWalletAssets)

    @Provides
    @Singleton
    fun provideGetWalletAssets(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetWalletAssets =
        WalletAssetsCoordinator(assetStore, getCurrentWalletId)

    @Provides
    @Singleton
    fun provideGetHideBalancesState(
        userConfig: UserConfig,
    ): GetHideBalancesState = GetHideBalancesStateImpl(userConfig)


}
