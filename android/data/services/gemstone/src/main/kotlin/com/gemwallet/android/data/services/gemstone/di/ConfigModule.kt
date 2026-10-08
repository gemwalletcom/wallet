package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.data.services.gemstone.config.AppLockPreferences
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.store.ConfigStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSecureStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfigModule {

    @Singleton
    @Provides
    fun provideUserConfig(preferencesService: GemPreferencesService): UserConfig = UserConfig(preferencesService = preferencesService)

    @Provides
    fun provideObservablePreferences(userConfig: UserConfig): ObservablePreferences = userConfig

    @Singleton
    @Provides
    fun provideSecurityPreferences(
        @ApplicationContext context: Context,
        secureStore: GemSecureStore,
        passwordProtection: WalletPasswordProtection,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ): SecurityPreferences = AppLockPreferences(
        context = context,
        configStore = ConfigStore(context.getSharedPreferences(AppLockPreferences.CONFIG_FILE_NAME, Context.MODE_PRIVATE)),
        secureStore = secureStore,
        passwordProtection = passwordProtection,
        ioDispatcher = ioDispatcher,
    )
}
