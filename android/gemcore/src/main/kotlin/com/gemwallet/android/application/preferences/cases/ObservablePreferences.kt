package com.gemwallet.android.application.preferences.cases

import com.wallet.core.primitives.Appearance
import kotlinx.coroutines.flow.Flow

interface ObservablePreferences {
    fun isHideBalances(): Flow<Boolean>

    fun hideBalances()

    fun isPerpetualEnabled(): Flow<Boolean>

    fun setPerpetualEnabled(enabled: Boolean)

    fun appearance(): Flow<Appearance>

    fun setAppearance(appearance: Appearance)

    fun developEnabled(): Boolean

    fun developEnabled(enabled: Boolean)
}
