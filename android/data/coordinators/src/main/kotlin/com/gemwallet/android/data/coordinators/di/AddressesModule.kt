package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.cases.addresses.GetAddressNames
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.cases.addresses.SaveWalletAddresses
import com.gemwallet.android.data.coordinators.addresses.GetAddressNameImpl
import com.gemwallet.android.data.coordinators.addresses.GetAddressNamesImpl
import com.gemwallet.android.data.coordinators.addresses.RenameWalletAddressesImpl
import com.gemwallet.android.data.coordinators.addresses.SaveWalletAddressesImpl
import com.gemwallet.android.data.repositories.gemstone.GemstoneAddressStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemNameService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesCasesModule {

    @Singleton
    @Provides
    fun provideGetAddressName(addressStore: GemstoneAddressStore): GetAddressName = GetAddressNameImpl(addressStore)

    @Singleton
    @Provides
    fun provideGetAddressNames(nameService: GemNameService): GetAddressNames = GetAddressNamesImpl(nameService)

    @Singleton
    @Provides
    fun provideRenameWalletAddresses(addressStore: GemstoneAddressStore): RenameWalletAddresses = RenameWalletAddressesImpl(addressStore)

    @Singleton
    @Provides
    fun provideSaveWalletAddresses(addressStore: GemstoneAddressStore): SaveWalletAddresses = SaveWalletAddressesImpl(addressStore)
}
