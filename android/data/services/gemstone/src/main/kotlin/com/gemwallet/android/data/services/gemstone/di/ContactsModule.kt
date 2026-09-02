package com.gemwallet.android.data.services.gemstone.di

import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemContactServiceInterface
import uniffi.gemstone.GemManageContactService
import uniffi.gemstone.GemManageContactServiceInterface
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemPaymentService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneContactStore
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemContactService
import uniffi.gemstone.GemContactStore
import javax.inject.Singleton
import uniffi.gemstone.GemFileStore

@InstallIn(SingletonComponent::class)
@Module
object ContactsModule {

    @Singleton
    @Provides
    fun provideGemstoneContactStore(contactsDao: ContactsDao, addressesDao: AddressesDao): GemstoneContactStore =
        GemstoneContactStore(contactsDao)

    @Singleton
    @Provides
    fun provideGemContactStore(store: GemstoneContactStore): GemContactStore = store

    @Singleton
    @Provides
    fun provideGemContactService(store: GemContactStore, addressStore: GemAddressStore, fileStore: GemFileStore): GemContactService =
        GemContactService(store, addressStore, fileStore)

    @Provides
    fun provideGemContactServiceInterface(service: GemContactService): GemContactServiceInterface = service

    @Provides
    fun provideGemManageContactService(
        contacts: GemContactService,
        addresses: GemAddressService,
        names: GemNameService,
        chains: GemChainService,
        payments: GemPaymentService,
    ): GemManageContactServiceInterface = GemManageContactService(contacts, addresses, names, chains, payments)
}
