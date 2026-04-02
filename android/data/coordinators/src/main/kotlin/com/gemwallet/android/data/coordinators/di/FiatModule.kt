package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.coordinators.fiat.GetFiatTransactionsImpl
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
}
