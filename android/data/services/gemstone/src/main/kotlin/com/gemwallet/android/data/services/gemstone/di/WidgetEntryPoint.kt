package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.assets.cases.GetWidgetAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

@EntryPoint
@InstallIn(SingletonComponent::class)
interface WidgetEntryPoint {
    fun getWidgetAssets(): GetWidgetAssets
    fun getCurrentCurrency(): GetCurrentCurrency
}