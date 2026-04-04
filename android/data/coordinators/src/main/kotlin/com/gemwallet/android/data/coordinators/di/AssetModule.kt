package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.coordinators.asset.PrefetchAssetsImpl
import com.gemwallet.android.data.coordinators.asset.GetActiveAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset.GetWalletSummaryImpl
import com.gemwallet.android.data.coordinators.asset.SyncAssetInfoImpl
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
    fun providePrefetchAssets(
        gemApiClient: GemApiClient,
        assetsRepository: AssetsRepository,
    ): PrefetchAssets = PrefetchAssetsImpl(
        gemApiClient = gemApiClient,
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
}
