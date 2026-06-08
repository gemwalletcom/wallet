package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.GetAuthPayload
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.blockchain.services.GemSignAuthOperator
import com.gemwallet.android.data.coordinators.GetAuthPayloadImpl
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AppModule {
    @Provides
    @Singleton
    fun provideGetAuthPayload(
        gemDeviceApiClient: GemDeviceApiClient,
        getDeviceId: GetDeviceId,
        passwordStore: PasswordStore,
        signAuthOperator: GemSignAuthOperator,
    ): GetAuthPayload {
        return GetAuthPayloadImpl(
            gemDeviceApiClient = gemDeviceApiClient,
            getDeviceId = getDeviceId,
            passwordStore = passwordStore,
            signAuthOperator = signAuthOperator,
        )
    }
}