package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.data.repositories.support.SupportTypingState
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SupportChatModule {

    @Provides
    @Singleton
    fun provideSupportTypingState(): SupportTypingState = SupportTypingState()

    @Provides
    @Singleton
    fun provideSupportChatRepository(
        gemDeviceApiClient: GemDeviceApiClient,
        supportMessagesDao: SupportMessagesDao,
        supportTypingState: SupportTypingState,
    ): SupportChatRepository = SupportChatRepository(
        gemDeviceApiClient = gemDeviceApiClient,
        supportMessagesDao = supportMessagesDao,
        supportTypingState = supportTypingState,
    )
}
