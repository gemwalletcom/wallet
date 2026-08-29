package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemSupportService
import uniffi.gemstone.GemSupportStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneSupportStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SupportChatModule {

    @Provides
    @Singleton
    fun provideGemstoneSupportStore(supportMessagesDao: SupportMessagesDao): GemstoneSupportStore = GemstoneSupportStore(supportMessagesDao)

    @Provides
    @Singleton
    fun provideGemSupportStore(supportStore: GemstoneSupportStore): GemSupportStore = supportStore

    @Provides
    @Singleton
    fun provideSupportChatRepository(
        supportService: GemSupportService,
        supportMessagesDao: SupportMessagesDao,
        supportStore: GemstoneSupportStore,
    ): SupportChatRepository = SupportChatRepository(
        supportService = supportService,
        supportMessagesDao = supportMessagesDao,
        supportStore = supportStore,
    )
}
