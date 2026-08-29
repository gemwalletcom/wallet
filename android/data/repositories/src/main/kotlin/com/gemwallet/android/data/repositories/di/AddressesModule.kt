package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.gemstone.GemstoneAddressStore
import com.gemwallet.android.data.service.store.database.AddressesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNameService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesModule {

    @Singleton
    @Provides
    fun provideGemAddressStore(addressesDao: AddressesDao): GemAddressStore = GemstoneAddressStore(addressesDao)

    @Singleton
    @Provides
    fun provideGemNameService(apiClient: GemDeviceApiClient, store: GemAddressStore): GemNameService = GemNameService(apiClient, store)

}
