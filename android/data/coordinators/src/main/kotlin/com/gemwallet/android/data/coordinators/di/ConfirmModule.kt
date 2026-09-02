package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.data.coordinators.confirm.BuildConfirmPropertiesImpl
import com.gemwallet.android.application.stake.cases.GetStakeValidator
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
    fun provideBuildConfirmProperties(
        getStakeValidator: GetStakeValidator,
        explorerService: GemExplorerService,
    ): BuildConfirmProperties = BuildConfirmPropertiesImpl(
        getStakeValidator,
        explorerService,
    )
}
