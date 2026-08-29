package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.data.coordinators.contacts.AddContactImpl
import com.gemwallet.android.data.coordinators.contacts.DeleteContactImpl
import com.gemwallet.android.data.coordinators.contacts.GetContactsImpl
import com.gemwallet.android.data.coordinators.contacts.UpdateContactImpl
import com.gemwallet.android.data.repositories.gemstone.GemstoneContactStore
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
    fun provideAddContact(contactService: GemContactService): AddContact = AddContactImpl(contactService)

    @Singleton
    @Provides
    fun provideUpdateContact(contactService: GemContactService): UpdateContact = UpdateContactImpl(contactService)

    @Singleton
    @Provides
    fun provideDeleteContact(contactService: GemContactService): DeleteContact = DeleteContactImpl(contactService)
}
