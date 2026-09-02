package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.domains.name.AddressInputResolving
import com.gemwallet.android.ext.addressInput
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemRecipientService
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemWalletSessionService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.gemwallet.android.data.service.store.database.AddressesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemSignMessageService
import uniffi.gemstone.GemSignMessageServiceInterface
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

    @Provides
    fun provideAddressInputResolving(service: GemNameServiceInterface): AddressInputResolving = service.addressInput()

    @Provides
    fun provideGemRecipientService(
        names: GemNameService,
        payments: GemPaymentService,
        session: GemWalletSessionService,
    ): GemRecipientServiceInterface = GemRecipientService(names, payments, session)

    @Provides
    fun provideGemSignMessageService(names: GemNameService, explorer: GemExplorerService): GemSignMessageServiceInterface =
        GemSignMessageService(names, explorer)

}
