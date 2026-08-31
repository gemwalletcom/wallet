package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.contacts.cases.AddContactAddress
import com.gemwallet.android.application.contacts.cases.DeleteContact
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.contacts.cases.SaveContact
import com.gemwallet.android.data.coordinators.contacts.ContactsCoordinator
import com.gemwallet.android.data.coordinators.contacts.GetContactsImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneContactStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemContactService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ContactsCasesModule {

    @Singleton
    @Provides
    fun provideGetContacts(contactStore: GemstoneContactStore): GetContacts = GetContactsImpl(contactStore)

    @Singleton
    @Provides
    fun provideContactsCoordinator(contactService: GemContactService): ContactsCoordinator = ContactsCoordinator(contactService)

    @Singleton
    @Provides
    fun provideSaveContact(coordinator: ContactsCoordinator): SaveContact = coordinator

    @Singleton
    @Provides
    fun provideAddContactAddress(coordinator: ContactsCoordinator): AddContactAddress = coordinator

    @Singleton
    @Provides
    fun provideDeleteContact(coordinator: ContactsCoordinator): DeleteContact = coordinator
}
