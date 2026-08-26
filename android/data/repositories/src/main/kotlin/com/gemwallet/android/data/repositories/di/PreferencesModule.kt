package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.data.repositories.gemstone.GemstonePreferencesStore
import com.gemwallet.android.data.repositories.pricealerts.MigratePriceAlertsPreference
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPreferencesService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PreferencesModule {

    @Singleton
    @Provides
    fun provideGemPreferencesService(@ApplicationContext context: Context): GemPreferencesService = GemPreferencesService(
        GemstonePreferencesStore(context.getSharedPreferences("gemstone_preferences", Context.MODE_PRIVATE))
    )

    @Singleton
    @Provides
    fun provideMigratePriceAlertsPreference(
        @ApplicationContext context: Context,
        preferencesService: GemPreferencesService,
    ): MigratePriceAlertsPreference = MigratePriceAlertsPreference(context, preferencesService)
}
