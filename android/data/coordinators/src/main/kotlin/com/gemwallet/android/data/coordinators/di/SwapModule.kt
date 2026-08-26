package com.gemwallet.android.data.coordinators.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
import com.gemwallet.android.application.swap.coordinators.GetSwapQuoteData
import com.gemwallet.android.application.swap.coordinators.GetSwapQuotes
import com.gemwallet.android.application.swap.coordinators.GetSwapSupported
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.application.swap.coordinators.SyncSwapAssets
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.data.coordinators.swap.BuildSwapConfirmParamsImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapAssetsImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapQuoteDataImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.GetSwapSupportedImpl
import com.gemwallet.android.data.coordinators.swap.RequestSwapQuotesImpl
import com.gemwallet.android.data.coordinators.swap.SearchSwapAssetsImpl
import com.gemwallet.android.data.coordinators.swap.SyncSwapAssetsImpl
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.ConfigStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemSwapper
import javax.inject.Qualifier
import uniffi.gemstone.GemAssetsService
import javax.inject.Singleton

@Qualifier
@Retention(AnnotationRetention.BINARY)
annotation class SwapConfigStore

@InstallIn(SingletonComponent::class)
@Module
object SwapModule {

    @Provides
    @Singleton
    @SwapConfigStore
    fun provideSwapConfigStore(
        @ApplicationContext context: Context,
    ): ConfigStore {
        return ConfigStore(
            context.getSharedPreferences(
                "swap_config",
                Context.MODE_PRIVATE,
            )
        )
    }

    @Singleton
    @Provides
    fun provideGemSwapper(
        alienProvider: AlienProvider,
    ): GemSwapper = GemSwapper(alienProvider)

    @Singleton
    @Provides
    fun provideGetSwapQuotes(
        gemSwapper: GemSwapper,
    ): GetSwapQuotes = GetSwapQuotesImpl(gemSwapper)

    @Singleton
    @Provides
    fun provideGetSwapSupported(
        gemSwapper: GemSwapper,
    ): GetSwapSupported = GetSwapSupportedImpl(gemSwapper)

    @Singleton
    @Provides
    fun provideGetSwapQuoteData(
        gemSwapper: GemSwapper,
        passwordStore: PasswordStore,
        signMessageOperator: GemSignMessageOperator,
    ): GetSwapQuoteData = GetSwapQuoteDataImpl(
        gemSwapper = gemSwapper,
        passwordStore = passwordStore,
        signMessageOperator = signMessageOperator,
    )

    @Singleton
    @Provides
    fun provideRequestSwapQuotes(
        getSwapQuotes: GetSwapQuotes,
    ): RequestSwapQuotes = RequestSwapQuotesImpl(getSwapQuotes)

    @Singleton
    @Provides
    fun provideBuildSwapConfirmParams(
        sessionRepository: SessionRepository,
        getSwapQuoteData: GetSwapQuoteData,
    ): BuildSwapConfirmParams = BuildSwapConfirmParamsImpl(
        sessionRepository = sessionRepository,
        getSwapQuoteData = getSwapQuoteData,
    )

    @Singleton
    @Provides
    fun provideSearchSwapAssets(
        searchService: AssetsSearchService,
        getSwapSupported: GetSwapSupported,
    ): SearchSwapAssets = SearchSwapAssetsImpl(
        searchService = searchService,
        getSwapSupported = getSwapSupported,
    )

    @Singleton
    @Provides
    fun provideGetSwapAssets(
        assetsService: GemAssetsService,
    ): GetSwapAssets = GetSwapAssetsImpl(
        assetsService = assetsService,
    )

    @Singleton
    @Provides
    fun provideSyncSwapAssets(
        @SwapConfigStore configStore: ConfigStore,
        getSwapAssets: GetSwapAssets,
        assetsRepository: AssetsRepository,
        availabilityService: AssetsAvailabilityService,
        prefetchAssets: PrefetchAssets,
    ): SyncSwapAssets = SyncSwapAssetsImpl(
        configStore = configStore,
        getSwapAssets = getSwapAssets,
        assetsRepository = assetsRepository,
        availabilityService = availabilityService,
        prefetchAssets = prefetchAssets,
    )
}
