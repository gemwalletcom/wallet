package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemConfirmTransferServiceInterface
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.data.coordinators.confirm.BuildConfirmPropertiesImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfirmModule {

    @Provides
    @Singleton
    fun provideBuildConfirmProperties(confirmService: GemConfirmTransferServiceInterface): BuildConfirmProperties = BuildConfirmPropertiesImpl(confirmService)
}
