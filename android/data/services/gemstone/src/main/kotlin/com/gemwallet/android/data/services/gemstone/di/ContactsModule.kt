package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneContactStore
import com.gemwallet.android.data.services.store.database.AddressesDao
import com.gemwallet.android.data.services.store.database.ContactsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemContactEditorService
import uniffi.gemstone.GemContactEditorServiceInterface
import uniffi.gemstone.GemContactService
import uniffi.gemstone.GemContactServiceInterface
import uniffi.gemstone.GemContactStore
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemNameService
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
    fun provideGemContactService(store: GemContactStore, nameService: GemNameService, fileStore: GemFileStore): GemContactService = GemContactService(store, nameService, fileStore)

    @Provides
    fun provideGemContactServiceInterface(service: GemContactService): GemContactServiceInterface = service

    @Provides
    fun provideGemContactEditorService(contacts: GemContactService, payments: GemPaymentService): GemContactEditorServiceInterface = GemContactEditorService(contacts, payments)
}
