package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.repositories.addresses.AddressesRepository
import com.gemwallet.android.data.service.store.database.AddressesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesModule {

    @Singleton
    @Provides
    fun provideAddressesRepository(addressesDao: AddressesDao): AddressesRepository =
        AddressesRepository(addressesDao)

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
    fun provideRenameWalletAddresses(addressesRepository: AddressesRepository): RenameWalletAddresses =
        addressesRepository
}
