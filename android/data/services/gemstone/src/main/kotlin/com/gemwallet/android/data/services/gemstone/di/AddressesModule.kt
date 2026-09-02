package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.gemwallet.android.data.service.store.database.AddressesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNameServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesModule {

    @Singleton
    @Provides
    fun provideGemstoneAddressStore(addressesDao: AddressesDao): GemstoneAddressStore = GemstoneAddressStore(addressesDao)

    @Singleton
    @Provides
    fun provideGemAddressStore(store: GemstoneAddressStore): GemAddressStore = store

    @Singleton
    @Provides
    fun provideGemNameService(apiClient: GemDeviceApiClient, store: GemAddressStore): GemNameService = GemNameService(apiClient, store)

    @Provides
    @Singleton
    fun provideGemNameServiceInterface(service: GemNameService): GemNameServiceInterface = service

}
