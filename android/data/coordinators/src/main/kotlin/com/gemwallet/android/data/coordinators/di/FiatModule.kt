package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.fiat.coordinators.AddBuyRecent
import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuoteUrl
import com.gemwallet.android.application.fiat.coordinators.GetBuyQuotes
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.ObserveBuyTransactions
import com.gemwallet.android.application.fiat.coordinators.RefreshBuyTransactions
import com.gemwallet.android.data.coordinators.fiat.AddBuyRecentImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyAssetInfoImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuoteUrlImpl
import com.gemwallet.android.data.coordinators.fiat.GetBuyQuotesImpl
import com.gemwallet.android.data.coordinators.fiat.GetFiatTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.ObserveBuyTransactionsImpl
import com.gemwallet.android.data.coordinators.fiat.RefreshBuyTransactionsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object FiatModule {
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
    fun provideObserveBuyTransactions(
        sessionRepository: SessionRepository,
        buyRepository: BuyRepository,
    ): ObserveBuyTransactions {
        return ObserveBuyTransactionsImpl(sessionRepository, buyRepository)
    }

    @Provides
    @Singleton
    fun provideRefreshBuyTransactions(
        sessionRepository: SessionRepository,
        buyRepository: BuyRepository,
    ): RefreshBuyTransactions {
        return RefreshBuyTransactionsImpl(sessionRepository, buyRepository)
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
        buyRepository: BuyRepository,
    ): GetBuyQuotes {
        return GetBuyQuotesImpl(buyRepository)
    }

    @Provides
    @Singleton
    fun provideGetBuyQuoteUrl(
        buyRepository: BuyRepository,
    ): GetBuyQuoteUrl {
        return GetBuyQuoteUrlImpl(buyRepository)
    }

    @Provides
    @Singleton
    fun provideAddBuyRecent(
        assetsRepository: AssetsRepository,
    ): AddBuyRecent {
        return AddBuyRecentImpl(assetsRepository)
    }
}
