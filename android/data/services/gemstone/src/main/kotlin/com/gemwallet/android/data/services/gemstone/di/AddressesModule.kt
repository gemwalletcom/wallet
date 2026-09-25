package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.WalletsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemRecipientService
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemSignMessageService
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AddressesModule {

    @Singleton
    @Provides
    fun provideGemstoneAddressStore(addressesDao: AddressesDao, accountsDao: AccountsDao, walletsDao: WalletsDao): GemstoneAddressStore = GemstoneAddressStore(addressesDao, accountsDao, walletsDao)

    @Singleton
    @Provides
    fun provideGemAddressStore(store: GemstoneAddressStore): GemAddressStore = store

    @Singleton
    @Provides
    fun provideGemNameService(apiClient: GemDeviceApiClient, store: GemAddressStore): GemNameService = GemNameService(apiClient, store)

    @Provides
    @Singleton
    fun provideGemNameServiceInterface(service: GemNameService): GemNameServiceInterface = service

    @Provides
    fun provideGemRecipientService(payments: GemPaymentService, session: GemWalletSessionService): GemRecipientServiceInterface = GemRecipientService(payments, session)

    @Provides
    fun provideGemSignMessageServiceInterface(service: GemSignMessageService): GemSignMessageServiceInterface = service
}
