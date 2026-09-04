package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.data.coordinators.contacts.GetContactsImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneContactStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ContactsCasesModule {

    @Singleton
    @Provides
    fun provideGetContacts(contactStore: GemstoneContactStore): GetContacts = GetContactsImpl(contactStore)
}
