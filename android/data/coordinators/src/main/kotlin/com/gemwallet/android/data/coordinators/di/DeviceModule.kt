package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.application.device.coordinators.EnableDevicePush
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.coordinators.device.EnableDevicePushImpl
import com.gemwallet.android.data.coordinators.device.GetDeviceIdImpl
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object DeviceModule {
    @Provides
    @Singleton
    fun provideDeviceId(securityStore: SecurityStore<Any>): GetDeviceId = GetDeviceIdImpl(securityStore)

    @Provides
    @Singleton
    fun provideEnableDevicePush(
        switchPushEnabled: SwitchPushEnabled,
        walletsRepository: WalletsRepository,
    ): EnableDevicePush = EnableDevicePushImpl(switchPushEnabled, walletsRepository)
}