package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.GetAddressNames
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.repositories.addresses.AddressesRepository
import com.gemwallet.android.data.repositories.addresses.GemstoneAddressStore
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

    @Singleton
    @Provides
    fun provideAddressesRepository(
        addressesDao: AddressesDao,
        nameService: GemNameService,
    ): AddressesRepository =
        AddressesRepository(addressesDao, nameService)

    @Singleton
    @Provides
    fun provideSaveAddressNames(addressesRepository: AddressesRepository): SaveAddressNames =
        addressesRepository

    @Singleton
    @Provides
    fun provideGetAddressName(addressesRepository: AddressesRepository): GetAddressName =
        addressesRepository

    @Singleton
    @Provides
    fun provideGetAddressNames(addressesRepository: AddressesRepository): GetAddressNames =
        addressesRepository

    @Singleton
    @Provides
    fun provideRenameWalletAddresses(addressesRepository: AddressesRepository): RenameWalletAddresses =
        addressesRepository
}
