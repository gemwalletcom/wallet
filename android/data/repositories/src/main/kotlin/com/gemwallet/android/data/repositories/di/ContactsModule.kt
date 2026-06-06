package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.data.repositories.contacts.ContactsRepository
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ContactsModule {

    @Singleton
    @Provides
    fun provideContactsRepository(
        contactsDao: ContactsDao,
        addressesDao: AddressesDao,
    ): ContactsRepository = ContactsRepository(contactsDao, addressesDao)

    @Singleton
    @Provides
    fun provideGetContacts(repository: ContactsRepository): GetContacts = repository

    @Singleton
    @Provides
    fun provideAddContact(repository: ContactsRepository): AddContact = repository

    @Singleton
    @Provides
    fun provideUpdateContact(repository: ContactsRepository): UpdateContact = repository

    @Singleton
    @Provides
    fun provideDeleteContact(repository: ContactsRepository): DeleteContact = repository
}
