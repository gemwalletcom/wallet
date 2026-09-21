package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneContactStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemContactService
import uniffi.gemstone.GemContactServiceInterface
import uniffi.gemstone.GemContactStore
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemManageContactService
import uniffi.gemstone.GemManageContactServiceInterface
import uniffi.gemstone.GemPaymentService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ContactsModule {

    @Singleton
    @Provides
    fun provideGemstoneContactStore(contactsDao: ContactsDao, addressesDao: AddressesDao): GemstoneContactStore = GemstoneContactStore(contactsDao)

    @Singleton
    @Provides
    fun provideGemContactStore(store: GemstoneContactStore): GemContactStore = store

    @Singleton
    @Provides
    fun provideGemContactService(store: GemContactStore, addressStore: GemAddressStore, fileStore: GemFileStore): GemContactService = GemContactService(store, addressStore, fileStore)

    @Provides
    fun provideGemContactServiceInterface(service: GemContactService): GemContactServiceInterface = service

    @Provides
    fun provideGemManageContactService(contacts: GemContactService, addresses: GemAddressService, payments: GemPaymentService): GemManageContactServiceInterface = GemManageContactService(contacts, addresses, payments)
}
