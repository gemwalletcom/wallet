package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetAssetById
import com.gemwallet.android.application.assets.cases.GetAssetChartData
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetChartPeriod
import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.application.assets.cases.GetImportInProgress
import com.gemwallet.android.application.assets.cases.GetSearchLists
import com.gemwallet.android.application.assets.cases.GetPortfolioData
import com.gemwallet.android.application.assets.cases.GetShowWelcomeBanner
import uniffi.gemstone.GemBannerService
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.assets.cases.HideAsset
import com.gemwallet.android.application.assets.cases.HideWelcomeBanner
import com.gemwallet.android.application.assets.cases.SyncMissingAssets
import com.gemwallet.android.application.assets.cases.SetChartPeriod
import com.gemwallet.android.application.assets.cases.SyncAssetInfo
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.assets.cases.SetAssetPinned
import com.gemwallet.android.application.assets.cases.ToggleHideBalances
import com.gemwallet.android.application.wallet_import.cases.GetImportWalletState
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.coordinators.asset.EnableAssetImpl
import com.gemwallet.android.data.coordinators.asset.GetActiveAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetByIdImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetChartDataImpl
import uniffi.gemstone.GemChartService
import com.gemwallet.android.data.coordinators.asset.GetAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetLinksImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetMarketImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetTokenInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetChainAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetChartPeriodImpl
import com.gemwallet.android.data.coordinators.asset.GetHideBalancesStateImpl
import com.gemwallet.android.data.coordinators.asset.GetImportInProgressImpl
import com.gemwallet.android.data.coordinators.asset.GetSearchListsImpl
import com.gemwallet.android.data.coordinators.asset.GetPortfolioDataImpl
import com.gemwallet.android.data.coordinators.asset.GetShowWelcomeBannerImpl
import com.gemwallet.android.data.coordinators.asset.GetWalletSummaryImpl
import com.gemwallet.android.data.coordinators.asset.DeviceAssetsSyncService
import com.gemwallet.android.data.coordinators.asset.HideAssetImpl
import com.gemwallet.android.data.coordinators.asset.HideWelcomeBannerImpl
import com.gemwallet.android.data.coordinators.asset.SyncMissingAssetsImpl
import com.gemwallet.android.data.coordinators.asset.SetChartPeriodImpl
import com.gemwallet.android.data.coordinators.asset.SyncAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.SyncAssetsImpl
import com.gemwallet.android.data.coordinators.asset.SetAssetPinnedImpl
import com.gemwallet.android.data.coordinators.asset.ToggleHideBalancesImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import uniffi.gemstone.GemAssetDiscoveryService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemDeviceApiClient
import dagger.Module
import uniffi.gemstone.GemPerpetualService
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemPortfolioService
import javax.inject.Singleton
import uniffi.gemstone.GemStreamSubscriptionService
import uniffi.gemstone.GemWalletPreferencesService

@InstallIn(SingletonComponent::class)
@Module
object AssetModule {
    @Provides
    @Singleton
    fun provideGetActiveAssetsInfo(assetsRepository: AssetsRepository): GetActiveAssetsInfo =
        GetActiveAssetsInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetSearchLists(searchService: AssetsSearchService): GetSearchLists =
        GetSearchListsImpl(searchService)

    @Provides
    @Singleton
    fun provideGetAssetTokenInfo(assetsRepository: AssetsRepository): GetAssetTokenInfo =
        GetAssetTokenInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetChainAssetInfo(assetsRepository: AssetsRepository): GetChainAssetInfo =
        GetChainAssetInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetAssetById(assetsRepository: AssetsRepository): GetAssetById =
        GetAssetByIdImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetAssetInfo(assetsRepository: AssetsRepository): GetAssetInfo =
        GetAssetInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetAssetLinks(assetsRepository: AssetsRepository): GetAssetLinks =
        GetAssetLinksImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetAssetMarket(assetsRepository: AssetsRepository): GetAssetMarket =
        GetAssetMarketImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetWalletSummary(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
        perpetualRepository: PerpetualRepository,
        observePerpetualWallet: ObservePerpetualWallet,
        hasMultiSign: HasMultiSign,
        userConfig: UserConfig,
        walletPreferencesService: GemWalletPreferencesService,
    ): GetWalletSummary = GetWalletSummaryImpl(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        perpetualRepository = perpetualRepository,
        observePerpetualWallet = observePerpetualWallet,
        hasMultiSign = hasMultiSign,
        userConfig = userConfig,
        walletPreferencesService = walletPreferencesService,
    )

    @Provides
    @Singleton
    fun provideGetAssetChartData(
        chartService: GemChartService,
    ): GetAssetChartData = GetAssetChartDataImpl(
        chartService = chartService,
    )

    @Provides
    @Singleton
    fun provideGetPortfolioData(
        portfolioService: GemPortfolioService,
        perpetualService: GemPerpetualService,
        sessionRepository: SessionRepository,
    ): GetPortfolioData = GetPortfolioDataImpl(
        portfolioService = portfolioService,
        perpetualService = perpetualService,
        sessionRepository = sessionRepository,
    )

    @Provides
    @Singleton
    fun provideSyncMissingAssets(
        assetsService: GemAssetsService,
    ): SyncMissingAssets = SyncMissingAssetsImpl(
        assetsService = assetsService,
    )

    @Provides
    @Singleton
    fun provideEnableAsset(
        balanceService: GemBalanceService,
    ): EnableAsset = EnableAssetImpl(balanceService)

    @Provides
    @Singleton
    fun provideSyncAssetInfo(
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        streamSubscriptionService: GemStreamSubscriptionService,
        syncMissingAssets: SyncMissingAssets,
        sessionRepository: SessionRepository,
    ): SyncAssetInfo = SyncAssetInfoImpl(
        assetsService = assetsService,
        balanceService = balanceService,
        streamSubscriptionService = streamSubscriptionService,
        syncMissingAssets = syncMissingAssets,
        sessionRepository = sessionRepository,
    )

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
    @Singleton
    fun provideSyncAssets(
        sessionRepository: SessionRepository,
        deviceAssetsSyncService: DeviceAssetsSyncService,
        assetsRepository: AssetsRepository,
    ): SyncAssets = SyncAssetsImpl(
        sessionRepository = sessionRepository,
        deviceAssetsSyncService = deviceAssetsSyncService,
        assetsRepository = assetsRepository,
    )

    @Provides
    @Singleton
    fun provideHideAsset(
        sessionRepository: SessionRepository,
        enableAsset: EnableAsset,
    ): HideAsset = HideAssetImpl(sessionRepository, enableAsset)

    @Provides
    @Singleton
    fun provideSetAssetPinned(
        sessionRepository: SessionRepository,
        balanceService: GemBalanceService,
    ): SetAssetPinned = SetAssetPinnedImpl(sessionRepository, balanceService)

    @Provides
    @Singleton
    fun provideGetShowWelcomeBanner(
        sessionRepository: SessionRepository,
        bannersDao: BannersDao,
        bannerService: GemBannerService,
        getActiveAssetsInfo: GetActiveAssetsInfo,
    ): GetShowWelcomeBanner = GetShowWelcomeBannerImpl(sessionRepository, bannersDao, bannerService, getActiveAssetsInfo)

    @Provides
    @Singleton
    fun provideHideWelcomeBanner(
        sessionRepository: SessionRepository,
        applyBannerAction: ApplyBannerAction,
    ): HideWelcomeBanner = HideWelcomeBannerImpl(sessionRepository, applyBannerAction)

    @Provides
    @Singleton
    fun provideGetHideBalancesState(
        userConfig: UserConfig,
    ): GetHideBalancesState = GetHideBalancesStateImpl(userConfig)

    @Provides
    @Singleton
    fun provideToggleHideBalances(
        userConfig: UserConfig,
    ): ToggleHideBalances = ToggleHideBalancesImpl(userConfig)

    @Provides
    @Singleton
    fun provideGetChartPeriod(
        userConfig: UserConfig,
    ): GetChartPeriod = GetChartPeriodImpl(userConfig)

    @Provides
    @Singleton
    fun provideSetChartPeriod(
        userConfig: UserConfig,
    ): SetChartPeriod = SetChartPeriodImpl(userConfig)

    @Provides
    @Singleton
    fun provideGetImportInProgress(
        sessionRepository: SessionRepository,
        getImportWalletState: GetImportWalletState,
    ): GetImportInProgress = GetImportInProgressImpl(sessionRepository, getImportWalletState)
}
