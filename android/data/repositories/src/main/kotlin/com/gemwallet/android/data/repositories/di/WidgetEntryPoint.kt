package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.assets.cases.GetWidgetAssets
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

@EntryPoint
@InstallIn(SingletonComponent::class)
interface WidgetEntryPoint {
    fun getWidgetAssets(): GetWidgetAssets
    fun sessionRepository(): SessionRepository
}