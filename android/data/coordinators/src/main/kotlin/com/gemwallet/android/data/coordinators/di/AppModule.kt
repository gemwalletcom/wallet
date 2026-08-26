package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.blockchain.services.GemSignAuthOperator
import com.gemwallet.android.data.coordinators.GetAuthPayloadImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAuthService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AppModule {
    @Provides
    @Singleton
    fun provideGetAuthPayload(
        authService: GemAuthService,
        getDeviceId: GetDeviceId,
        passwordStore: PasswordStore,
        signAuthOperator: GemSignAuthOperator,
    ): GetAuthPayload {
        return GetAuthPayloadImpl(
            authService = authService,
            getDeviceId = getDeviceId,
            passwordStore = passwordStore,
            signAuthOperator = signAuthOperator,
        )
    }
}