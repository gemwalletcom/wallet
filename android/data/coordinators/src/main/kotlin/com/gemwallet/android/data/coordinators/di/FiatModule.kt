package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneFiatStore
import com.gemwallet.android.data.services.store.database.FiatTransactionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemFiatStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object FiatModule {

    @Provides
    @Singleton
    fun provideGemstoneFiatStore(fiatTransactionsDao: FiatTransactionsDao): GemstoneFiatStore = GemstoneFiatStore(fiatTransactionsDao)

    @Provides
    @Singleton
    fun provideGemFiatStore(store: GemstoneFiatStore): GemFiatStore = store
}
