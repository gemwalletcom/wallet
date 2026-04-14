package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.application.assets.coordinators.GetImportInProgress
import com.gemwallet.android.application.assets.coordinators.GetShowWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.application.assets.coordinators.HideWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.application.assets.coordinators.ToggleHideBalances
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.coordinators.asset.GetActiveAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetAssetChartDataImpl
import com.gemwallet.android.data.coordinators.asset.GetHideBalancesStateImpl
import com.gemwallet.android.data.coordinators.asset.GetImportInProgressImpl
import com.gemwallet.android.data.coordinators.asset.GetShowWelcomeBannerImpl
import com.gemwallet.android.data.coordinators.asset.GetWalletSummaryImpl
import com.gemwallet.android.data.coordinators.asset.DeviceAssetsSyncService
import com.gemwallet.android.data.coordinators.asset.EnsureWalletAssetsImpl
import com.gemwallet.android.data.coordinators.asset.HideAssetImpl
import com.gemwallet.android.data.coordinators.asset.HideWelcomeBannerImpl
import com.gemwallet.android.data.coordinators.asset.PrefetchAssetsImpl
import com.gemwallet.android.data.coordinators.asset.SearchAssetsImpl
import com.gemwallet.android.data.coordinators.asset.SyncAssetInfoImpl
import com.gemwallet.android.data.coordinators.asset.SyncAssetsImpl
import com.gemwallet.android.data.coordinators.asset.ToggleAssetPinImpl
import com.gemwallet.android.data.coordinators.asset.ToggleHideBalancesImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.services.gemapi.GemApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AssetModule {
    @Provides
    @Singleton
    fun provideSearchAssets(
        gemApiClient: GemApiClient,
    ): SearchAssets = SearchAssetsImpl(
        gemApiClient = gemApiClient,
    )

    @Provides
    @Singleton
    fun provideGetActiveAssetsInfo(assetsRepository: AssetsRepository): GetActiveAssetsInfo =
        GetActiveAssetsInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetWalletSummary(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
        hasMultiSign: HasMultiSign,
        userConfig: UserConfig,
    ): GetWalletSummary = GetWalletSummaryImpl(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        hasMultiSign = hasMultiSign,
        userConfig = userConfig,
    )

    @Provides
    @Singleton
    fun provideGetAssetChartData(
        gemApiClient: GemApiClient,
        assetsRepository: AssetsRepository,
    ): GetAssetChartData = GetAssetChartDataImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
    )

    @Provides
    @Singleton
    fun providePrefetchAssets(
        gemApiClient: GemApiClient,
        assetsRepository: AssetsRepository,
    ): PrefetchAssets = PrefetchAssetsImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
    )

    @Provides
    @Singleton
    fun provideEnsureWalletAssets(
        assetsRepository: AssetsRepository,
    ): EnsureWalletAssets = EnsureWalletAssetsImpl(
        assetsRepository = assetsRepository,
    )

    @Provides
    @Singleton
    fun provideSyncAssetInfo(
        gemApiClient: GemApiClient,
        assetsRepository: AssetsRepository,
        streamSubscriptionService: StreamSubscriptionService,
    ): SyncAssetInfo = SyncAssetInfoImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
        streamSubscriptionService = streamSubscriptionService,
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
        assetsRepository: AssetsRepository,
    ): HideAsset = HideAssetImpl(sessionRepository, assetsRepository)

    @Provides
    @Singleton
    fun provideToggleAssetPin(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
    ): ToggleAssetPin = ToggleAssetPinImpl(sessionRepository, assetsRepository)

    @Provides
    @Singleton
    fun provideGetShowWelcomeBanner(
        sessionRepository: SessionRepository,
        userConfig: UserConfig,
    ): GetShowWelcomeBanner = GetShowWelcomeBannerImpl(sessionRepository, userConfig)

    @Provides
    @Singleton
    fun provideHideWelcomeBanner(
        sessionRepository: SessionRepository,
        userConfig: UserConfig,
    ): HideWelcomeBanner = HideWelcomeBannerImpl(sessionRepository, userConfig)

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
    fun provideGetImportInProgress(
        sessionRepository: SessionRepository,
        getImportWalletState: GetImportWalletState,
    ): GetImportInProgress = GetImportInProgressImpl(sessionRepository, getImportWalletState)
}
