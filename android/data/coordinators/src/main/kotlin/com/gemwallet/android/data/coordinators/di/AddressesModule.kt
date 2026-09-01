package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.addresses.cases.RenameWalletAddresses
import com.gemwallet.android.application.addresses.cases.SaveWalletAddresses
import com.gemwallet.android.data.coordinators.addresses.RenameWalletAddressesImpl
import com.gemwallet.android.data.coordinators.addresses.SaveWalletAddressesImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesCasesModule {

    @Singleton
    @Provides
    fun provideRenameWalletAddresses(addressStore: GemstoneAddressStore): RenameWalletAddresses = RenameWalletAddressesImpl(addressStore)

    @Singleton
    @Provides
    fun provideSaveWalletAddresses(addressStore: GemstoneAddressStore): SaveWalletAddresses = SaveWalletAddressesImpl(addressStore)
}
