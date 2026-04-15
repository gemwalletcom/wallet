package com.gemwallet.android.features.update_app.viewmodels

import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class InAppUpdateModule {
    @Singleton
    @Binds
    abstract fun bindInAppUpdateService(
        service: InAppUpdateServiceImpl,
    ): InAppUpdateService
}
