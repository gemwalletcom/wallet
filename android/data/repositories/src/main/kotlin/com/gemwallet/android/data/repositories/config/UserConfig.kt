package com.gemwallet.android.data.repositories.config

import android.content.Context
import android.text.format.DateUtils
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.data.service.store.ConfigStore
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemPreferencesService
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map

private val Context.dataStore by preferencesDataStore(name = "user_config")

class UserConfig(
    private val context: Context,
    private val configStore: ConfigStore,
    private val preferencesService: GemPreferencesService,
) {

    fun authRequired(): Boolean = configStore.getBoolean(ConfigKey.Auth.string)

    fun setAuthRequired(enabled: Boolean) = configStore.putBoolean(ConfigKey.Auth.string, enabled)

    fun developEnabled(): Boolean = configStore.getBoolean(ConfigKey.DevelopEnabled.string)

    fun developEnabled(enabled: Boolean) = configStore.putBoolean(ConfigKey.DevelopEnabled.string, enabled)

    fun getLaunchNumber(): Int = configStore.getInt(ConfigKey.LaunchNumber.string)

    fun increaseLaunchNumber() = configStore.putInt(ConfigKey.LaunchNumber.string, getLaunchNumber() + 1)

    fun chartPeriod(): ChartPeriod = configStore.getString(ConfigKey.ChartPeriod.string).toChartPeriod()

    fun setChartPeriod(period: ChartPeriod) = configStore.putString(ConfigKey.ChartPeriod.string, period.string)

    fun perpetualChartPeriod(): ChartPeriod =
        configStore.getString(ConfigKey.PerpetualChartPeriod.string).toChartPeriod()

    fun setPerpetualChartPeriod(period: ChartPeriod) =
        configStore.putString(ConfigKey.PerpetualChartPeriod.string, period.string)

    fun isHideBalances(): Flow<Boolean> = read(Key.IsHideBalances, false)

    suspend fun hideBalances() = update(Key.IsHideBalances, false) { !it }

    fun isPerpetualEnabled(): Flow<Boolean> = read(Key.IsPerpetualEnabled, false)

    suspend fun setPerpetualEnabled(enabled: Boolean) = write(Key.IsPerpetualEnabled, enabled)

    fun appearance(): Flow<Appearance> =
        read(Key.Appearance, "").map { value ->
            Appearance.entries.firstOrNull { it.string == value } ?: Appearance.System
        }

    suspend fun setAppearance(appearance: Appearance) = write(Key.Appearance, appearance.string)



    private val perpetualLeverageState = MutableStateFlow(preferencesService.getPerpetualLeverage().toInt())
    private val perpetualTakeProfitState = MutableStateFlow(preferencesService.getPerpetualTakeProfitPercent().toInt())
    private val perpetualStopLossState = MutableStateFlow(preferencesService.getPerpetualStopLossPercent().toInt())
    private val swapSlippageBpsState = MutableStateFlow(preferencesService.getSwapSlippageBps())

    fun perpetualLeverage(): Flow<Int> = perpetualLeverageState

    fun setPerpetualLeverage(value: Int) {
        preferencesService.setPerpetualLeverage(value.toUByte())
        perpetualLeverageState.value = preferencesService.getPerpetualLeverage().toInt()
    }

    fun perpetualTakeProfit(): Flow<Int> = perpetualTakeProfitState

    fun setPerpetualTakeProfit(value: Int) {
        preferencesService.setPerpetualTakeProfitPercent(value.toUByte())
        perpetualTakeProfitState.value = preferencesService.getPerpetualTakeProfitPercent().toInt()
    }

    fun perpetualStopLoss(): Flow<Int> = perpetualStopLossState

    fun setPerpetualStopLoss(value: Int) {
        preferencesService.setPerpetualStopLossPercent(value.toUByte())
        perpetualStopLossState.value = preferencesService.getPerpetualStopLossPercent().toInt()
    }

    fun swapSlippageBps(): Flow<UInt?> = swapSlippageBpsState

    fun setSwapSlippageBps(bps: UInt?) {
        preferencesService.setSwapSlippageBps(bps)
        swapSlippageBpsState.value = preferencesService.getSwapSlippageBps()
    }

    fun getLockInterval(): Flow<Int> = read(Key.LockInterval, 1)

    suspend fun setLockInterval(minutes: Int) = write(Key.LockInterval, minutes)

    fun isTermsAccepted(): Flow<Boolean> = read(Key.IsTermsAccepted, false)

    suspend fun acceptTerms() = write(Key.IsTermsAccepted, true)

    fun isAskNotifications(): Flow<Boolean> = read(Key.AskNotifications, 0L)
        .map { it < System.currentTimeMillis() - 30 * DateUtils.DAY_IN_MILLIS }

    suspend fun stopAskNotifications() = write(Key.AskNotifications, System.currentTimeMillis())

    private fun <T> read(key: Preferences.Key<T>, default: T): Flow<T> =
        context.dataStore.data.map { it[key] ?: default }

    private suspend fun <T> write(key: Preferences.Key<T>, value: T) {
        context.dataStore.edit { it[key] = value }
    }

    private suspend fun <T> update(key: Preferences.Key<T>, default: T, transform: (T) -> T) {
        context.dataStore.edit { it[key] = transform(it[key] ?: default) }
    }

    private fun String.toChartPeriod(): ChartPeriod =
        ChartPeriod.entries.firstOrNull { it.string == this } ?: ChartPeriod.Day

    private enum class ConfigKey(val string: String) {
        Auth("auth"),
        ChartPeriod("chart_period"),
        DevelopEnabled("develop_enabled"),
        PerpetualChartPeriod("perpetual_chart_period"),
        LaunchNumber("launch_number"),
        ;
    }

    private object Key {
        val IsHideBalances = booleanPreferencesKey("hide_balances")
        val LockInterval = intPreferencesKey("lock_interval")
        val IsTermsAccepted = booleanPreferencesKey("is_terms_accepted")
        val AskNotifications = longPreferencesKey("ask_notifications")
        val IsPerpetualEnabled = booleanPreferencesKey("is_perpetual_enabled")
        val Appearance = stringPreferencesKey("appearance")
    }
}
