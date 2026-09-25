package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.data.services.store.ConfigStore
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSecureStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfigModule {

    @Singleton
    @Provides
    fun provideUserConfig(@ApplicationContext context: Context, preferencesService: GemPreferencesService, secureStore: GemSecureStore): UserConfig = UserConfig(
        context = context,
        configStore = ConfigStore(context.getSharedPreferences("config", Context.MODE_PRIVATE)),
        preferencesService = preferencesService,
        secureStore = secureStore,
    )
}
