package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.cases.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.cases.GetBuyQuotes
import com.gemwallet.android.application.fiat.cases.ObserveFiatTransactions
import com.gemwallet.android.application.fiat.cases.SyncFiatTransactions
import com.gemwallet.android.data.coordinators.fiat.GetAssetPriceUsdImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyAssetInfoImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuoteUrlImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuotesImpl
import com.gemwallet.android.data.coordinators.fiat.ObserveFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.SyncFiatTransactionsImpl
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemFiatStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneFiatStore
import javax.inject.Singleton
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo

@InstallIn(SingletonComponent::class)
@Module
object FiatModule {




    @Provides
    @Singleton
    fun provideObserveFiatTransactions(
        getSession: GetSession,
        fiatStore: GemstoneFiatStore,
    ): ObserveFiatTransactions {
        return ObserveFiatTransactionsImpl(getSession, fiatStore)
    }


    @Provides
    @Singleton
    fun provideGemstoneFiatStore(fiatTransactionsDao: FiatTransactionsDao): GemstoneFiatStore = GemstoneFiatStore(fiatTransactionsDao)

    @Provides
    @Singleton
    fun provideGemFiatStore(store: GemstoneFiatStore): GemFiatStore = store

    @Provides
    @Singleton
    fun provideSyncFiatTransactions(
        getSession: GetSession,
        fiatService: GemFiatService,
    ): SyncFiatTransactions = SyncFiatTransactionsImpl(getSession, fiatService)

    @Provides
    @Singleton
    fun provideGetBuyAssetInfo(
        getSession: GetSession,
        getAssetTokenInfo: GetAssetTokenInfo,
    ): GetBuyAssetInfo = GetBuyAssetInfoImpl(getSession, getAssetTokenInfo)

    @Provides
    @Singleton
    fun provideGetAssetPriceUsd(
        priceStore: GemstonePriceStore,
    ): GetAssetPriceUsd {
        return GetAssetPriceUsdImpl(priceStore)
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
