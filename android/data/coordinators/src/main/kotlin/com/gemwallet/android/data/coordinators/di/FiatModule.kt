package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.fiat.coordinators.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.ObserveFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.data.coordinators.fiat.GetAssetPriceUsdImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyAssetInfoImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuoteUrlImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuotesImpl
import com.gemwallet.android.data.coordinators.fiat.GetFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.ObserveFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.SyncFiatTransactionsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemFiatService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object FiatModule {



    @Provides
    @Singleton
    fun provideGetFiatTransactions(
        fiatService: GemFiatService,
    ): GetFiatTransactions {
        return GetFiatTransactionsImpl(
            fiatService = fiatService,
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
    fun provideGetAssetPriceUsd(
        pricesDao: PricesDao,
    ): GetAssetPriceUsd {
        return GetAssetPriceUsdImpl(pricesDao)
    }

    @Provides
    @Singleton
    fun provideGetBuyQuotes(
        fiatService: GemFiatService,
    ): GetBuyQuotes {
        return GetBuyQuotesImpl(fiatService)
    }

    @Provides
    @Singleton
    fun provideGetBuyQuoteUrl(
        fiatService: GemFiatService,
    ): GetBuyQuoteUrl {
        return GetBuyQuoteUrlImpl(fiatService)
    }

}
