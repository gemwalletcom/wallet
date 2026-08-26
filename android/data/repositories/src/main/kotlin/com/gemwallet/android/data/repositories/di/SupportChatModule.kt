package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.data.repositories.support.SupportTypingState
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
    fun provideSupportTypingState(): SupportTypingState = SupportTypingState()

    @Provides
    @Singleton
    fun provideGemSupportStore(supportMessagesDao: SupportMessagesDao): GemSupportStore = GemstoneSupportStore(supportMessagesDao)

    @Provides
    @Singleton
    fun provideSupportChatRepository(
        supportService: GemSupportService,
        supportMessagesDao: SupportMessagesDao,
        supportTypingState: SupportTypingState,
    ): SupportChatRepository = SupportChatRepository(
        supportService = supportService,
        supportMessagesDao = supportMessagesDao,
        supportTypingState = supportTypingState,
    )
}
