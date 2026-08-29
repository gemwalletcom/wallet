package com.gemwallet.android.data.adapters.di

import android.content.Context
import com.gemwallet.android.data.adapters.config.UserConfig
import com.gemwallet.android.data.service.store.ConfigStore
import dagger.Module
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSecureStore
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfigModule {

    @Singleton
    @Provides
    fun provideUserConfig(
        @ApplicationContext context: Context,
        preferencesService: GemPreferencesService,
        secureStore: GemSecureStore,
    ): UserConfig = UserConfig(
        context = context,
        configStore = ConfigStore(context.getSharedPreferences("config", Context.MODE_PRIVATE)),
        preferencesService = preferencesService,
        secureStore = secureStore,
    )
}
