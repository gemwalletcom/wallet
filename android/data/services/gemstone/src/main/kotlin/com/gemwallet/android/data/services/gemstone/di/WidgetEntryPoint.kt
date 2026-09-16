package com.gemwallet.android.data.services.gemstone.di

import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWidgetService

@EntryPoint
@InstallIn(SingletonComponent::class)
interface WidgetEntryPoint {
    fun widgetService(): GemWidgetService
    fun preferencesService(): GemPreferencesService
}
