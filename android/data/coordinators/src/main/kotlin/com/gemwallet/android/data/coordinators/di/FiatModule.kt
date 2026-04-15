package com.gemwallet.android.data.coordinators.di

import android.content.Context
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.application.fiat.coordinators.ObserveFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.data.coordinators.fiat.GetBuyAssetInfoImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuoteUrlImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuotesImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyableFiatAssetsImpl
import com.gemwallet.android.data.coordinators.fiat.GetFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.GetSellableFiatAssetsImpl
import com.gemwallet.android.data.coordinators.fiat.ObserveFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.SyncFiatAssetsImpl
import com.gemwallet.android.data.coordinators.fiat.SyncFiatTransactionsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Qualifier
import javax.inject.Singleton

@Qualifier
@Retention(AnnotationRetention.BINARY)
annotation class FiatConfigStore

@InstallIn(SingletonComponent::class)
@Module
object FiatModule {
    @Provides
    @Singleton
    @FiatConfigStore
    fun provideFiatConfigStore(
        @ApplicationContext context: Context,
    ): ConfigStore {
        return ConfigStore(
            context.getSharedPreferences(
                "buy_config",
                Context.MODE_PRIVATE,
            )
        )
    }

    @Provides
    @Singleton
    fun provideGetBuyableFiatAssets(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetBuyableFiatAssets {
        return GetBuyableFiatAssetsImpl(
            gemDeviceApiClient = gemDeviceApiClient,
        )
    }

    @Provides
    @Singleton
    fun provideGetSellableFiatAssets(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetSellableFiatAssets {
        return GetSellableFiatAssetsImpl(
            gemDeviceApiClient = gemDeviceApiClient,
        )
    }

    @Provides
    @Singleton
    fun provideGetFiatTransactions(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetFiatTransactions {
        return GetFiatTransactionsImpl(
            gemDeviceApiClient = gemDeviceApiClient,
        )
    }

    @Provides
    @Singleton
    fun provideObserveFiatTransactions(
        sessionRepository: SessionRepository,
        fiatTransactionsDao: FiatTransactionsDao,
    ): ObserveFiatTransactions {
        return ObserveFiatTransactionsImpl(sessionRepository, fiatTransactionsDao)
    }

    @Provides
    @Singleton
    fun provideSyncFiatAssets(
        @FiatConfigStore configStore: ConfigStore,
        getRemoteConfig: GetRemoteConfig,
        getBuyableFiatAssets: GetBuyableFiatAssets,
        getSellableFiatAssets: GetSellableFiatAssets,
        assetsRepository: AssetsRepository,
        prefetchAssets: PrefetchAssets,
    ): SyncFiatAssets {
        return SyncFiatAssetsImpl(
            configStore = configStore,
            getRemoteConfig = getRemoteConfig,
            getBuyableFiatAssets = getBuyableFiatAssets,
            getSellableFiatAssets = getSellableFiatAssets,
            assetsRepository = assetsRepository,
            prefetchAssets = prefetchAssets,
        )
    }

    @Provides
    @Singleton
    fun provideSyncFiatTransactions(
        sessionRepository: SessionRepository,
        getFiatTransactions: GetFiatTransactions,
        prefetchAssets: PrefetchAssets,
        fiatTransactionsDao: FiatTransactionsDao,
    ): SyncFiatTransactions {
        return SyncFiatTransactionsImpl(
            sessionRepository = sessionRepository,
            getFiatTransactions = getFiatTransactions,
            prefetchAssets = prefetchAssets,
            fiatTransactionsDao = fiatTransactionsDao,
        )
    }

    @Provides
    @Singleton
    fun provideGetBuyAssetInfo(
        sessionRepository: SessionRepository,
        assetsRepository: AssetsRepository,
    ): GetBuyAssetInfo {
        return GetBuyAssetInfoImpl(sessionRepository, assetsRepository)
    }

    @Provides
    @Singleton
    fun provideGetBuyQuotes(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetBuyQuotes {
        return GetBuyQuotesImpl(gemDeviceApiClient)
    }

    @Provides
    @Singleton
    fun provideGetBuyQuoteUrl(
        gemDeviceApiClient: GemDeviceApiClient,
    ): GetBuyQuoteUrl {
        return GetBuyQuoteUrlImpl(gemDeviceApiClient)
    }

}
